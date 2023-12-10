#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use minimap_core::{GitRemote, MemoryRemote, Record, TicketState, Workspace};
use paste::paste;
use serde::{de::Deserialize, ser::Serialize};
use slotmap::{new_key_type, Key, KeyData, SlotMap};
use std::{
	collections::HashMap,
	sync::{Arc, Mutex},
};
use tauri::State;

new_key_type! { pub struct WorkspaceKey; }

macro_rules! remote_backend_impl {
	($Registry:ty, $Record:ty, $prefix:ident) => {
		paste! {
			#[tauri::command]
			fn [<$prefix _workspace_name>](
				workspace: WorkspaceKey,
				workspace_registry: State<$Registry>,
			) -> Result<Option<$Record>> {
				let workspace_registry = workspace_registry.lock().unwrap();
				let workspace_mutex = workspace_registry
					.get(workspace)
					.cloned()
					.ok_or(Error::NoSuchWorkspace(workspace))?;
				let workspace = workspace_mutex.lock().unwrap();
				let name = workspace.name()?.map(Into::into);
				drop(workspace);
				drop(workspace_mutex);
				drop(workspace_registry);
				Ok(name)
			}

			#[tauri::command]
			fn [<$prefix _workspace_set_name>](
				workspace: WorkspaceKey,
				workspace_registry: State<$Registry>,
				name: String,
			) -> Result<$Record> {
				let workspace_registry = workspace_registry.lock().unwrap();
				let workspace_mutex = workspace_registry
					.get(workspace)
					.cloned()
					.ok_or(Error::NoSuchWorkspace(workspace))?;
				let workspace = workspace_mutex.lock().unwrap();
				let record = workspace.set_name(&name)?.into();
				Ok(record)
			}

			#[tauri::command]
			fn [<$prefix _workspace_description>](
				workspace: WorkspaceKey,
				workspace_registry: State<$Registry>,
			) -> Result<Option<$Record>> {
				let workspace_registry = workspace_registry.lock().unwrap();
				let workspace_mutex = workspace_registry
					.get(workspace)
					.cloned()
					.ok_or(Error::NoSuchWorkspace(workspace))?;
				let workspace = workspace_mutex.lock().unwrap();
				let record = workspace.description()?.map(Into::into);
				Ok(record)
			}

			#[tauri::command]
			fn [<$prefix _workspace_set_description>](
				workspace: WorkspaceKey,
				workspace_registry: State<$Registry>,
				description: String,
			) -> Result<$Record> {
				let workspace_registry = workspace_registry.lock().unwrap();
				let workspace_mutex = workspace_registry
					.get(workspace)
					.cloned()
					.ok_or(Error::NoSuchWorkspace(workspace))?;
				let workspace = workspace_mutex.lock().unwrap();
				let record = workspace.set_description(&description)?.into();
				Ok(record)
			}

			#[tauri::command]
			fn [<$prefix _workspace_create_project>](
				workspace: WorkspaceKey,
				workspace_registry: State<$Registry>,
				project: String,
			) -> Result<std::result::Result<String, $Record>> {
				let workspace_registry = workspace_registry.lock().unwrap();
				let workspace_mutex = workspace_registry
					.get(workspace)
					.cloned()
					.ok_or(Error::NoSuchWorkspace(workspace))?;
				let workspace = workspace_mutex.lock().unwrap();
				let record = workspace
					.create_project(&project)?
					.map(|_| project)
					.map_err(Into::into);
				Ok(record)
			}

			#[tauri::command]
			fn [<$prefix _workspace_projects>](
				workspace: WorkspaceKey,
				workspace_registry: State<$Registry>,
			) -> Result<Vec<$Record>> {
				let workspace_registry = workspace_registry.lock().unwrap();
				let workspace_mutex = workspace_registry
					.get(workspace)
					.cloned()
					.ok_or(Error::NoSuchWorkspace(workspace))?;
				let workspace = workspace_mutex.lock().unwrap();
				let record = Vec::from_iter(
					workspace.projects()?.into_iter().map(Into::into),
				);
				Ok(record)
			}

			#[tauri::command]
			fn [<$prefix _workspace_delete_project>](
				workspace: WorkspaceKey,
				workspace_registry: State<$Registry>,
				project: String,
			) -> Result<std::result::Result<$Record, Option<$Record>>> {
				let workspace_registry = workspace_registry.lock().unwrap();
				let workspace_mutex = workspace_registry
					.get(workspace)
					.cloned()
					.ok_or(Error::NoSuchWorkspace(workspace))?;
				let workspace = workspace_mutex.lock().unwrap();
				let record = workspace
					.delete_project(&project)?
					.map(Into::into)
					.map_err(|e| e.map(Into::into));
				Ok(record)
			}

			#[tauri::command]
			fn [<$prefix _project_set_name>](
				workspace: WorkspaceKey,
				workspace_registry: State<$Registry>,
				project: String,
				name: String,
			) -> Result<$Record> {
				let workspace_registry = workspace_registry.lock().unwrap();
				let workspace_mutex = workspace_registry
					.get(workspace)
					.cloned()
					.ok_or(Error::NoSuchWorkspace(workspace))?;
				let workspace = workspace_mutex.lock().unwrap();
				let project = workspace.project(&project)?;
				let record = project.set_name(&name)?.into();
				Ok(record)
			}

			#[tauri::command]
			fn [<$prefix _project_set_description>](
				workspace: WorkspaceKey,
				workspace_registry: State<$Registry>,
				project: String,
				description: String,
			) -> Result<$Record> {
				let workspace_registry = workspace_registry.lock().unwrap();
				let workspace_mutex = workspace_registry
					.get(workspace)
					.cloned()
					.ok_or(Error::NoSuchWorkspace(workspace))?;
				let workspace = workspace_mutex.lock().unwrap();
				let project = workspace.project(&project)?;
				let record = project.set_description(&description)?.into();
				Ok(record)
			}

			#[tauri::command]
			fn [<$prefix _project_name>](
				workspace: WorkspaceKey,
				workspace_registry: State<$Registry>,
				project: String,
			) -> Result<Option<$Record>> {
				let workspace_registry = workspace_registry.lock().unwrap();
				let workspace_mutex = workspace_registry
					.get(workspace)
					.cloned()
					.ok_or(Error::NoSuchWorkspace(workspace))?;
				let workspace = workspace_mutex.lock().unwrap();
				let project = workspace.project(&project)?;
				let record = project.name()?.map(Into::into);
				Ok(record)
			}

			#[tauri::command]
			fn [<$prefix _project_description>](
				workspace: WorkspaceKey,
				workspace_registry: State<$Registry>,
				project: String,
			) -> Result<Option<$Record>> {
				let workspace_registry = workspace_registry.lock().unwrap();
				let workspace_mutex = workspace_registry
					.get(workspace)
					.cloned()
					.ok_or(Error::NoSuchWorkspace(workspace))?;
				let workspace = workspace_mutex.lock().unwrap();
				let project = workspace.project(&project)?;
				let record = project.description()?.map(Into::into);
				Ok(record)
			}

			#[tauri::command]
			fn [<$prefix _project_create_ticket>](
				workspace: WorkspaceKey,
				workspace_registry: State<$Registry>,
				project: String,
			) -> Result<String> {
				let workspace_registry = workspace_registry.lock().unwrap();
				let workspace_mutex = workspace_registry.get(workspace).cloned().unwrap();
				let workspace = workspace_mutex.lock().unwrap();
				let project = workspace.project(&project)?;
				Ok(project.create_ticket()?.slug().to_string())
			}

			#[tauri::command]
			fn [<$prefix _ticket_title>](
				workspace: WorkspaceKey,
				workspace_registry: State<$Registry>,
				ticket: String,
			) -> Result<Option<$Record>> {
				let workspace_registry = workspace_registry.lock().unwrap();
				let workspace_mutex = workspace_registry.get(workspace).cloned().unwrap();
				let workspace = workspace_mutex.lock().unwrap();
				let ticket = workspace.ticket(&ticket)?;
				let record = ticket.title()?.map(Into::into);
				Ok(record)
			}

			#[tauri::command]
			fn [<$prefix _ticket_set_title>](
				workspace: WorkspaceKey,
				workspace_registry: State<$Registry>,
				ticket: String,
				title: String,
			) -> Result<$Record> {
				let workspace_registry = workspace_registry.lock().unwrap();
				let workspace_mutex = workspace_registry.get(workspace).cloned().unwrap();
				let workspace = workspace_mutex.lock().unwrap();
				let ticket = workspace.ticket(&ticket)?;
				let record = ticket.set_title(&title)?.into();
				Ok(record)
			}

			#[tauri::command]
			fn [<$prefix _ticket_add_comment>](
				workspace: WorkspaceKey,
				workspace_registry: State<$Registry>,
				ticket: String,
				comment: String,
			) -> Result<$Record> {
				let workspace_registry = workspace_registry.lock().unwrap();
				let workspace_mutex = workspace_registry.get(workspace).cloned().unwrap();
				let workspace = workspace_mutex.lock().unwrap();
				let ticket = workspace.ticket(&ticket)?;
				let record = ticket.add_comment(&comment)?.into();
				Ok(record)
			}

			#[tauri::command]
			fn [<$prefix _ticket_comments>](
				workspace: WorkspaceKey,
				workspace_registry: State<$Registry>,
				ticket: String,
			) -> Result<Vec<$Record>> {
				let workspace_registry = workspace_registry.lock().unwrap();
				let workspace_mutex = workspace_registry.get(workspace).cloned().unwrap();
				let workspace = workspace_mutex.lock().unwrap();
				let ticket = workspace.ticket(&ticket)?;
				let mut comments = Vec::new();
				for comment_record in ticket.comments()? {
					comments.push(comment_record?.into());
				}
				Ok(comments)
			}

			#[tauri::command]
			fn [<$prefix _ticket_upsert_attachment>](
				workspace: WorkspaceKey,
				workspace_registry: State<$Registry>,
				ticket: String,
				name: String,
				data: Vec<u8>,
			) -> Result<$Record> {
				let workspace_registry = workspace_registry.lock().unwrap();
				let workspace_mutex = workspace_registry.get(workspace).cloned().unwrap();
				let workspace = workspace_mutex.lock().unwrap();
				let ticket = workspace.ticket(&ticket)?;
				let record = ticket.upsert_attachment(&name, &data)?.into();
				Ok(record)
			}

			#[tauri::command]
			fn [<$prefix _ticket_upsert_attachment_filepath>](
				workspace: WorkspaceKey,
				workspace_registry: State<$Registry>,
				ticket: String,
				name: String,
				filepath: String,
			) -> Result<$Record> {
				let data = std::fs::read(filepath).map_err(minimap_core::Error::Io)?;
				let workspace_registry = workspace_registry.lock().unwrap();
				let workspace_mutex = workspace_registry.get(workspace).cloned().unwrap();
				let workspace = workspace_mutex.lock().unwrap();
				let ticket = workspace.ticket(&ticket)?;
				let record = ticket.upsert_attachment(&name, &data)?.into();
				Ok(record)
			}

			#[tauri::command]
			fn [<$prefix _ticket_remove_attachment>](
				workspace: WorkspaceKey,
				workspace_registry: State<$Registry>,
				ticket: String,
				name: String,
			) -> Result<std::result::Result<$Record, Option<$Record>>> {
				let workspace_registry = workspace_registry.lock().unwrap();
				let workspace_mutex = workspace_registry.get(workspace).cloned().unwrap();
				let workspace = workspace_mutex.lock().unwrap();
				let ticket = workspace.ticket(&ticket)?;
				let record = ticket
					.remove_attachment(&name)?
					.map(Into::into)
					.map_err(|e| e.map(Into::into));
				Ok(record)
			}

			#[tauri::command]
			fn [<$prefix _ticket_attachment>](
				workspace: WorkspaceKey,
				workspace_registry: State<$Registry>,
				ticket: String,
				name: String,
			) -> Result<Option<Vec<u8>>> {
				let workspace_registry = workspace_registry.lock().unwrap();
				let workspace_mutex = workspace_registry.get(workspace).cloned().unwrap();
				let workspace = workspace_mutex.lock().unwrap();
				let ticket = workspace.ticket(&ticket)?;
				Ok(ticket.attachment(&name)?)
			}

			#[tauri::command]
			fn [<$prefix _ticket_attachment_base64>](
				workspace: WorkspaceKey,
				workspace_registry: State<$Registry>,
				ticket: String,
				name: String,
			) -> Result<Option<String>> {
				use base64::{engine::general_purpose, Engine as _};
				let workspace_registry = workspace_registry.lock().unwrap();
				let workspace_mutex = workspace_registry.get(workspace).cloned().unwrap();
				let workspace = workspace_mutex.lock().unwrap();
				let ticket = workspace.ticket(&ticket)?;
				let data = ticket.attachment(&name)?;
				Ok(data.map(|d| general_purpose::STANDARD_NO_PAD.encode(d)))
			}

			#[tauri::command]
			fn [<$prefix _ticket_state>](
				workspace: WorkspaceKey,
				workspace_registry: State<$Registry>,
				ticket: String,
			) -> Result<(String, Option<$Record>)> {
				let workspace_registry = workspace_registry.lock().unwrap();
				let workspace_mutex = workspace_registry.get(workspace).cloned().unwrap();
				let workspace = workspace_mutex.lock().unwrap();
				let ticket = workspace.ticket(&ticket)?;
				Ok(ticket
					.state()
					.map(|(s, r)| (s.to_string(), r.map(Into::into)))?)
			}

			#[tauri::command]
			fn [<$prefix _ticket_set_state>](
				workspace: WorkspaceKey,
				workspace_registry: State<$Registry>,
				ticket: String,
				state: String,
			) -> Result<$Record> {
				let state = TicketState::try_from(state)?;
				let workspace_registry = workspace_registry.lock().unwrap();
				let workspace_mutex = workspace_registry.get(workspace).cloned().unwrap();
				let workspace = workspace_mutex.lock().unwrap();
				let ticket = workspace.ticket(&ticket)?;
				let record = ticket.set_state(state)?.into();
				Ok(record)
			}

			#[tauri::command]
			fn [<$prefix _ticket_is_open>](
				workspace: WorkspaceKey,
				workspace_registry: State<$Registry>,
				ticket: String,
			) -> Result<bool> {
				let workspace_registry = workspace_registry.lock().unwrap();
				let workspace_mutex = workspace_registry.get(workspace).cloned().unwrap();
				let workspace = workspace_mutex.lock().unwrap();
				let ticket = workspace.ticket(&ticket)?;
				Ok(ticket.is_open()?)
			}

			#[tauri::command]
			fn [<$prefix _ticket_is_closed>](
				workspace: WorkspaceKey,
				workspace_registry: State<$Registry>,
				ticket: String,
			) -> Result<bool> {
				let workspace_registry = workspace_registry.lock().unwrap();
				let workspace_mutex = workspace_registry.get(workspace).cloned().unwrap();
				let workspace = workspace_mutex.lock().unwrap();
				let ticket = workspace.ticket(&ticket)?;
				Ok(ticket.is_closed()?)
			}

			#[tauri::command]
			fn [<$prefix _ticket_dependencies>](
				workspace: WorkspaceKey,
				workspace_registry: State<$Registry>,
				ticket: String,
			) -> Result<Vec<(String, String, $Record)>> {
				let workspace_registry = workspace_registry.lock().unwrap();
				let workspace_mutex = workspace_registry.get(workspace).cloned().unwrap();
				let workspace = workspace_mutex.lock().unwrap();

				let result = workspace
					.ticket(&ticket)?
					.dependencies()?
					.into_iter()
					.map(|(a, b, r)| (a, b, r.into()))
					.collect();

				Ok(result)
			}

			#[tauri::command]
			fn [<$prefix _ticket_add_dependency>](
				workspace: WorkspaceKey,
				workspace_registry: State<$Registry>,
				ticket: String,
				origin: String,
				endpoint: String,
			) -> Result<$Record> {
				let workspace_registry = workspace_registry.lock().unwrap();
				let workspace_mutex = workspace_registry.get(workspace).cloned().unwrap();
				let workspace = workspace_mutex.lock().unwrap();
				let record = workspace
					.ticket(&ticket)?
					.add_dependency(&origin, &endpoint)?
					.into();
				Ok(record)
			}

			#[tauri::command]
			fn [<$prefix _ticket_remove_dependency>](
				workspace: WorkspaceKey,
				workspace_registry: State<$Registry>,
				ticket: String,
				origin: String,
				endpoint: String,
			) -> Result<Option<$Record>> {
				let workspace_registry = workspace_registry.lock().unwrap();
				let workspace_mutex = workspace_registry.get(workspace).cloned().unwrap();
				let workspace = workspace_mutex.lock().unwrap();
				let record = workspace
					.ticket(&ticket)?
					.remove_dependency(&origin, &endpoint)?
					.map(Into::into);
				Ok(record)
			}
		}
	};
}

