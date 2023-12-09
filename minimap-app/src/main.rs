#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};

use minimap_core::{MemoryRemote, Record, Workspace};
use serde::{de::Deserialize, ser::Serialize};
use slotmap::{new_key_type, Key, KeyData, SlotMap};
use tauri::State;

new_key_type! { pub struct WorkspaceKey; }

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

#[tauri::command]
fn workspace_open(
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
fn workspace_name(
	workspace: WorkspaceKey,
	workspace_registry: State<WorkspaceRegistry>,
) -> Result<Option<String>> {
	let workspace_registry = workspace_registry.lock().unwrap();
	let workspace_mutex = workspace_registry
		.get(workspace)
		.cloned()
		.ok_or(Error::NoSuchWorkspace(workspace))?;
	let workspace = workspace_mutex.lock().unwrap();
	Ok(workspace.name().map(|s| s.map(|s| s.message()))?)
}

#[tauri::command]
fn workspace_set_name(
	workspace: WorkspaceKey,
	workspace_registry: State<WorkspaceRegistry>,
	name: String,
) -> Result<()> {
	let workspace_registry = workspace_registry.lock().unwrap();
	let workspace_mutex = workspace_registry
		.get(workspace)
		.cloned()
		.ok_or(Error::NoSuchWorkspace(workspace))?;
	let workspace = workspace_mutex.lock().unwrap();
	workspace.set_name(&name)?;
	Ok(())
}

fn main() {
	tauri::Builder::default()
		.manage(WorkspaceRegistry::default())
		.manage::<Mutex<Option<WorkspaceKey>>>(Mutex::default())
		.invoke_handler(tauri::generate_handler![
			workspace_open,
			workspace_name,
			workspace_set_name
		])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
