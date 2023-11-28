//! Minimap core functionality crate.
//!
//! This crate contains the core functionality of Minimap's
//! data modeling and Git repository interop.
//!
//! The entry point to any Minimap project is the [`Workspace`]
//! struct.
#![deny(missing_docs, unsafe_code)]

pub(crate) mod workspace;

#[cfg(feature = "git")]
pub use workspace::git::*;
pub use workspace::memory::*;

use indexmap::{IndexMap, IndexSet};
use std::hash::Hash;

/// The error type for all Minimap operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
	/// An error occurred while interacting with the Git repository.
	/// This is mostly unexpected, as Minimap tries to convert expected
	/// errors into more specific errors.
	#[error("git error: {0}")]
	Git(#[from] git2::Error),
	/// An error occured while performing some sort of I/O operation.
	#[error("io error: {0}")]
	Io(#[from] ::std::io::Error),
	/// libgit2 doesn't have straightforward "success" or "failure"
	/// codes when pushing; instead, we get a callback for each pushed
	/// ref, with a status. If the status is `None`, the push succeeded.
	/// If it's `Some(_)`, the push failed, and `PushFailed` is returned.
	/// However, if we don't even detect the ref being pushed (for some
	/// bizarre reason), we return `NotPushed` instead.
	#[error("a push operation succeeded, but a ref was not pushed: {0}")]
	NotPushed(String),
	/// See [`NotPushed`].
	#[error("failed to push ref {0}: {1}")]
	PushFailed(String, String),
	/// A fetch or commit operation couldn't find the entity on which
	/// it was meant to operate.
	#[error("entity not found: {0}/{1}")]
	NotFound(String, String),
	/// A create operation failed because the entity already exists.
	#[error("entity exists: {0}/{1}")]
	Exists(String, String),
	/// An operation that expected a specific format of data
	/// found malformed data. This indicates one of two things:
	/// either there's a bug with Minimap, or a human has
	/// manually modified the remote repository (which should
	/// not be done).
	#[error("malformed entity: {0}")]
	Malformed(String),
}

/// The result type for all Minimap operations.
pub type Result<T> = ::std::result::Result<T, Error>;

/// A Minimap workspace holds all project tickets, assets, and other data.
/// It is routinely synchronized with a local clone that Minimap manages
/// itself - thus, it is not necessary nor recommended to manually clone
/// workspaces yourself.
///
/// Workspaces work within the context of a user, which is already established
/// at te time Workspace is created. This should include a name and email address.
#[allow(clippy::type_complexity)]
pub trait Workspace: Sized {
	/// The type of record that this workspace produces.
	type Record<'a>: Record
	where
		Self: 'a;
	/// The type of record builder that this workspace produces.
	type RecordBuilder<'a>: RecordBuilder<'a, Record<'a> = Self::Record<'a>>
	where
		Self: 'a;
	/// Iterates over records in a collection in **reverse** order from latest
	/// to oldest created. Note that this isn't necessarily a timestamp
	/// ordering, and may yield results in a different order than expected
	/// (especially in the case of e.g. Git, which orders based on parent/child
	/// relationships).
	type Iterator<'a>: Iterator<Item = Result<Self::Record<'a>>>
	where
		Self: 'a;
	/// Iterates over a set of records in a collection in order of creation,
	/// returning both the record itself and the operation that was performed on it.
	type SetIterator<'a>: Iterator<Item = Result<(Self::Record<'a>, SetOperation)>>
	where
		Self: 'a;

	/// Get an iterator over all records in the collection, in order from first to last.
	fn walk<'a>(&'a self, collection: &str) -> Result<Self::Iterator<'a>>;

	/// Creates a new record builder that is used to submit a record to the workspace.
	fn record_builder<'a>(&'a self, collection: &str) -> Self::RecordBuilder<'a>;

	/// Returns a record based on its ID.
	fn get_record<'a>(&'a self, id: &str) -> Result<Option<Self::Record<'a>>>;

	/// Returns the latest record in the collection.
	fn latest<'a>(&'a self, collection: &str) -> Result<Option<Self::Record<'a>>> {
		self.walk(collection)?.next().transpose()
	}

	/// Adds an item to a set. Does not check if the item already exists.
	fn set_add_unchecked<'a>(&'a self, collection: &str, message: &str)
	-> Result<Self::Record<'a>>;

	/// Removes an item from a set. Does not check if the item already exists.
	fn set_del_unchecked<'a>(&'a self, collection: &str, message: &str)
	-> Result<Self::Record<'a>>;

	/// Get an iterator over a set of records in a collection, in order of creation.
	/// The iterator returns both the record itself and the operation that was performed on it.
	fn walk_set<'a>(&'a self, collection: &str) -> Result<Self::SetIterator<'a>>;

	/// Gets an item in a set. After unwrapping the outer `Result`,
	/// `Ok(record)` indicates the item exists, `Err(Some(record))`
	/// indicates the item does not exist and `record` is the record
	/// from when the item was removed, and `Err(None)` indicates the
	/// item does not exist and there is no record from when the item
	/// was removed.
	fn set_find<'a>(
		&'a self,
		collection: &str,
		message: &str,
	) -> Result<::std::result::Result<Self::Record<'a>, Option<Self::Record<'a>>>> {
		for result in self.walk_set(collection)? {
			let (record, op) = result?;
			if record.message() == message {
				return Ok(match op {
					SetOperation::Add => Ok(record),
					SetOperation::Del => Err(Some(record)),
				});
			}
		}
		Ok(Err(None))
	}

	/// Adds an item to a set. If the item already exists, returns the
	/// existing item as an `Err` value. Otherwise, returns a tuple of
	/// `(added_record, Option<removed_record>)`, where `added_record` is
	/// the new record adding the item, and `removed_record` is the record
	/// from when the item was removed (or `None` if the item did not exist).
	/// The outer `Result` is an error if some operational error occurred.
	fn set_add<'a>(
		&'a self,
		collection: &str,
		message: &str,
	) -> Result<::std::result::Result<(Self::Record<'a>, Option<Self::Record<'a>>), Self::Record<'a>>>
	{
		match self.set_find(collection, message)? {
			Ok(record) => Ok(Err(record)),
			Err(record) => Ok(Ok((self.set_add_unchecked(collection, message)?, record))),
		}
	}

	/// Removes an item from a set. If the item existed, returns
	/// `Some((removed_record, added_record))`, where `removed_record` is
	/// the new record removing the item, and `added_record` is the record
	/// from when the item was added. If the item did not exist, returns
	/// `Option(removed_record)`. The outer `Result` is an error if some
	/// operational error occurred.
	fn set_del<'a>(
		&'a self,
		collection: &str,
		message: &str,
	) -> Result<::std::result::Result<(Self::Record<'a>, Self::Record<'a>), Option<Self::Record<'a>>>>
	{
		match self.set_find(collection, message)? {
			Ok(record) => Ok(Ok((self.set_del_unchecked(collection, message)?, record))),
			Err(record) => Ok(Err(record)),
		}
	}

	/// Gets all items in a set.
	fn set_get_all<'a>(&'a self, collection: &str) -> Result<IndexSet<Self::Record<'a>>> {
		// Since we walk backwards in time, deletions are held as gravestones (`None`)
		// in a map, which are removed when an addition is found. If a value is in the map
		// already, then the iteration is ignored.
		let mut map = IndexMap::new();

		for result in self.walk_set(collection)? {
			let (record, op) = result?;
			match op {
				SetOperation::Add => match map.get(&record.message()) {
					Some(None) => {
						map.remove(&record.message());
					}
					Some(Some(_)) => {}
					None => {
						map.insert(record.message(), Some(record));
					}
				},
				SetOperation::Del => {
					let message = record.message();
					if map.get(&message).is_none() {
						map.insert(message, None);
					}
				}
			}
		}

		let mut r = IndexSet::new();

		for v in map.into_iter().rev().filter_map(|t| t.1) {
			r.insert(v);
		}

		Ok(r)
	}

	/// Gets the name of the workspace
	fn name(&self) -> Result<Option<Self::Record<'_>>> {
		self.walk("meta/workspace/name")?.next().transpose()
	}

	/// Sets the name of the workspace
	fn set_name<'a>(&'a self, name: &str) -> Result<Self::Record<'a>> {
		self.record_builder("meta/workspace/name").commit(name)
	}

	/// Gets the description of the workspace
	fn description(&self) -> Result<Option<Self::Record<'_>>> {
		self.walk("meta/workspace/description")?.next().transpose()
	}

	/// Sets the description of the workspace
	fn set_description<'a>(&'a self, description: &str) -> Result<Self::Record<'a>> {
		self.record_builder("meta/workspace/description")
			.commit(description)
	}

	/// Returns a project given its slug.
	fn project<'a>(&'a self, slug: &str) -> Result<Project<'a, Self>> {
		self.set_find("meta/projects", slug)?
			.map_err(|_| Error::NotFound("meta/projects".to_string(), slug.to_string()))?;

		Ok(Project {
			workspace: self,
			slug: slug.to_string(),
			meta_path: format!("meta/project/{}", slug),
			path: format!("project/{}", slug),
		})
	}

	/// Creates a project with the given slug.
	/// If the project already exists, returns `Ok(Err(record))` with the
	/// set record of the existing project.
	fn create_project<'a>(
		&'a self,
		slug: &str,
	) -> Result<::std::result::Result<Project<'a, Self>, Self::Record<'a>>> {
		self.set_add("meta/projects", slug)
			.map(|result| match result {
				Ok(_) => Ok(Project {
					workspace: self,
					slug: slug.to_string(),
					meta_path: format!("meta/project/{}", slug),
					path: format!("project/{}", slug),
				}),
				Err(record) => Err(record),
			})
	}

	/// Gets a ticket by its slug.
	/// Returns [`Error::NotFound`] if either the project or ticket do not exist.
	fn ticket<'a>(&'a self, slug: &str) -> Result<Ticket<'a, Self>> {
		let (project_slug, ticket_id) = slug
			.rsplit_once('-')
			.ok_or_else(|| Error::Malformed(slug.to_string()))?;

		let project = self.project(project_slug)?;

		let ticket_id = ticket_id
			.parse::<u64>()
			.map_err(|_| Error::Malformed(slug.to_string()))?;

		project.ticket(ticket_id)
	}
}