remote_backend_impl!(WorkspaceRegistry, TauriRecord<impl Record>, mem);
remote_backend_impl!(GitWorkspaceRegistry, ConcreteTauriRecord, git);

impl Serialize for WorkspaceKey {
	fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_u64(self.data().as_ffi())
	}
}

impl<'a> Deserialize<'a> for WorkspaceKey {
	fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
	where
		D: serde::Deserializer<'a>,
	{
		let id = u64::deserialize(deserializer)?;
		Ok(KeyData::from_ffi(id).into())
	}
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
	#[error("tauri error: {0}")]
	Tauri(#[from] tauri::Error),
	#[error("minimap error: {0}")]
	Minimap(#[from] minimap_core::Error),
	#[error("no such workspace: {0:?}")]
	NoSuchWorkspace(WorkspaceKey),
}

impl serde::ser::Serialize for Error {
	fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_str(&self.to_string())
	}
}

pub(crate) type Result<T> = std::result::Result<T, Error>;

type WorkspaceRegistry<'a> = Mutex<SlotMap<WorkspaceKey, Arc<Mutex<Workspace<'a, MemoryRemote>>>>>;

#[derive(Default)]
struct GitWorkspaceRegistry<'a> {
	inner: Mutex<SlotMap<WorkspaceKey, Arc<Mutex<Workspace<'a, GitRemote>>>>>,
	remotes: Mutex<HashMap<String, WorkspaceKey>>,
}

