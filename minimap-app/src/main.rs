#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use minimap_core::{Error, GitRemote, Result, Workspace};
use std::{
	collections::HashMap,
	sync::{Arc, Mutex},
};
use tauri::State;

#[derive(Default)]
struct MinimapState(Mutex<HashMap<String, Arc<Mutex<Workspace<'static, GitRemote>>>>>);

#[tauri::command]
async fn hello() -> String {
	"Hello World!".to_string()
}

#[tauri::command]
fn workspace_open(uri: String, state: State<'_, MinimapState>) -> Result<()> {
	let mut workspaces = state.0.lock().unwrap();
	if !workspaces.contains_key(&uri) {
		let remote = GitRemote::open(&uri)?;
		let workspace = Workspace::open(remote);
		workspaces.insert(uri.clone(), Arc::new(Mutex::new(workspace)));
	}
	Ok(())
}

#[tauri::command]
fn workspace_project_create(
	uri: String,
	slug: String,
	state: State<'_, MinimapState>,
) -> Result<()> {
	let mut workspaces = state.0.lock().unwrap();
	let workspace = workspaces
		.get_mut(&uri)
		.ok_or_else(|| Error::NotFound("workspace".to_string(), uri))?;
	let workspace = workspace.lock().unwrap();
	let res = workspace.create_project(&slug)?;

	if res.is_err() {
		Err(Error::Exists("project".to_string(), slug))
	} else {
		Ok(())
	}
}

fn main() {
	tauri::Builder::default()
		.manage(MinimapState::default())
		.invoke_handler(tauri::generate_handler![
			hello,
			workspace_open,
			workspace_project_create
		])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
