use std::{collections::HashMap, path::PathBuf, sync::{Arc, Mutex}, time::Duration};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use pyre_python_engine::{
    self, InterpreterMetadata, ProcessOutput, ProcessSupervisor,
    venv::{VenvManager, VenvMetadata},
};

use tokio::sync::oneshot;

#[derive(Clone, Serialize)]
struct PythonExitPayload {
    pid: u32,
    exit_code: Option<i32>
}

#[derive(Default)]
struct ProcessRegistry {
    kill_senders: Arc<Mutex<HashMap<u32, tokio::sync::oneshot::Sender<()>>>>
}

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
    registry: State<'_, ProcessRegistry>,
    python_path: String,
    script_path: String,
) -> Result<u32, String> {
    let py_path = PathBuf::from(python_path);
    let scr_path = PathBuf::from(script_path);
    let empty_env = HashMap::new();
    let empty_args = vec![];

    let (mut supervisor, mut rx) =
        ProcessSupervisor::spawn(&py_path, &scr_path, &empty_args, &empty_env, None)
            .await
            .map_err(|e| e.to_string())?;

    let pid = supervisor.pid();
    let (kill_tx, mut kill_rx) = oneshot::channel::<()>();

    {
        let mut map = registry.kill_senders.lock().unwrap_or_else(|e| e.into_inner());
        map.insert(pid, kill_tx);
    }

    let kill_senders_cleanup = registry.kill_senders.clone();

    tokio::spawn(async move {
        let mut killed = false;

        loop {
            tokio::select! {
                _ = &mut kill_rx => {
                    killed = true;
                    let _ = supervisor.terminate_tree().await;
                    break;
                }
                maybe_output = rx.recv() => {
                    match maybe_output {
                        Some(ProcessOutput::Stdout(line)) => {
                            let _ = app.emit("python-stdout", line);
                        }
                        Some(ProcessOutput::Stderr(line)) => {
                            let _ = app.emit("python-stderr", line);
                        }
                        None => break,
                    }
                }
            }
        }

        let exit_code = if killed {
            None
        } else {
            supervisor
                .wait_with_timeout(Duration::from_secs(300))
                .await
                .ok()
        };

        {
            let mut map = kill_senders_cleanup.lock().unwrap_or_else(|e| e.into_inner());
            map.remove(&pid);
        }

        let _ = app.emit(
            "python-exit",
            PythonExitPayload { pid, exit_code },
        );
    });

    Ok(pid)
}

#[tauri::command]
async fn stop_python_script_command(
    registry: State<'_, ProcessRegistry>,
    pid: u32,
) -> Result<(), String> {
    let sender = {
        let mut map = registry
            .kill_senders
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        map.remove(&pid)
    };

    if let Some(tx) = sender {
        let _ = tx.send(());
        Ok(())
    } else {
        Err(format!("No active process found with PID {}", pid))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(ProcessRegistry::default())
        .invoke_handler(tauri::generate_handler![
            greet,
            test_pyre_engine,
            create_venv_command,
            run_python_script_command,
            stop_python_script_command
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