impl<'a> GitWorkspaceRegistry<'a> {
	/// Locks (but does NOT unwrap, so as to return a Result) the inner slotmap.
	fn lock(
		&self,
	) -> std::result::Result<
		std::sync::MutexGuard<'_, SlotMap<WorkspaceKey, Arc<Mutex<Workspace<'a, GitRemote>>>>>,
		std::sync::PoisonError<
			std::sync::MutexGuard<'_, SlotMap<WorkspaceKey, Arc<Mutex<Workspace<'a, GitRemote>>>>>,
		>,
	> {
		self.inner.lock()
	}
}

#[derive(Debug)]
struct TauriRecord<R: Record>(R);

impl<R: Record> From<R> for TauriRecord<R> {
	#[inline]
	fn from(r: R) -> Self {
		Self(r)
	}
}

impl<R: Record> serde::ser::Serialize for TauriRecord<R> {
	fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		use serde::ser::SerializeMap;
		let mut map = serializer.serialize_map(Some(5))?;
		map.serialize_entry("id", &self.0.id())?;
		map.serialize_entry("author", &self.0.author())?;
		map.serialize_entry("email", &self.0.email())?;
		map.serialize_entry("timestamp", &self.0.timestamp())?;
		map.serialize_entry("message", &self.0.message())?;
		map.end()
	}
}

