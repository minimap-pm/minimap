//! An in-memory Minimap workspace, useful for testing.

use crate::{Error, Record, RecordBuilder, Result, SetOperation, Workspace};
use sha2::{Digest, Sha256};
use std::{
	collections::HashMap,
	hash::Hash,
	sync::{Arc, Mutex},
	time::{SystemTime, UNIX_EPOCH},
};

/// A memory record for in-memory workspaces.
#[derive(Clone)]
pub struct MemoryRecord {
	id: String,
	parent: Option<String>,
	author: String,
	email: String,
	message: String,
	timestamp: i64,
	op: Option<SetOperation>,
	attachments: HashMap<String, String>,
}

impl Hash for MemoryRecord {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.id.hash(state);
	}
}

impl PartialEq for MemoryRecord {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}
}

impl Eq for MemoryRecord {}

impl std::fmt::Debug for MemoryRecord {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		// just format the ID
		self.id.fmt(f)
	}
}

#[derive(Default)]
struct State {
	total_ids: u64,
	heads: HashMap<String, String>,
	attachment_pool: HashMap<String, Vec<u8>>,
	records: HashMap<String, MemoryRecord>,
}

impl State {
	fn next_id(&mut self) -> String {
		self.total_ids += 1;
		let id = format!("MINIMAPINMEMORY::{:x}::MINIMAPINMEMORY", self.total_ids);
		let mut sha = Sha256::new();
		sha.update(id.as_bytes());
		format!("{:x}", sha.finalize())
	}
}

/// An in-memory Minimap workspace, useful for testing.
#[derive(Default, Clone)]
pub struct MemoryWorkspace {
	author: String,
	email: String,
	state: Arc<Mutex<State>>,
}

impl MemoryWorkspace {
	/// Creates a new in-memory workspace.
	pub fn new(author: &str, email: &str) -> Self {
		Self {
			author: author.to_string(),
			email: email.to_string(),
			..Self::default()
		}
	}

	/// Uses sha256 to generate a unique ID for a record.
	fn insert_attachment(&self, data: Vec<u8>) -> String {
		let mut state = self.state.lock().unwrap();
		let mut sha = Sha256::new();
		sha.update(data.as_slice());
		let id = format!("{:x}", sha.finalize());
		state.attachment_pool.insert(id.clone(), data);
		id
	}
}

/// A reference to a record in an in-memory workspace.
/// This is the primary external type for interacting with
/// in-memory workspace records.
#[derive(Clone)]
pub struct MemoryRecordRef(Arc<Mutex<State>>, MemoryRecord);

impl std::fmt::Debug for MemoryRecordRef
where
	MemoryRecord: std::fmt::Debug,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.1.fmt(f)
	}
}

impl PartialEq for MemoryRecordRef
where
	MemoryRecord: PartialEq,
{
	fn eq(&self, other: &Self) -> bool {
		self.1.eq(&other.1)
	}
}

impl Eq for MemoryRecordRef where MemoryRecord: Eq {}

impl Hash for MemoryRecordRef
where
	MemoryRecord: Hash,
{
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.1.hash(state);
	}
}

impl<'a> Workspace<'a> for MemoryWorkspace {
	type Record = MemoryRecordRef;
	type RecordBuilder = MemoryRecordBuilder<'a>;
	type Iterator = MemoryIterator;
	type SetIterator = MemorySetIterator;

	fn walk(&'a self, collection: &str) -> Result<Self::Iterator> {
		let state = self.state.lock().unwrap();
		let next = state
			.heads
			.get(collection)
			.and_then(|id| state.records.get(id).cloned());
		Ok(MemoryIterator(
			self.state.clone(),
			next.map(|record| MemoryRecordRef(self.state.clone(), record)),
		))
	}

	fn record_builder(&'a self, collection: &str) -> Self::RecordBuilder {
		MemoryRecordBuilder::new(self, collection.to_string())
	}

	fn set_add_unchecked(&'a self, collection: &str, message: &str) -> Result<Self::Record> {
		self.record_builder(collection)
			.op(SetOperation::Add)
			.commit(message)
	}

	fn set_del_unchecked(&'a self, collection: &str, message: &str) -> Result<Self::Record> {
		self.record_builder(collection)
			.op(SetOperation::Del)
			.commit(message)
	}

	fn walk_set(&'a self, collection: &str) -> Result<Self::SetIterator> {
		self.walk(collection).map(MemorySetIterator)
	}

	fn get_record(&'a self, id: &str) -> Result<Option<Self::Record>> {
		let state = self.state.lock().unwrap();
		Ok(state
			.records
			.get(id)
			.cloned()
			.map(|record| MemoryRecordRef(self.state.clone(), record)))
	}
}

