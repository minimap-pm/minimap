//! Implmentation of the [`Workspace`] trait for Git repositories.
//!
//! This allows Minimap to be used with a remote git repository
//! as a backend. Reads hit the local repository, and writes
//! are immediately pushed to the workspace.

use crate::{Error, Record, RecordBuilder, Remote, Result, SetOperation};
use git2::{
	build::{RepoBuilder, TreeUpdateBuilder},
	AutotagOption, Commit, Cred, FetchOptions, FetchPrune, ObjectType, Oid, PushOptions,
	RemoteCallbacks, Repository, Revwalk,
};
use std::{
	cell::RefCell,
	hash::{Hash, Hasher},
	path::{Path, PathBuf},
};

/// An iterator over the commits in a [`GitRemote`].
pub struct GitIterator<'a>(&'a GitRemote, Revwalk<'a>);

/// A remote git repository.
pub struct GitRemote {
	repo: Repository,
	set_add_oid: Oid,
	set_del_oid: Oid,
}

impl GitRemote {
	/// Opens a remote repository. If the repository hasn't been cloned yet,
	/// Minimap will attempt to clone it from the remote prior to returning.
	pub fn open(remote: &str) -> Result<Self> {
		let local_dir = generate_tmp_dir(remote)?;

		// Try to open it as a local repository first,
		// and if that fails, clone it from the remote.
		let repo = if let Ok(repo) = Repository::open(&local_dir) {
			repo
		} else {
			let mut callbacks = RemoteCallbacks::new();
			callbacks.credentials(|_url, username_from_url, _allowed_types| {
				Cred::ssh_key(
					username_from_url.unwrap(),
					None,
					Path::new(&format!(
						"{}/.ssh/id_rsa",
						std::env::var("HOME").expect("HOME environment variable not set")
					)),
					None,
				)
			});

			let mut fetch_opts = FetchOptions::new();
			fetch_opts.update_fetchhead(false);
			fetch_opts.download_tags(AutotagOption::All);
			fetch_opts.prune(FetchPrune::On);
			fetch_opts.remote_callbacks(callbacks);

			RepoBuilder::new()
				.bare(true)
				.fetch_options(fetch_opts)
				.clone(remote, &local_dir)?
		};

		// The set_add_oid/ set_del_oid are the OIDs of two
		// empty commits tagged with `meta/+` and `meta/-`,
		// respectively. These commits are used as parents
		// to commits made within a set to determine the
		// operation that was performed, and are created
		// and tagged if they don't exist here. We hold a
		// reference to them for the lifetime of the workspace
		// as an optimization. It's important that we create,
		// tag, and push these commits upon opening if they
		// don't exist.
		let mut needs_push = false;
		let set_add_oid = {
			let (oid, created) = Self::upsert_operator_tag(&repo, "meta/+")?;
			needs_push = needs_push || created;
			oid
		};
		let set_del_oid = {
			let (oid, created) = Self::upsert_operator_tag(&repo, "meta/-")?;
			needs_push = needs_push || created;
			oid
		};

		if needs_push {
			let mut remote = repo.find_remote("origin")?;
			let mut callbacks = RemoteCallbacks::new();

			callbacks.credentials(|_url, username_from_url, _allowed_types| {
				Cred::ssh_key(
					username_from_url.unwrap(),
					None,
					Path::new(&format!(
						"{}/.ssh/id_rsa",
						std::env::var("HOME").expect("HOME environment variable not set")
					)),
					None,
				)
			});

			remote.push(
				&["refs/tags/meta/+", "refs/tags/meta/-"],
				Some(PushOptions::new().remote_callbacks(callbacks)),
			)?;
		}

		Ok(Self {
			repo,
			set_add_oid,
			set_del_oid,
		})
	}