#[derive(Debug, serde::Serialize)]
struct ConcreteTauriRecord {
	id: String,
	author: String,
	email: String,
	message: String,
	timestamp: i64,
}

impl<R: Record> From<R> for ConcreteTauriRecord {
	#[inline]
	fn from(r: R) -> Self {
		Self {
			id: r.id(),
			author: r.author(),
			email: r.email(),
			message: r.message(),
			timestamp: r.timestamp(),
		}
	}
}

#[tauri::command]
fn mem_workspace_open(
	workspace_registry: State<WorkspaceRegistry>,
	workspace_key: State<Mutex<Option<WorkspaceKey>>>,
) -> Result<WorkspaceKey> {
	if let Some(ref k) = workspace_key.lock().unwrap().clone() {
		return Ok(*k);
	}

	let workspace = Workspace::open(MemoryRemote::new("Max Mustermann", "max@example.com"));
	let key = workspace_registry
		.lock()
		.unwrap()
		.insert(Arc::new(Mutex::new(workspace)));
	workspace_key.lock().unwrap().replace(key);
	Ok(key)
}

#[tauri::command]
fn git_workspace_open(
	workspace_registry: State<GitWorkspaceRegistry>,
	remote: String,
) -> Result<WorkspaceKey> {
	let workspace = Workspace::open(GitRemote::open(&remote)?);
	let key = {
		workspace_registry
			.inner
			.lock()
			.unwrap()
			.insert(Arc::new(Mutex::new(workspace)))
	};
	workspace_registry
		.remotes
		.lock()
		.unwrap()
		.insert(remote, key);
	Ok(key)
}

