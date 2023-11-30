use crate::{DependencyOrigin, DependencyStatus, Error, GitRemote, Workspace};

/// A dependency origin that queries remote Minimap workspaces
/// for dependency statuses over Git.
pub struct MinimapDependencyOrigin;

impl DependencyOrigin for MinimapDependencyOrigin {
	fn slug(&self) -> &str {
		"minimap"
	}

	fn status(
		&self,
		endpoint: &str,
	) -> std::result::Result<DependencyStatus, Box<dyn std::error::Error>> {
		// Parse the endpoint. Endpoints are `git-remote@ticket-slug`.
		let mut parts = endpoint.split('@');
		let remote = parts
			.next()
			.ok_or(Error::MalformedEndpoint(endpoint.to_string()))?;
		let ticket_slug = parts
			.next()
			.ok_or(Error::MalformedEndpoint(endpoint.to_string()))?;

		let remote = GitRemote::open(remote)?;
		let workspace = Workspace::open(remote);

		Ok(workspace.ticket(ticket_slug)?.state().map(|s| s.0)?.into())
	}
}