	/// Gets the OID of an operator tag (e.g. `refs/tags/meta/+`)
	/// or creates it if it doesn't exist. Returns the [`git2::Oid`]
	/// and a boolean for whether or not the tag had to be created.
	fn upsert_operator_tag(repo: &Repository, name: &str) -> Result<(Oid, bool)> {
		let tag = repo.find_reference(&format!("refs/tags/{}", name));
		if let Ok(tag) = tag {
			return Ok((tag.target().unwrap(), false));
		}

		let commit_oid = repo.commit(
			None,
			&repo.signature()?,
			&repo.signature()?,
			name,
			&repo.find_tree(repo.treebuilder(None)?.write()?)?,
			&[],
		)?;
		let commit = repo.find_object(commit_oid, Some(ObjectType::Commit))?;

		repo.tag_lightweight(name, &commit, false)?;
		Ok((commit_oid, true))
	}
}

/// A singular git record (a wrapper around a [`git2::Commit`]).
#[derive(Clone)]
pub struct GitRecord<'a>(&'a GitRemote, Commit<'a>);

impl<'a> Hash for GitRecord<'a> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.1.id().hash(state);
	}
}

impl<'a> PartialEq for GitRecord<'a> {
	#[inline]
	fn eq(&self, other: &Self) -> bool {
		self.1.id() == other.1.id()
	}
}

impl<'a> std::fmt::Debug for GitRecord<'a> {
	#[inline]
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		// just forward to the commit
		self.1.fmt(f)
	}
}

impl<'a> Eq for GitRecord<'a> {}

impl<'a> Remote<'a> for GitRemote {
	type Record = GitRecord<'a>;
	type RecordBuilder = GitRecordBuilder<'a>;
	type Iterator = GitIterator<'a>;
	type SetIterator = GitSetIterator<'a>;

	fn record_builder(&'a self, collection: &str) -> Self::RecordBuilder {
		GitRecordBuilder::new(self, collection)
	}

	fn get_record(&'a self, id: &str) -> Result<Option<Self::Record>> {
		self.repo
			.find_commit(Oid::from_str(id)?)
			.map(|c| GitRecord(self, c))
			.map(Some)
			.or_else(|e| {
				if e.code() == git2::ErrorCode::NotFound {
					Ok(None)
				} else {
					Err(e.into())
				}
			})
	}

	fn walk(&'a self, collection: &str) -> Result<Self::Iterator> {
		match self
			.repo
			.revparse_single(&format!("refs/heads/{collection}"))
		{
			Ok(head) => {
				let mut walk = self.repo.revwalk()?;
				walk.push(head.id())?;
				Ok(GitIterator(self, walk))
			}
			Err(e) if e.code() == git2::ErrorCode::NotFound => {
				Ok(GitIterator(self, self.repo.revwalk()?))
			}
			Err(e) => Err(e.into()),
		}
	}

	fn set_add_unchecked(&'a self, collection: &str, message: &str) -> Result<Self::Record> {
		let mut b = self.record_builder(collection);
		b.add_parent(self.set_add_oid);
		b.commit(message)
	}

	fn set_del_unchecked(&'a self, collection: &str, message: &str) -> Result<Self::Record> {
		let mut b = self.record_builder(collection);
		b.add_parent(self.set_del_oid);
		b.commit(message)
	}

	fn walk_set(&'a self, collection: &str) -> Result<Self::SetIterator> {
		Ok(GitSetIterator(self.walk(collection)?))
	}
}

/// An iterator over a set of records in a collection. The iterator returns
/// both the record itself and the operation that was performed on it.
pub struct GitSetIterator<'a>(GitIterator<'a>);

