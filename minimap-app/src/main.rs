#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tauri::command]
fn hello() -> String {
	"Hello World!".to_string()
}

fn main() {
	tauri::Builder::default()
		.invoke_handler(tauri::generate_handler![hello])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
