use std::{collections::HashMap, path::PathBuf};
use tauri::{AppHandle, Emitter};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use pyre_python_engine::{
    self, InterpreterMetadata, ProcessOutput, ProcessSupervisor, supervisor,
    venv::{VenvManager, VenvMetadata},
};

use tokio::spawn;

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

#[tauri::command]
async fn run_python_script_command(
    app: AppHandle,
    python_path: String,
    script_path: String,
) -> Result<u32, String> {
    let py_path: PathBuf = PathBuf::from(python_path);
    let scr_path: PathBuf = PathBuf::from(script_path);
    let empty_env = HashMap::new();
    let empty_args = vec![];

    let (supervisor, mut rx) =
        match ProcessSupervisor::spawn(&py_path, &scr_path, &empty_args, &empty_env, None).await {
            Ok(res) => res,
            Err(e) => {
                eprintln!("FAILED TO SPAWN PYTHON SCRIPT: {:?}", e);
                return Err(e.to_string());
            }
        };

    let pid = supervisor.pid();

    spawn(async move {
        while let Some(output) = rx.recv().await {
            match output {
                ProcessOutput::Stdout(line) => {
                    let _ = app.emit("python-stdout", line);
                }
                ProcessOutput::Stderr(line) => {
                    let _ = app.emit("python-stderr", line);
                }
            }
        }
    });

    Ok(pid)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            test_pyre_engine,
            create_venv_command,
            run_python_script_command
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