impl Record for MemoryRecordRef {
	#[inline]
	fn id(&self) -> String {
		self.1.id.clone()
	}

	#[inline]
	fn author(&self) -> String {
		self.1.author.clone()
	}

	#[inline]
	fn email(&self) -> String {
		self.1.email.clone()
	}

	#[inline]
	fn message(&self) -> String {
		self.1.message.clone()
	}

	#[inline]
	fn timestamp(&self) -> i64 {
		self.1.timestamp
	}

	fn attachment(&self, name: &str) -> Result<Option<Vec<u8>>> {
		let id = match self.1.attachments.get(name) {
			Some(id) => id,
			None => return Ok(None),
		};

		let state = self.0.lock().unwrap();
		Ok(state.attachment_pool.get(id).cloned())
	}
}

/// The iterator type for [`MemoryWorkspace`].
pub struct MemoryIterator(Arc<Mutex<State>>, Option<MemoryRecordRef>);

impl Iterator for MemoryIterator {
	type Item = Result<MemoryRecordRef>;

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		let state = self.0.lock().unwrap();
		let (record, next) = {
			let next = self.1.as_ref().and_then(|record| {
				record
					.1
					.parent
					.as_ref()
					.and_then(|parent| state.records.get(parent).cloned())
			});
			(self.1.take(), next)
		};
		self.1 = next.map(|r| MemoryRecordRef(self.0.clone(), r.clone()));
		record.map(Ok)
	}
}

/// The set iterator type for [`MemoryWorkspace`].
pub struct MemorySetIterator(MemoryIterator);

impl Iterator for MemorySetIterator {
	type Item = Result<(MemoryRecordRef, SetOperation)>;

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		let record = match self.0.next() {
			Some(Ok(record)) => record,
			Some(Err(e)) => return Some(Err(e)),
			None => return None,
		};

		if let Some(op) = record.1.op {
			Some(Ok((record, op)))
		} else {
			Some(Err(Error::Malformed(format!(
				"record {} is not a set operation",
				record.1.id
			))))
		}
	}
}

/// The record builder type for [`MemoryWorkspace`].
pub struct MemoryRecordBuilder<'a> {
	workspace: &'a MemoryWorkspace,
	collection: String,
	attachments: HashMap<String, Option<String>>,
	op: Option<SetOperation>,
}

impl<'a> MemoryRecordBuilder<'a> {
	fn new(workspace: &'a MemoryWorkspace, collection: String) -> Self {
		Self {
			workspace,
			collection,
			attachments: HashMap::new(),
			op: None,
		}
	}

	fn op(self, op: SetOperation) -> Self {
		Self {
			op: Some(op),
			..self
		}
	}
}

impl<'a> RecordBuilder<'a> for MemoryRecordBuilder<'a> {
	type Record = MemoryRecordRef;

	fn upsert_attachment<D: AsRef<[u8]>>(mut self, name: &str, data: D) -> Result<Self> {
		let data = data.as_ref().to_vec();
		let id = self.workspace.insert_attachment(data);
		self.attachments.insert(name.to_string(), Some(id.clone()));
		Ok(self)
	}

	fn remove_attachment(mut self, name: &str) -> Result<Self> {
		self.attachments.insert(name.to_string(), None);
		Ok(self)
	}

	fn commit(self, message: &str) -> Result<Self::Record> {
		let mut state = self.workspace.state.lock().unwrap();
		let timestamp = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.unwrap()
			.as_secs() as i64;
		let id = state.next_id();

		// get the latest record, clone its attachments, and then
		// apply the updates from the builder
		let parent_id = state.heads.get(&self.collection);
		let mut attachments = parent_id
			.and_then(|p| state.records.get(p))
			.map(|r| r.attachments.clone())
			.unwrap_or_default();

		for (name, id) in self.attachments {
			if let Some(id) = id {
				attachments.insert(name, id);
			} else {
				attachments.remove(name.as_str());
			}
		}

		let record = MemoryRecord {
			id: id.clone(),
			message: message.to_string(),
			author: self.workspace.author.clone(),
			email: self.workspace.email.clone(),
			timestamp,
			op: self.op,
			attachments,
			parent: parent_id.cloned(),
		};

		state.records.insert(id.clone(), record.clone());
		state.heads.insert(self.collection, id.clone());

		Ok(MemoryRecordRef(self.workspace.state.clone(), record))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	macro_rules! create_test_workspace {
		() => {
			MemoryWorkspace::new("Max Mustermann", "max@example.com")
		};
	}

	include!("../acceptance-tests.inc.rs");
}