impl<'a> Iterator for GitSetIterator<'a> {
	type Item = Result<(GitRecord<'a>, SetOperation)>;

	fn next(&mut self) -> Option<Self::Item> {
		while let Some(commit) = self.0.next() {
			let commit = match commit {
				Ok(commit) => commit,
				Err(e) => return Some(Err(e)),
			};

			if commit.1.id() == self.0.0.set_add_oid || commit.1.id() == self.0.0.set_del_oid {
				continue;
			}

			if !matches!(commit.1.parent_count(), 1 | 2) {
				return Some(Err(Error::Malformed(format!(
					"commit {} has {} parents, expected 2",
					commit.id(),
					commit.1.parent_count()
				))));
			}

			let op = commit
				.1
				.parents()
				.find(|p| p.id() == self.0.0.set_add_oid || p.id() == self.0.0.set_del_oid)
				.map(|p| {
					if p.id() == self.0.0.set_add_oid {
						SetOperation::Add
					} else {
						SetOperation::Del
					}
				});

			return Some(
				op.ok_or_else(|| {
					Error::Malformed(format!(
						"commit {} is missing an operator tag parent",
						commit.id()
					))
				})
				.map(|op| (commit, op)),
			);
		}

		None
	}
}

impl<'a> Iterator for GitIterator<'a> {
	type Item = Result<GitRecord<'a>>;

	fn next(&mut self) -> Option<Self::Item> {
		self.1.next().map(|id| {
			self.0
				.repo
				.find_commit(id?)
				.map(|c| GitRecord(self.0, c))
				.map_err(Into::into)
		})
	}
}

impl<'b> Record for GitRecord<'b> {
	fn id(&self) -> String {
		self.1.id().to_string()
	}

	fn author(&self) -> String {
		self.1
			.author()
			.name()
			.map(|s| s.to_string())
			.unwrap_or_else(|| String::from_utf8_lossy(self.1.author().name_bytes()).to_string())
	}

	fn email(&self) -> String {
		self.1
			.author()
			.email()
			.map(|s| s.to_string())
			.unwrap_or_else(|| String::from_utf8_lossy(self.1.author().email_bytes()).to_string())
	}

	fn message(&self) -> String {
		self.1
			.message()
			.map(|s| s.to_string())
			.unwrap_or_else(|| String::from_utf8_lossy(self.1.message_bytes()).to_string())
	}

	fn timestamp(&self) -> i64 {
		self.1.time().seconds()
	}

	fn attachment(&self, path: &str) -> Result<Option<Vec<u8>>> {
		let tree = self.1.tree()?;
		let entry = tree.get_path(Path::new(path))?;
		let blob = self.0.repo.find_blob(entry.id())?;
		Ok(Some(blob.content().to_vec()))
	}
}

/// Builds a commit (with attachments) in order to submit it to a [`GitRemote`].
pub struct GitRecordBuilder<'a> {
	workspace: &'a GitRemote,
	branch: String,
	update: TreeUpdateBuilder,
	additional_parents: Vec<Oid>,
}

impl<'a> GitRecordBuilder<'a> {
	#[inline]
	fn new(workspace: &'a GitRemote, branch: &str) -> Self {
		Self {
			workspace,
			branch: branch.to_string(),
			update: TreeUpdateBuilder::new(),
			additional_parents: Vec::new(),
		}
	}

	#[inline]
	fn add_parent(&mut self, parent: Oid) {
		self.additional_parents.push(parent);
	}
}

