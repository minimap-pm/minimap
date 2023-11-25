//! An in-memory Minimap workspace, useful for testing.

use crate::{Error, Record, RecordBuilder, Result, SetOperation, Workspace};
use sha2::{Digest, Sha256};
use std::{
	collections::HashMap,
	hash::Hash,
	sync::{Arc, Mutex, MutexGuard},
	time::{SystemTime, UNIX_EPOCH},
};

/// A memory record for in-memory workspaces.
#[derive(Debug, Clone)]
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

impl Workspace for MemoryWorkspace {
	type Record<'a> = MemoryRecord;
	type RecordBuilder<'a> = MemoryRecordBuilder<'a>;
	type Iterator<'a> = MemoryIterator<'a>;
	type SetIterator<'a> = MemorySetIterator<'a>;

	fn walk<'a>(&'a self, collection: &str) -> Result<Self::Iterator<'a>> {
		let state = self.state.lock().unwrap();
		let next = state
			.heads
			.get(collection)
			.and_then(|id| state.records.get(id).cloned());
		Ok(MemoryIterator(state, next))
	}

	fn record_builder<'a>(&'a self, collection: &str) -> Self::RecordBuilder<'a> {
		MemoryRecordBuilder::new(self, collection.to_string())
	}

	fn set_add_unchecked<'a>(
		&'a self,
		collection: &str,
		message: &str,
	) -> Result<Self::Record<'a>> {
		self.record_builder(collection)
			.op(SetOperation::Add)
			.commit(message)
	}

	fn set_del_unchecked<'a>(
		&'a self,
		collection: &str,
		message: &str,
	) -> Result<Self::Record<'a>> {
		self.record_builder(collection)
			.op(SetOperation::Del)
			.commit(message)
	}

	fn walk_set<'a>(&'a self, collection: &str) -> Result<Self::SetIterator<'a>> {
		self.walk(collection).map(MemorySetIterator)
	}

	fn get_record<'a>(&'a self, id: &str) -> Result<Option<Self::Record<'a>>> {
		let state = self.state.lock().unwrap();
		Ok(state.records.get(id).cloned())
	}
}

impl Record for MemoryRecord {
	#[inline]
	fn id(&self) -> String {
		self.id.clone()
	}

	#[inline]
	fn author(&self) -> String {
		self.author.clone()
	}

	#[inline]
	fn email(&self) -> String {
		self.email.clone()
	}

	#[inline]
	fn message(&self) -> String {
		self.message.clone()
	}

	#[inline]
	fn timestamp(&self) -> i64 {
		self.timestamp
	}
}

/// The iterator type for [`MemoryWorkspace`].
pub struct MemoryIterator<'a>(MutexGuard<'a, State>, Option<MemoryRecord>);

impl<'a> Iterator for MemoryIterator<'a> {
	type Item = Result<MemoryRecord>;

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		let (record, next) = {
			let state = &mut self.0;
			let next = self.1.as_ref().and_then(|record| {
				record
					.parent
					.as_ref()
					.and_then(|parent| state.records.get(parent).cloned())
			});
			(self.1.take(), next)
		};
		self.1 = next;
		record.map(Ok)
	}
}

/// The set iterator type for [`MemoryWorkspace`].
pub struct MemorySetIterator<'a>(MemoryIterator<'a>);

impl<'a> Iterator for MemorySetIterator<'a> {
	type Item = Result<(MemoryRecord, SetOperation)>;

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		let record = match self.0.next() {
			Some(Ok(record)) => record,
			Some(Err(e)) => return Some(Err(e)),
			None => return None,
		};

		if let Some(op) = record.op {
			Some(Ok((record, op)))
		} else {
			Some(Err(Error::Malformed(format!(
				"record {} is not a set operation",
				record.id
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
	type Record<'b> = MemoryRecord where Self: 'b;

	fn upsert_attachment<D: AsRef<[u8]>>(&mut self, name: &str, data: D) -> Result<()> {
		let data = data.as_ref().to_vec();
		let id = self.workspace.insert_attachment(data);
		self.attachments.insert(name.to_string(), Some(id.clone()));
		Ok(())
	}

	fn remove_attachment(&mut self, name: &str) -> Result<()> {
		self.attachments.insert(name.to_string(), None);
		Ok(())
	}

	fn commit(self, message: &str) -> Result<Self::Record<'a>> {
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

		Ok(record)
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