fn main() {
	tauri::Builder::default()
		.manage(WorkspaceRegistry::default())
		.manage(GitWorkspaceRegistry::default())
		.manage::<Mutex<Option<WorkspaceKey>>>(Mutex::default())
		.invoke_handler(tauri::generate_handler![
			mem_workspace_open,
			mem_workspace_name,
			mem_workspace_set_name,
			mem_workspace_description,
			mem_workspace_set_description,
			mem_workspace_create_project,
			mem_workspace_projects,
			mem_workspace_delete_project,
			mem_project_set_name,
			mem_project_set_description,
			mem_project_name,
			mem_project_description,
			mem_project_create_ticket,
			mem_ticket_title,
			mem_ticket_set_title,
			mem_ticket_add_comment,
			mem_ticket_comments,
			mem_ticket_upsert_attachment,
			mem_ticket_upsert_attachment_filepath,
			mem_ticket_remove_attachment,
			mem_ticket_attachment,
			mem_ticket_attachment_base64,
			mem_ticket_state,
			mem_ticket_set_state,
			mem_workspace_create_project,
			mem_workspace_projects,
			mem_workspace_delete_project,
			mem_project_set_name,
			mem_project_set_description,
			mem_project_name,
			mem_project_description,
			mem_project_create_ticket,
			mem_ticket_title,
			mem_ticket_set_title,
			mem_ticket_add_comment,
			mem_ticket_comments,
			mem_ticket_upsert_attachment,
			mem_ticket_upsert_attachment_filepath,
			mem_ticket_remove_attachment,
			mem_ticket_attachment,
			mem_ticket_attachment_base64,
			mem_ticket_state,
			mem_ticket_set_state,
			mem_ticket_is_open,
			mem_ticket_is_closed,
			mem_ticket_dependencies,
			mem_ticket_add_dependency,
			mem_ticket_remove_dependency,
			git_workspace_open,
			git_workspace_name,
			git_workspace_set_name,
			git_workspace_description,
			git_workspace_set_description,
			git_workspace_create_project,
			git_workspace_projects,
			git_workspace_delete_project,
			git_project_set_name,
			git_project_set_description,
			git_project_name,
			git_project_description,
			git_project_create_ticket,
			git_ticket_title,
			git_ticket_set_title,
			git_ticket_add_comment,
			git_ticket_comments,
			git_ticket_upsert_attachment,
			git_ticket_upsert_attachment_filepath,
			git_ticket_remove_attachment,
			git_ticket_attachment,
			git_ticket_attachment_base64,
			git_ticket_state,
			git_ticket_set_state,
			git_workspace_create_project,
			git_workspace_projects,
			git_workspace_delete_project,
			git_project_set_name,
			git_project_set_description,
			git_project_name,
			git_project_description,
			git_project_create_ticket,
			git_ticket_title,
			git_ticket_set_title,
			git_ticket_add_comment,
			git_ticket_comments,
			git_ticket_upsert_attachment,
			git_ticket_upsert_attachment_filepath,
			git_ticket_remove_attachment,
			git_ticket_attachment,
			git_ticket_attachment_base64,
			git_ticket_state,
			git_ticket_set_state,
			git_ticket_is_open,
			git_ticket_is_closed,
			git_ticket_dependencies,
			git_ticket_add_dependency,
			git_ticket_remove_dependency,
		])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