impl<'a> RecordBuilder<'a> for GitRecordBuilder<'a> {
	type Record = GitRecord<'a>;

	fn upsert_attachment<D: AsRef<[u8]>>(mut self, path: &str, data: D) -> Result<Self> {
		self.update.upsert(
			path,
			self.workspace.repo.blob(data.as_ref())?,
			git2::FileMode::Blob,
		);
		Ok(self)
	}

	fn remove_attachment(mut self, path: &str) -> Result<Self> {
		self.update.remove(path);
		Ok(self)
	}

	fn commit(self, message: &str) -> Result<Self::Record> {
		let ref_head = format!("refs/heads/{}", self.branch);

		let head = self
			.workspace
			.repo
			.revparse_single(&ref_head)
			.and_then(|h| h.peel_to_commit())
			.ok();

		// Get the tree of the head commit, or create a new one if there's no head.
		let base_tree = head.clone().map(|h| h.tree()).unwrap_or_else(|| {
			self.workspace
				.repo
				.find_tree(self.workspace.repo.treebuilder(None)?.write()?)
		})?;

		let mut update = self.update;
		let tree_oid = update.create_updated(&self.workspace.repo, &base_tree)?;
		let tree = self.workspace.repo.find_tree(tree_oid)?;

		let sig = self.workspace.repo.signature()?;

		let mut parents = head.map(|h| vec![h]).unwrap_or_default();
		for additional_parent in self.additional_parents {
			let parent = self.workspace.repo.find_commit(additional_parent)?;
			parents.push(parent);
		}

		let parent_refs = parents.iter().collect::<Vec<_>>();

		let commit = self
			.workspace
			.repo
			.commit(None, &sig, &sig, message, &tree, &parent_refs)?;

		// Now push the commit to the remote. We don't update the local ref
		// yet until the push succeeds. Yes, this creates a bit of a race condition,
		// but the more error-prone operation is the push, whereas the local ref update
		// is trivial and only fails if there's some sort of disk I/O failure, or if something
		// else is modifies the repository at the same time.
		let mut remote = self.workspace.repo.find_remote("origin")?;
		let pushed_status = RefCell::new(None);
		let mut callbacks = RemoteCallbacks::new();

		callbacks.credentials(|_url, username_from_url, _allowed_types| {
			Cred::ssh_key(
				username_from_url.unwrap(),
				None,
				Path::new(&format!(
					"{}/.ssh/id_rsa",
					std::env::var("HOME").expect("HOME environment variable not set")
				)),
				None,
			)
		});

		callbacks.push_update_reference(|refname, status| {
			if refname == ref_head {
				pushed_status
					.borrow_mut()
					.replace(status.map(|s| s.to_string()));
			}
			Ok(())
		});

		remote.push(
			&[format!("{commit}:{ref_head}")],
			Some(PushOptions::new().remote_callbacks(callbacks)),
		)?;

		match pushed_status.take() {
			None => Err(Error::NotPushed(self.branch)),
			Some(Some(status)) => Err(Error::PushFailed(self.branch, status)),
			Some(None) => {
				// Finally update the branch's ref to the newly created commit
				// in our local repository.
				self.workspace.repo.reference(
					&ref_head,
					commit,
					true,
					&format!("commit: {commit}"),
				)?;

				let commit = self.workspace.repo.find_commit(commit)?;
				Ok(GitRecord(self.workspace, commit))
			}
		}
	}
}

/// Generates the temporary directory for a given remote
/// by first hashing the remote and using that as a subfolder
/// in the standard temporary directory joined with the
/// "minimap" subfolder (e.g. if the system tmp directory
/// is "/tmp" and the remote hash is "12345", the resulting
/// path will be "/tmp/minimap/12345").
pub(crate) fn generate_tmp_dir(remote: &str) -> Result<PathBuf> {
	use ::sha2::Digest;

	let mut hasher = ::sha2::Sha256::new();
	hasher.update(remote.as_bytes());
	let hash = hasher.finalize();
	let hash = format!("{:x}", hash);
	let mut path = ::std::env::temp_dir();
	path.push("minimap");
	path.push(hash);
	::std::fs::create_dir_all(&path)?;
	Ok(path)
}

#[cfg(test)]
mod test {
	use super::*;