/// A single record from a collection.
pub trait Record: Clone + Sized + Hash + PartialEq + Eq {
	/// Gets the globally unique identifier for the record.
	fn id(&self) -> String;
	/// Gets the name of the author of the record.
	fn author(&self) -> String;
	/// Gets the email address of the author of the record.
	fn email(&self) -> String;
	/// The message of the record. Must be character-for-character identical
	/// to the message that was original created.
	fn message(&self) -> String;
	/// Gets the unix timestamp of the record.
	fn timestamp(&self) -> i64;
}

/// Builds a record (with attachments) in order to submit a
/// record to a remote collection
pub trait RecordBuilder<'a> {
	/// The type of record that this record builder produces.
	type Record<'b>: Record
	where
		Self: 'b;

	/// Builds the record and submits it to the remote.
	fn commit(self, message: &str) -> Result<Self::Record<'a>>;

	/// Upserts an attachment to the record.
	fn upsert_attachment<D: AsRef<[u8]>>(&mut self, name: &str, data: D) -> Result<()>;

	/// Removes an attachment from the collection entirely upon record.
	/// Future records will not contain this attachment.
	fn remove_attachment(&mut self, name: &str) -> Result<()>;
}

/// The type of operation performed on a record in a set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetOperation {
	/// A record was added to the set.
	Add,
	/// A record was deleted from the set.
	Del,
}

