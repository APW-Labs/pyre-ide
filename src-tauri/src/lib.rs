use std::path::PathBuf;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use pyre_python_engine::{
    self, InterpreterMetadata,
    venv::{VenvManager, VenvMetadata},
};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn test_pyre_engine() -> Result<Vec<InterpreterMetadata>, String> {
    let workspace_root = std::env::current_dir().unwrap_or_default();
    let interpreters =
        pyre_python_engine::interpreter::InterpreterScanner::discover_all(&workspace_root).await;
    Ok(interpreters)
}

#[tauri::command]
async fn create_venv_command(
    project_path: String,
    venv_name: String,
) -> Result<VenvMetadata, String> {
    let root = PathBuf::from(project_path);

    let metadata = VenvManager::create_venv(&root, &venv_name, None)
        .await
        .map_err(|e| e.to_string())?;

    Ok(metadata)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, test_pyre_engine, create_venv_command])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