	macro_rules! function {
		() => {{
			fn f() {}
			fn type_name_of<T>(_: T) -> &'static str {
				std::any::type_name::<T>()
			}
			let name = type_name_of(f);
			name.strip_suffix("::f").unwrap()
		}};
	}

	fn get_remote_uri(test_name: String) -> (PathBuf, String) {
		let mut path = ::std::env::temp_dir();
		path.push("minimap-test");
		path.push(test_name);
		let uri = format!("file://{}", path.display());
		(path, uri)
	}

	fn init_test_remote(path: &PathBuf, remote_uri: &str) -> GitRemote {
		::std::fs::remove_dir_all(path)
			.or_else(|e| {
				if e.kind() == ::std::io::ErrorKind::NotFound {
					Ok(())
				} else {
					Err(e)
				}
			})
			.unwrap();

		let tmp_path = generate_tmp_dir(remote_uri).unwrap();
		::std::fs::remove_dir_all(&tmp_path)
			.or_else(|e| {
				if e.kind() == ::std::io::ErrorKind::NotFound {
					Ok(())
				} else {
					Err(e)
				}
			})
			.unwrap();

		Repository::init_bare(path).unwrap();

		// Init the test repository and set the user.name and
		// user.email config values since there's no guarantee
		// they've been set up globally on the system that's
		// testing Minimap.
		let repo = Repository::init(&tmp_path).unwrap();
		repo.config()
			.unwrap()
			.set_str("user.name", "Test User")
			.unwrap();
		repo.config()
			.unwrap()
			.set_str("user.email", "test@example.com")
			.unwrap();

		// We also have to manually create the 'origin' remote.
		// This is normally done by the clone operation, but
		// since we're not cloning, we have to do it ourselves.
		repo.remote_set_url("origin", remote_uri).unwrap();

		GitRemote::open(remote_uri).unwrap()
	}

	fn create_test_remote(test_name: String) -> GitRemote {
		let (path, remote_uri) = get_remote_uri(test_name);
		init_test_remote(&path, &remote_uri)
	}

	macro_rules! create_test_remote {
		() => {
			create_test_remote(function!().to_string())
		};
		($suffix:literal) => {
			create_test_remote(format!("{}-{}", function!(), $suffix))
		};
	}

	include!("../acceptance-tests.inc.rs");

	#[test]
	fn test_remote_minimap_dependencies() {
		let our_workspace = Workspace::open(create_test_remote!());
		let (their_path, their_remote_uri) = get_remote_uri(format!("{}-other", function!()));
		let their_workspace = Workspace::open(init_test_remote(&their_path, &their_remote_uri));

		let our_project = our_workspace.create_project("test").unwrap().unwrap();
		let their_project = their_workspace.create_project("other").unwrap().unwrap();

		let our_ticket = our_project.create_ticket().unwrap();
		let their_ticket = their_project.create_ticket().unwrap();

		assert_eq!(our_ticket.id(), 1);
		assert_eq!(their_ticket.id(), 1);

		our_ticket
			.add_dependency(
				"minimap",
				&format!("{}@{}", their_remote_uri, their_ticket.slug()),
			)
			.unwrap();

		assert_eq!(our_ticket.dependencies().unwrap().len(), 1);

		let registry = DependencyRegistry::new();

		let mut found = false;
		for (origin, endpoint, status) in our_ticket
			.resolve_dependencies(&registry)
			.unwrap()
			.map(|d| d.unwrap())
		{
			assert!(!found);
			found = true;
			assert_eq!(origin, "minimap");
			assert_eq!(
				endpoint,
				format!("{}@{}", their_remote_uri, their_ticket.slug())
			);
			assert_eq!(status, DependencyStatus::Pending);
		}

		assert!(found);

		their_ticket.set_state(TicketState::Closed).unwrap();

		let mut found = false;
		for (origin, endpoint, status) in our_ticket
			.resolve_dependencies(&registry)
			.unwrap()
			.map(|d| d.unwrap())
		{
			assert!(!found);
			found = true;
			assert_eq!(origin, "minimap");
			assert_eq!(
				endpoint,
				format!("{}@{}", their_remote_uri, their_ticket.slug())
			);
			assert_eq!(status, DependencyStatus::Complete);
		}

		assert!(found)
	}
}

#[cfg(feature = "serde")]
impl serde::Serialize for GitRecord<'_> {
	fn serialize<S: serde::Serializer>(
		&self,
		serializer: S,
	) -> std::result::Result<S::Ok, S::Error> {
		use serde::ser::SerializeStruct;
		let mut state = serializer.serialize_struct("GitRecord", 5)?;
		state.serialize_field("id", &Record::id(self))?;
		state.serialize_field("author", &Record::author(self))?;
		state.serialize_field("email", &Record::email(self))?;
		state.serialize_field("message", &Record::message(self))?;
		state.serialize_field("timestamp", &Record::timestamp(self))?;
		state.end()
	}
}
