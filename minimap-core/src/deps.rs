//! Dependency management utility types and origin implementations.
//!
//! Minimap's dependency system doesn't dictate that dependencies
//! need to be other Minimap tickets in the same workspace. In fact,
//! dependencies don't need to be ticket-like entities at all, and
//! can originate from any source and represent any kind of check
//! as long as the check results in a "pending" or "completed" state.
//!
//! Dependency "origins" are sources from which dependency statuses
//! can be queried. Each entity for which a status is queried is referred
//! to as an "endpoint". For example, ticket IDs, issue numbers, CI job
//! run identifiers, etc. are all endpoints.
//!
//! The core [`Workspace`] type holds only a slug for an origin, and the
//! endpoint string. The slug is used to look up instances of origins,
//! and the endpoint string is meant to be passed to the origin to
//! perform the lookup.
//!
//! Dependency origin slugs must not contain the `@` character, and
//! the special origin slug `_` (by itself) refers to the workspace
//! within which the ticket resides (and thus the `_` origin's endpoints
//! are ticket slugs, i.e. `project-123`).

use crate::{DependencyResolver, DependencyStatus, Error, Result};
use std::collections::HashMap;

pub(crate) mod minimap;

pub use self::minimap::*;

/// Dependency origins are sources from which dependency statuses
/// can be queried. The "handle" to a dependency is referred to as
/// an endpoint, and is a string that uniquely identifies the
/// dependency within the origin.
pub trait DependencyOrigin {
	/// The unique identifier ("slug") of the origin.
	///
	/// Origin slugs cannot be `_` and cannot contain the `@` character.
	fn slug(&self) -> &str;

	/// Query the origin for the status of the given endpoint.
	fn status(
		&self,
		endpoint: &str,
	) -> std::result::Result<DependencyStatus, Box<dyn std::error::Error>>;
}

/// A registry of dependency origins that can be queried for
/// dependency statuses.
pub struct DependencyRegistry {
	origins: HashMap<String, Box<dyn DependencyOrigin>>,
}

fn validate_origin_slug(slug: &str) -> Result<()> {
	if slug == "_" || slug == "minimap" || slug.contains('@') {
		return Err(Error::MalformedOrigin(slug.to_string()));
	}

	Ok(())
}

impl Default for DependencyRegistry {
	fn default() -> Self {
		Self::new()
	}
}

impl DependencyRegistry {
	/// Create a new registry. By default, the `minimap` origin is
	/// registered, which is the origin for Minimap workspaces.
	pub fn new() -> Self {
		let mut origins = HashMap::<String, Box<dyn DependencyOrigin>>::new();

		origins.insert("minimap".to_string(), Box::new(MinimapDependencyOrigin));

		Self { origins }
	}

	/// Register an origin with this registry.
	pub fn register(&mut self, origin: Box<dyn DependencyOrigin>) -> Result<()> {
		validate_origin_slug(origin.slug())?;

		self.origins.insert(origin.slug().to_string(), origin);
		Ok(())
	}
}

impl DependencyResolver for DependencyRegistry {
	fn status(&self, slug: &str, endpoint: &str) -> Result<DependencyStatus> {
		if slug == "_" {
			return Err(Error::MalformedOrigin(slug.to_string()));
		}

		match self.origins.get(slug) {
			Some(origin) => origin.status(endpoint).map_err(Error::Origin),
			None => Err(Error::UnknownOrigin(slug.to_string())),
		}
	}
}