/// A Minimap project. Projects are a collection of tickets,
/// which are a collection of comments, attachments, and other
/// such resources.
pub struct Project<'a, W: Workspace> {
	workspace: &'a W,
	slug: String,
	meta_path: String,
	path: String,
}

impl<'a, W: Workspace> Project<'a, W>
where
	W: Workspace,
{
	/// Gets the slug of the project.
	#[inline]
	pub fn slug(&self) -> &str {
		&self.slug
	}

	/// Gets the name of the workspace.
	pub fn name(&self) -> Result<Option<W::Record<'a>>> {
		self.workspace
			.walk(&format!("{}/name", self.meta_path))?
			.next()
			.transpose()
	}

	/// Sets the name of the workspace.
	pub fn set_name(&self, name: &str) -> Result<W::Record<'a>> {
		self.workspace
			.record_builder(&format!("{}/name", self.meta_path))
			.commit(name)
	}

	/// Gets the description of the workspace.
	pub fn description(&self) -> Result<Option<W::Record<'a>>> {
		self.workspace
			.walk(&format!("{}/description", self.meta_path))?
			.next()
			.transpose()
	}

	/// Sets the description of the project.
	pub fn set_description(&self, description: &str) -> Result<W::Record<'a>> {
		self.workspace
			.record_builder(&format!("{}/description", self.meta_path))
			.commit(description)
	}

	/// Creates a ticket in the project.
	pub fn create_ticket(&self) -> Result<Ticket<'a, W>> {
		// First, get a new ticket ID by incrementing the ticket counter.
		// The ticket counter is stored in the meta/project/<slug>/ticket_counter
		// collection, and is the head record with a single integer value.
		// If the collection doesn't exist, the counter starts at 1.
		// The ticket counter is not a set, it's just a running count.
		let ticket_counter_path = format!("{}/ticket_counter", self.meta_path);
		let ticket_counter = self
			.workspace
			.walk(&ticket_counter_path)?
			.next()
			.transpose()?
			.map(|record| {
				record
					.message()
					.parse::<u64>()
					.map_err(|_| Error::Malformed(ticket_counter_path.clone()))
			})
			.transpose()?
			.unwrap_or(0);

		let ticket_id = ticket_counter + 1;
		let ticket_slug = format!("{}-{}", self.slug, ticket_id);

		// First, we try to increment the ID. The worst case here is that we have a skipped ticket
		// count if the tickets set add fails, which is fine - because in the inverse cass (where
		// we increment after we add to the set, but the increment fails), the next time a ticket
		// is created we'll get a malformed collection error.
		self.workspace
			.record_builder(&ticket_counter_path)
			.commit(&ticket_id.to_string())?;

		// Now, create the ticket in the project/tickets set.
		self.workspace
			.set_add(&format!("{}/tickets", self.path), &ticket_id.to_string())?
			.map_err(|_| Error::Malformed(format!("{}/tickets", self.path)))?;

		Ok(Ticket {
			workspace: self.workspace,
			slug: ticket_slug,
			id: ticket_id,
			path: format!("{}/ticket/{}", self.path, ticket_id),
		})
	}

	/// Gets a ticket by its ID.
	pub fn ticket(&self, id: u64) -> Result<Ticket<'a, W>> {
		// First, check if the ticket exists.
		self.workspace
			.set_find(&format!("{}/tickets", self.path), &id.to_string())?
			.map_err(|_| Error::NotFound(format!("{}/tickets", self.path), id.to_string()))?;

		Ok(Ticket {
			workspace: self.workspace,
			slug: format!("{}-{}", self.slug, id),
			id,
			path: format!("{}/ticket/{}", self.path, id),
		})
	}
}

/// A Minimap ticket. Tickets are a collection of comments,
/// attachments, and other such resources, and belong to a
/// project.
pub struct Ticket<'a, W: Workspace> {
	workspace: &'a W,
	slug: String,
	id: u64,
	path: String,
}

impl<'a, W: Workspace> Ticket<'a, W> {
	/// Gets the slug of the ticket.
	pub fn slug(&self) -> &str {
		&self.slug
	}

	/// Gets the ticket's ID.
	pub fn id(&self) -> u64 {
		self.id
	}

	/// Gets the title of the ticket.
	pub fn title(&self) -> Result<Option<W::Record<'a>>> {
		self.workspace
			.walk(&format!("{}/title", self.path))?
			.next()
			.transpose()
	}

	/// Sets the title of the ticket.
	pub fn set_title(&self, name: &str) -> Result<W::Record<'a>> {
		self.workspace
			.record_builder(&format!("{}/title", self.path))
			.commit(name)
	}
}
