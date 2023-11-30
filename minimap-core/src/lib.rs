//! Minimap core functionality crate.
//!
//! This crate contains the core functionality of Minimap's
//! data modeling and Git repository interop.
//!
//! The entry point to any Minimap project is the [`Workspace`]
//! struct.
#![deny(missing_docs, unsafe_code)]

pub(crate) mod deps;
pub(crate) mod remote;

pub use deps::*;
#[cfg(feature = "git")]
pub use remote::git::*;
pub use remote::memory::*;

use indexmap::{IndexMap, IndexSet};
use std::{collections::HashSet, hash::Hash, marker::PhantomData};

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
	/// A dependency origin slug is malformed
	#[error("malformed dependency origin slug: {0}")]
	MalformedOrigin(String),
	/// The origin slug does not exist.
	#[error("unknown dependency origin: {0}")]
	UnknownOrigin(String),
	/// An error occurred interacting with an origin.
	#[error("dependency origin error: {0}")]
	Origin(Box<dyn std::error::Error>),
	/// The endpoint is malformed
	#[error("malformed dependency endpoint: {0}")]
	MalformedEndpoint(String),
	/// The project slug is malformed
	#[error("malformed project slug: {0}")]
	MalformedProjectSlug(String),
}

/// The result type for all Minimap operations.
pub type Result<T> = ::std::result::Result<T, Error>;

