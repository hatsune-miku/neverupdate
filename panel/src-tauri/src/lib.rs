use std::error::Error;

use nu_core::{
    clear_history, clear_interceptions, daemon_service_exists, execute_all, execute_guard_action,
    list_guard_points, load_daemon_snapshot, query_guard_states, read_history, read_interceptions,
    register_daemon_service,
    reregister_daemon_service, run_extreme_mode, run_preflight_checks, start_daemon_service,
    stop_daemon_service, unregister_daemon_service, GuardAction, GuardPointDefinition,
    GuardPointStatus, GuardSummary, HistoryEntry, InterceptionEntry, PreflightReport,
};
use tauri::{Manager, Runtime};

#[tauri::command]
fn run_preflight_checks_cmd() -> PreflightReport {
    run_preflight_checks()
}

#[tauri::command]
fn list_guard_points_cmd() -> Vec<GuardPointDefinition> {
    list_guard_points()
}

#[tauri::command]
fn query_guard_states_cmd() -> Result<Vec<GuardPointStatus>, String> {
    query_guard_states().map_err(|error| error.to_string())
}

#[tauri::command]
fn execute_guard_action_cmd(
    point_id: String,
    action: GuardAction,
) -> Result<GuardPointStatus, String> {
    execute_guard_action(&point_id, action).map_err(|error| error.to_string())
}

#[tauri::command]
fn execute_all_cmd(action: GuardAction) -> GuardSummary {
    execute_all(action)
}

#[tauri::command]
fn read_history_cmd(limit: usize) -> Result<Vec<HistoryEntry>, String> {
    read_history(limit).map_err(|error| error.to_string())
}

#[tauri::command]
fn clear_history_cmd() -> Result<(), String> {
    clear_history().map_err(|error| error.to_string())
}

#[tauri::command]
fn read_interceptions_cmd(limit: usize) -> Result<Vec<InterceptionEntry>, String> {
    read_interceptions(limit).map_err(|error| error.to_string())
}

#[tauri::command]
fn clear_interceptions_cmd() -> Result<(), String> {
    clear_interceptions().map_err(|error| error.to_string())
}

#[tauri::command]
fn daemon_snapshot_cmd() -> Result<Option<nu_core::DaemonSnapshot>, String> {
    load_daemon_snapshot().map_err(|error| error.to_string())
}

fn resolve_bundled_daemon_path<R: Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<std::path::PathBuf, String> {
    if let Ok(resource_dir) = app.path().resource_dir() {
        let file = resource_dir.join("neverupdate-daemon.exe");
        if file.exists() {
            return Ok(file);
        }

        let nested = resource_dir.join("bin").join("neverupdate-daemon.exe");
        if nested.exists() {
            return Ok(nested);
        }
    }

    let exe_path = std::env::current_exe().map_err(|error| error.to_string())?;
    let bin_dir = exe_path
        .parent()
        .ok_or_else(|| String::from("no parent directory"))?;

    let local_release = bin_dir
        .join("..")
        .join("..")
        .join("..")
        .join("target")
        .join("release")
        .join("neverupdate-daemon.exe");
    if local_release.exists() {
        return Ok(local_release);
    }

    let local_debug = bin_dir
        .join("..")
        .join("..")
        .join("..")
        .join("target")
        .join("debug")
        .join("neverupdate-daemon.exe");
    if local_debug.exists() {
        return Ok(local_debug);
    }

    Err(String::from("bundled daemon binary was not found"))
}

#[tauri::command]
async fn daemon_service_register<R: Runtime>(app: tauri::AppHandle<R>) -> Result<bool, String> {
    let daemon_path = resolve_bundled_daemon_path(&app)?;
    let path = daemon_path.to_string_lossy().into_owned();
    tauri::async_runtime::spawn_blocking(move || register_daemon_service(&path).map_err(|e| e.to_string()))
        .await
        .map_err(|e| format!("join: {e}"))??;
    Ok(true)
}

#[tauri::command]
async fn daemon_service_reregister<R: Runtime>(app: tauri::AppHandle<R>) -> Result<bool, String> {
    let daemon_path = resolve_bundled_daemon_path(&app)?;
    let path = daemon_path.to_string_lossy().into_owned();
    tauri::async_runtime::spawn_blocking(move || reregister_daemon_service(&path).map_err(|e| e.to_string()))
        .await
        .map_err(|e| format!("join: {e}"))??;
    Ok(true)
}

#[tauri::command]
async fn daemon_service_start() -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(|| start_daemon_service().map_err(|e| e.to_string()))
        .await
        .map_err(|e| format!("join: {e}"))??;
    Ok(true)
}

#[tauri::command]
async fn daemon_service_stop() -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(|| stop_daemon_service().map_err(|e| e.to_string()))
        .await
        .map_err(|e| format!("join: {e}"))??;
    Ok(true)
}

#[tauri::command]
async fn daemon_service_unregister() -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(|| unregister_daemon_service().map_err(|e| e.to_string()))
        .await
        .map_err(|e| format!("join: {e}"))??;
    Ok(true)
}

#[tauri::command]
fn run_extreme_mode_cmd() -> Result<bool, String> {
    run_extreme_mode().map_err(|error| error.to_string())?;
    Ok(true)
}

#[tauri::command]
fn daemon_service_exists_cmd() -> bool {
    daemon_service_exists()
}

fn setup(app: &mut tauri::App) -> Result<(), Box<dyn Error>> {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_zoom(1.0);
        let _ = window.set_content_protected(false);
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let result = tauri::Builder::default()
        .setup(setup)
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .invoke_handler(tauri::generate_handler![
            run_preflight_checks_cmd,
            list_guard_points_cmd,
            query_guard_states_cmd,
            execute_guard_action_cmd,
            execute_all_cmd,
            read_history_cmd,
            clear_history_cmd,
            read_interceptions_cmd,
            clear_interceptions_cmd,
            daemon_snapshot_cmd,
            daemon_service_register,
            daemon_service_reregister,
            daemon_service_start,
            daemon_service_stop,
            daemon_service_unregister,
            daemon_service_exists_cmd,
            run_extreme_mode_cmd,
        ])
        .run(tauri::generate_context!());

    if let Err(error) = result {
        eprintln!("tauri application stopped with error: {error}");
    }
}
