//! Minimap core functionality crate.
//!
//! This crate contains the core functionality of Minimap's
//! data modeling and Git repository interop.
//!
//! The entry point to any Minimap project is the [`Remote`]
//! struct, which represents a remote Git repository managed
//! solely by Minimap.
#![deny(missing_docs, unsafe_code)]

use git2::{
	build::RepoBuilder, AutotagOption, Cred, FetchOptions, FetchPrune, PushOptions,
	RemoteCallbacks, Repository,
};
use std::{
	cell::RefCell,
	path::{Path, PathBuf},
};

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
	/// Minimap has to create a temporary directory within which
	/// [`Remote`]s are cloned. If the directory cannot be created,
	/// this error is returned.
	#[error("failed to create temporary directory: {0}: {1}")]
	TempDir(PathBuf, #[source] ::std::io::Error),
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
}

/// A Minimap remote is a git repository that holds
/// all project tickets, assets, and other data.
/// It is routinely synchronized with a local clone
/// that Minimap manages itself - thus, it is not
/// necessary nor recommended to manually clone
/// remotes yourself.
pub struct Remote {
	repo: Repository,
}

impl Remote {
	/// Opens a remote repository. If the repository hasn't been cloned yet,
	/// Minimap will attempt to clone it from the remote prior to returning.
	pub fn open(remote: &str) -> Result<Self, Error> {
		let local_dir = generate_tmp_dir(remote)?;

		// Try to open it as a local repository first,
		// and if that fails, clone it from the remote.
		let repo = Repository::open(&local_dir);
		if let Ok(repo) = repo {
			return Ok(Self { repo });
		}

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

		Ok(Self {
			repo: RepoBuilder::new()
				.bare(true)
				.fetch_options(fetch_opts)
				.clone(remote, &local_dir)?,
		})
	}

	/// Gets the human-readable name of this remote (e.g. the name of the overarching organization).
	pub fn name(&self) -> Result<Option<String>, Error> {
		self.fetch_commit_message("meta/name")
	}

	/// Sets the human-readable name of this remote (e.g. the name of the overarching organization).
	pub fn set_name(&self, name: &str) -> Result<(), Error> {
		self.send_commit("meta/name", name)
	}

	fn fetch_commit_message(&self, branch: &str) -> Result<Option<String>, Error> {
		// Get the HEAD commit of the branch
		let head = match self.repo.revparse_single(&format!("refs/heads/{branch}")) {
			Ok(h) => h,
			Err(_) => return Ok(None),
		};
		let commit = head.peel_to_commit()?;
		Ok(commit.message().map(|b| b.to_string()))
	}

	fn send_commit(&self, branch: &str, message: &str) -> Result<(), Error> {
		let ref_head = format!("refs/heads/{branch}");

		let head = self
			.repo
			.revparse_single(&ref_head)
			.and_then(|h| h.peel_to_commit())
			.ok();

		let tree = head
			.clone()
			.map(|h| h.tree())
			.unwrap_or_else(|| self.repo.find_tree(self.repo.treebuilder(None)?.write()?))?;

		let sig = self.repo.signature()?;

		let commit = self.repo.commit(
			None,
			&sig,
			&sig,
			message,
			&tree,
			&head.as_ref().map(|h| vec![h]).unwrap_or_default(),
		)?;

		// Now push the commit to the remote. We don't update the local ref
		// yet until the push succeeds. Yes, this creates a bit of a race condition,
		// but the more error-prone operation is the push, whereas the local ref update
		// is trivial and only fails if there's some sort of disk I/O failure, or if something
		// else is modifies the repository at the same time.
		let mut remote = self.repo.find_remote("origin")?;
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
			None => Err(Error::NotPushed(branch.to_string())),
			Some(Some(status)) => Err(Error::PushFailed(branch.to_string(), status)),
			Some(None) => {
				// Finally update the branch's ref to the newly created commit
				// in our local repository.
				self.repo.reference(
					"refs/heads/meta/name",
					commit,
					true,
					&format!("commit: {commit}"),
				)?;

				Ok(())
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
fn generate_tmp_dir(remote: &str) -> Result<PathBuf, Error> {
	use ::sha2::Digest;

	let mut hasher = ::sha2::Sha256::new();
	hasher.update(remote.as_bytes());
	let hash = hasher.finalize();
	let hash = ::hex::encode(hash);
	let mut path = ::std::env::temp_dir();
	path.push("minimap");
	path.push(hash);
	::std::fs::create_dir_all(&path).map_err(|e| Error::TempDir(path.clone(), e))?;
	Ok(path)
}