/// Minimap remotes are implementations of datastores,
/// each implementing primitive operations on collections
/// of records.
pub trait Remote<'a>: Sized
where
	Self: 'a,
{
	/// The type of record that this workspace produces.
	type Record: Record;
	/// The type of record builder that this workspace produces.
	type RecordBuilder: RecordBuilder<'a, Record = Self::Record>;
	/// Iterates over records in a collection in **reverse** order from latest
	/// to oldest created. Note that this isn't necessarily a timestamp
	/// ordering, and may yield results in a different order than expected
	/// (especially in the case of e.g. Git, which orders based on parent/child
	/// relationships).
	type Iterator: Iterator<Item = Result<Self::Record>>;
	/// Iterates over a set of records in a collection in order of creation,
	/// returning both the record itself and the operation that was performed on it.
	type SetIterator: Iterator<Item = Result<(Self::Record, SetOperation)>>;

	/// Get an iterator over all records in the collection, in order from first to last.
	fn walk(&'a self, collection: &str) -> Result<Self::Iterator>;

	/// Creates a new record builder that is used to submit a record to the workspace.
	fn record_builder(&'a self, collection: &str) -> Self::RecordBuilder;

	/// Returns a record based on its ID.
	fn get_record(&'a self, id: &str) -> Result<Option<Self::Record>>;

	/// Returns the latest record in the collection.
	#[inline]
	fn latest(&'a self, collection: &str) -> Result<Option<Self::Record>> {
		self.walk(collection)?.next().transpose()
	}

	/// Adds an item to a set. Does not check if the item already exists.
	fn set_add_unchecked(&'a self, collection: &str, message: &str) -> Result<Self::Record>;

	/// Removes an item from a set. Does not check if the item already exists.
	fn set_del_unchecked(&'a self, collection: &str, message: &str) -> Result<Self::Record>;

	/// Get an iterator over a set of records in a collection, in order of creation.
	/// The iterator returns both the record itself and the operation that was performed on it.
	fn walk_set(&'a self, collection: &str) -> Result<Self::SetIterator>;

	/// Gets an item in a set. After unwrapping the outer `Result`,
	/// `Ok(record)` indicates the item exists, `Err(Some(record))`
	/// indicates the item does not exist and `record` is the record
	/// from when the item was removed, and `Err(None)` indicates the
	/// item does not exist and there is no record from when the item
	/// was removed.
	fn set_find(
		&'a self,
		collection: &str,
		message: &str,
	) -> Result<::std::result::Result<Self::Record, Option<Self::Record>>> {
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
	#[allow(clippy::type_complexity)]
	fn set_add(
		&'a self,
		collection: &str,
		message: &str,
	) -> Result<::std::result::Result<(Self::Record, Option<Self::Record>), Self::Record>> {
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
	#[allow(clippy::type_complexity)]
	fn set_del(
		&'a self,
		collection: &str,
		message: &str,
	) -> Result<::std::result::Result<(Self::Record, Self::Record), Option<Self::Record>>> {
		match self.set_find(collection, message)? {
			Ok(record) => Ok(Ok((self.set_del_unchecked(collection, message)?, record))),
			Err(record) => Ok(Err(record)),
		}
	}

	/// Gets all items in a set in order from oldest to latest.
	fn set_get_all(&'a self, collection: &str) -> Result<IndexSet<Self::Record>> {
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

	/// Gets all items in a set in order from latest to oldest.
	fn set_get_all_reverse(&'a self, collection: &str) -> Result<IndexSet<Self::Record>> {
		let mut set = IndexSet::new();
		for item in self.walk_set_present(collection)? {
			set.insert(item?);
		}
		Ok(set)
	}

	/// Returns a set iterator that yields only present records in the set.
	/// This is different from `walk_set()` in that it does not return
	/// records that have been deleted, doesn't return the operation,
	/// and doesn't return a set item more than once (e.g. in the case
	/// the item was added, removed, and then re-added).
	fn walk_set_present(&'a self, collection: &str) -> Result<SetWalkIterator<'a, Self>> {
		Ok(SetWalkIterator {
			inner: self.walk_set(collection)?,
			map: HashSet::new(),
		})
	}
}

/// An iterator over set items, yielding only items that are present in the set.
/// Does not return the same set item more than once.
pub struct SetWalkIterator<'a, R: Remote<'a>> {
	inner: R::SetIterator,
	map: HashSet<String>,
}

impl<'a, R: Remote<'a>> Iterator for SetWalkIterator<'a, R> {
	type Item = Result<R::Record>;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			let (record, op) = match self.inner.next() {
				Some(result) => match result {
					Ok(result) => result,
					Err(err) => return Some(Err(err)),
				},
				None => return None,
			};

			let message = record.message();

			if self.map.contains(&message) {
				continue;
			}

			self.map.insert(message.clone());

			if op == SetOperation::Add {
				return Some(Ok(record));
			}
		}
	}
}

/// A Minimap workspace holds all project tickets, assets, and other data.
/// It is routinely synchronized with a local clone that Minimap manages
/// itself - thus, it is not necessary nor recommended to manually clone
/// workspaces yourself.
///
/// Workspaces work within the context of a user, which is already established
/// at te time Workspace is created. This should include a name and email address.
pub struct Workspace<'a, R: Remote<'a>>
where
	Self: 'a,
{
	remote: R,
	_phantom: PhantomData<&'a ()>,
}

impl<'a, R: Remote<'a>> Workspace<'a, R>
where
	Self: 'a,
{
	/// Opens a workspace given the remote.
	pub fn open(remote: R) -> Self {
		Self {
			remote,
			_phantom: PhantomData,
		}
	}

	/// Returns a reference to the remote.
	#[inline]
	pub fn remote(&'a self) -> &'a R {
		&self.remote
	}

	/// Gets the name of the workspace
	pub fn name(&'a self) -> Result<Option<R::Record>> {
		self.remote.latest("meta/workspace/name")
	}

	/// Sets the name of the workspace
	pub fn set_name(&'a self, name: &str) -> Result<R::Record> {
		self.remote
			.record_builder("meta/workspace/name")
			.commit(name)
	}

	/// Gets the description of the workspace
	pub fn description(&'a self) -> Result<Option<R::Record>> {
		self.remote.latest("meta/workspace/description")
	}

	/// Sets the description of the workspace
	pub fn set_description(&'a self, description: &str) -> Result<R::Record> {
		self.remote
			.record_builder("meta/workspace/description")
			.commit(description)
	}

	/// Returns a project given its slug.
	pub fn project(&'a self, slug: &str) -> Result<Project<'a, R>> {
		self.remote
			.set_find("meta/projects", slug)?
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
	///
	/// Project slugs cannot have `/` or whitespace characters.
	pub fn create_project(
		&'a self,
		slug: &str,
	) -> Result<::std::result::Result<Project<'a, R>, R::Record>> {
		validate_project_slug(slug)?;

		self.remote
			.set_add("meta/projects", slug)
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
	pub fn ticket(&'a self, slug: &str) -> Result<Ticket<'a, R>> {
		let (project_slug, ticket_id) = slug
			.rsplit_once('-')
			.ok_or_else(|| Error::Malformed(slug.to_string()))?;

		let project = self.project(project_slug)?;

		let ticket_id = ticket_id
			.parse::<u64>()
			.map_err(|_| Error::Malformed(slug.to_string()))?;

		project.ticket(ticket_id)
	}

	/// **Soft-deletes** a project given its slug.
	///
	/// **NOTE:** Re-creating a project with the same slug will
	/// **re-enable** the project, and all tickets will be restored.
	///
	/// After unwrapping the outer error, returns `Ok(record)` with the
	/// newly created set delete record, `Err(Some(record))` with the deletion
	/// record if the project was already deleted, or `Err(None)` if the project
	/// never existed.
	pub fn delete_project(
		&'a self,
		slug: &str,
	) -> Result<std::result::Result<R::Record, Option<R::Record>>> {
		self.remote
			.set_del("meta/projects", slug)
			.map(|result| match result {
				Ok((removed, _)) => Ok(removed),
				Err(record) => Err(record),
			})
	}
}

/// A single record from a collection.
pub trait Record: Clone + Sized + Hash + PartialEq + Eq + std::fmt::Debug {
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
	/// Gets an attachment by its name.
	fn attachment(&self, name: &str) -> Result<Option<Vec<u8>>>;
}

/// Builds a record (with attachments) in order to submit a
/// record to a remote collection
pub trait RecordBuilder<'a>
where
	Self: Sized,
{
	/// The type of record that this record builder produces.
	type Record: Record;

	/// Builds the record and submits it to the remote.
	fn commit(self, message: &str) -> Result<Self::Record>;

	/// Upserts an attachment to the record.
	fn upsert_attachment<D: AsRef<[u8]>>(self, name: &str, data: D) -> Result<Self>;

	/// Removes an attachment from the collection entirely upon record.
	/// Future records will not contain this attachment.
	fn remove_attachment(self, name: &str) -> Result<Self>;
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
pub struct Project<'a, R: Remote<'a>> {
	workspace: &'a Workspace<'a, R>,
	slug: String,
	meta_path: String,
	path: String,
}

fn validate_project_slug(slug: &str) -> Result<()> {
	// Slugs cannot have `/` or whitespace characters.
	if slug.contains('/') || slug.contains(char::is_whitespace) {
		return Err(Error::MalformedProjectSlug(slug.to_string()));
	}

	Ok(())
}

impl<'a, R: Remote<'a>> Project<'a, R> {
	/// Gets the slug of the project.
	#[inline]
	pub fn slug(&self) -> &str {
		&self.slug
	}

	/// Gets the name of the workspace.
	pub fn name(&self) -> Result<Option<R::Record>> {
		self.workspace
			.remote
			.latest(&format!("{}/name", self.meta_path))
	}

	/// Sets the name of the workspace.
	pub fn set_name(&self, name: &str) -> Result<R::Record> {
		self.workspace
			.remote
			.record_builder(&format!("{}/name", self.meta_path))
			.commit(name)
	}

	/// Gets the description of the workspace.
	pub fn description(&self) -> Result<Option<R::Record>> {
		self.workspace
			.remote
			.latest(&format!("{}/description", self.meta_path))
	}

	/// Sets the description of the project.
	pub fn set_description(&self, description: &str) -> Result<R::Record> {
		self.workspace
			.remote
			.record_builder(&format!("{}/description", self.meta_path))
			.commit(description)
	}

	/// Creates a ticket in the project.
	pub fn create_ticket(&self) -> Result<Ticket<'a, R>> {
		// First, get a new ticket ID by incrementing the ticket counter.
		// The ticket counter is stored in the meta/project/<slug>/ticket_counter
		// collection, and is the head record with a single integer value.
		// If the collection doesn't exist, the counter starts at 1.
		// The ticket counter is not a set, it's just a running count.
		let ticket_counter_path = format!("{}/ticket_counter", self.meta_path);
		let ticket_counter = self
			.workspace
			.remote
			.latest(&ticket_counter_path)?
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
			.remote
			.record_builder(&ticket_counter_path)
			.commit(&ticket_id.to_string())?;

		// Now, create the ticket in the project/tickets set.
		self.workspace
			.remote
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
	pub fn ticket(&self, id: u64) -> Result<Ticket<'a, R>> {
		// First, check if the ticket exists.
		self.workspace
			.remote
			.set_find(&format!("{}/tickets", self.path), &id.to_string())?
			.map_err(|_| Error::NotFound(format!("{}/tickets", self.path), id.to_string()))?;

		Ok(Ticket {
			workspace: self.workspace,
			slug: format!("{}-{}", self.slug, id),
			id,
			path: format!("{}/ticket/{}", self.path, id),
		})
	}

	/// Creates a (sub)-project with the given slug.
	/// If the project already exists, returns `Ok(Err(record))` with the
	/// set record of the existing project.
	///
	/// Project slugs cannot have `/` or whitespace characters, and despite
	/// being a sub-project (where the `self` project is the parent of the
	/// new project), the project slug must be unique to the workspace.
	pub fn create_project(
		&self,
		slug: &str,
	) -> Result<::std::result::Result<Project<'a, R>, R::Record>> {
		let project = match self.workspace.create_project(slug)? {
			Ok(project) => project,
			Err(record) => return Ok(Err(record)),
		};

		self.workspace
			.remote
			.record_builder(&format!("{}/parent", project.meta_path))
			.commit(&self.slug)?;

		Ok(Ok(project))
	}

	/// Gets the parent project of this project, or `None`
	/// if the project is a root project.
	pub fn parent(&self) -> Result<Option<Project<'a, R>>> {
		self.workspace
			.remote
			.latest(&format!("{}/parent", self.meta_path))?
			.map_or_else(
				|| Ok(None),
				|record| {
					let slug = record.message();
					Ok(Some(self.workspace.project(&slug)?))
				},
			)
	}
}

/// A Minimap ticket. Tickets are a collection of comments,
/// attachments, and other such resources, and belong to a
/// project.
pub struct Ticket<'a, R: Remote<'a>> {
	workspace: &'a Workspace<'a, R>,
	slug: String,
	id: u64,
	path: String,
}

impl<'a, R: Remote<'a>> Ticket<'a, R> {
	/// Gets the slug of the ticket.
	pub fn slug(&self) -> &str {
		&self.slug
	}

	/// Gets the ticket's ID.
	pub fn id(&self) -> u64 {
		self.id
	}

	/// Gets the title of the ticket.
	pub fn title(&self) -> Result<Option<R::Record>> {
		self.workspace
			.remote
			.latest(&format!("{}/title", self.path))
	}

	/// Sets the title of the ticket.
	pub fn set_title(&self, name: &str) -> Result<R::Record> {
		self.workspace
			.remote
			.record_builder(&format!("{}/title", self.path))
			.commit(name)
	}

	/// Gets an iterator over all comments on the ticket,
	/// in reverse order from latest to oldest.
	pub fn comments(&self) -> Result<R::Iterator> {
		self.workspace
			.remote
			.walk(&format!("{}/comment", self.path))
	}

	/// Creates a new comment on the ticket.
	pub fn add_comment(&self, comment: &str) -> Result<R::Record> {
		self.workspace
			.remote
			.record_builder(&format!("{}/comment", self.path))
			.commit(comment)
	}

	/// Creates a new attachment on the ticket.
	pub fn upsert_attachment(&self, name: &str, data: &[u8]) -> Result<R::Record> {
		self.workspace
			.remote
			.record_builder(&format!("{}/attachment", self.path))
			.upsert_attachment(name, data)?
			.commit(&format!("+{}", name))
	}

	/// Removes an attachment from the ticket.
	pub fn remote_attachment(&self, name: &str) -> Result<R::Record> {
		self.workspace
			.remote
			.record_builder(&format!("{}/attachment", self.path))
			.remove_attachment(name)?
			.commit(&format!("-{}", name))
	}

	/// Gets an attachment from the ticket.
	pub fn attachment(&self, name: &str) -> Result<Option<Vec<u8>>> {
		let record = self
			.workspace
			.remote
			.latest(&format!("{}/attachment", self.path))?;

		match record {
			Some(record) => record.attachment(name),
			None => Ok(None),
		}
	}

	/// Gets the status of the ticket. Tickets are open by default;
	/// thus if the ticket state has never been changed, the returned
	/// record is None. Otherwise, the latest state change record is
	/// returned.
	pub fn state(&self) -> Result<(TicketState, Option<R::Record>)> {
		self.workspace
			.remote
			.latest(&format!("{}/state", self.path))?
			.map_or_else(
				|| Ok((TicketState::Open, None)),
				|record| {
					let state = match record.message().as_str() {
						"open" => TicketState::Open,
						"closed" => TicketState::Closed,
						_ => return Err(Error::Malformed(format!("{}/state", self.path))),
					};
					Ok((state, Some(record)))
				},
			)
	}

	/// Sets the state of a ticket.
	pub fn set_state(&self, state: TicketState) -> Result<R::Record> {
		self.workspace
			.remote
			.record_builder(&format!("{}/state", self.path))
			.commit(match state {
				TicketState::Open => "open",
				TicketState::Closed => "closed",
			})
	}

	/// Returns if the ticket is open.
	#[inline]
	pub fn is_open(&self) -> Result<bool> {
		Ok(self.state()?.0 == TicketState::Open)
	}

	/// Returns if the ticket is closed.
	#[inline]
	pub fn is_closed(&self) -> Result<bool> {
		Ok(self.state()?.0 == TicketState::Closed)
	}

	/// Adds a dependency for the ticket.
	///
	/// Dependencies are tuples of `(origin, endpoint)`,
	/// where the origin is a slug of some external service
	/// (e.g. `minimap`, `github`, `gitlab`, etc.), and the
	/// endpoint is some unique identifier, the format of which
	/// is dependent on the origin type.
	///
	/// The special `_` origin refers to the current workspace,
	/// and can be used with a `project-#` (ticket slug) endpoint to
	/// create dependencies on other tickets in the same workspace.
	///
	/// Returns the record of the dependency addition if created,
	/// or the record of the existing dependency if it already exists.
	pub fn add_dependency(&self, origin: &str, endpoint: &str) -> Result<R::Record> {
		validate_origin(origin)?;

		self.workspace
			.remote
			.set_add(
				&format!("{}/dependencies", self.path),
				&format!("{}@{}", origin, endpoint),
			)?
			.map_or_else(Ok, |(r, _)| Ok(r))
	}

	/// Removes a dependency from the ticket.
	///
	/// See [`add_dependency`] for more information on dependencies.
	///
	/// Returns the record of the dependency removal if created,
	/// or None if the dependency did not exist.
	pub fn remove_dependency(&self, origin: &str, endpoint: &str) -> Result<Option<R::Record>> {
		validate_origin(origin)?;

		self.workspace
			.remote
			.set_del(
				&format!("{}/dependencies", self.path),
				&format!("{}@{}", origin, endpoint),
			)?
			.map_or_else(|_| Ok(None), |(r, _)| Ok(Some(r)))
	}

	/// Lists all dependencies for the ticket.
	///
	/// See [`add_dependency`] for more information on dependencies.
	pub fn dependencies(&self) -> Result<IndexSet<(String, String)>> {
		self.workspace
			.remote
			.set_get_all(&format!("{}/dependencies", self.path))?
			.into_iter()
			.map(|r| {
				let message = r.message();
				let (origin, endpoint) = message
					.split_once('@')
					.ok_or_else(|| Error::Malformed(format!("{}/dependencies", self.path)))?;
				Ok((origin.to_string(), endpoint.to_string()))
			})
			.collect()
	}

	/// Returns an iterator over all dependencies for the ticket,
	/// each iteration resolving the dependency's status.
	pub fn resolve_dependencies<D: DependencyResolver>(
		&self,
		resolver: &'a D,
	) -> Result<TicketDependencyIterator<'a, R, D>> {
		let path = format!("{}/dependencies", self.path);
		let inner = self.workspace.remote.walk_set_present(&path)?;
		Ok(TicketDependencyIterator {
			workspace: self.workspace,
			path,
			inner,
			resolver,
		})
	}
}

fn validate_origin(origin: &str) -> Result<()> {
	if origin.contains('@') {
		return Err(Error::MalformedOrigin(origin.to_string()));
	}

	Ok(())
}

/// An iterator over a ticket's dependencies that resolves
/// the status of each dependency.
pub struct TicketDependencyIterator<'a, R: Remote<'a>, D: DependencyResolver> {
	workspace: &'a Workspace<'a, R>,
	path: String,
	inner: SetWalkIterator<'a, R>,
	resolver: &'a D,
}

impl<'a, R: Remote<'a>, D: DependencyResolver> Iterator for TicketDependencyIterator<'a, R, D> {
	type Item = Result<(String, String, DependencyStatus)>;

	fn next(&mut self) -> Option<Self::Item> {
		// parse the next dependency in the set
		let message = self.inner.next()?.ok()?.message();
		let (origin, endpoint) = message
			.split_once('@')
			.ok_or_else(|| Error::Malformed(self.path.clone()))
			.ok()?;

		if origin == "_" {
			let ticket = self.workspace.ticket(endpoint).ok()?;
			let state = ticket.state().ok()?.0.into();
			return Some(Ok((origin.to_string(), endpoint.to_string(), state)));
		}

		let state = self.resolver.status(origin, endpoint).ok()?;
		Some(Ok((origin.to_string(), endpoint.to_string(), state)))
	}
}

/// The status of a ticket.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TicketState {
	/// The ticket is open.
	Open,
	/// The ticket is closed.
	Closed,
}

impl TryFrom<&str> for TicketState {
	type Error = Error;

	fn try_from(value: &str) -> Result<Self> {
		match value {
			"open" => Ok(Self::Open),
			"closed" => Ok(Self::Closed),
			_ => Err(Error::Malformed(value.to_string())),
		}
	}
}

impl TryFrom<String> for TicketState {
	type Error = Error;

	#[inline]
	fn try_from(value: String) -> Result<Self> {
		// Just forward to the &str implementation.
		Self::try_from(value.as_str())
	}
}

/// Dependency resolvers take an origin slug and endpoint string
/// and resolve the current status of the dependency.
///
/// A general purpose registry is implemented in [`DependencyRegistry`].
pub trait DependencyResolver {
	/// Resolves the status of a dependency given its origin and endpoint.
	fn status(&self, slug: &str, endpoint: &str) -> Result<DependencyStatus>;
}

/// The state of a dependency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyStatus {
	/// The dependency is still pending
	Pending,
	/// The dependency has been completed
	Complete,
}

impl From<TicketState> for DependencyStatus {
	fn from(state: TicketState) -> Self {
		match state {
			TicketState::Open => Self::Pending,
			TicketState::Closed => Self::Complete,
		}
	}
}
