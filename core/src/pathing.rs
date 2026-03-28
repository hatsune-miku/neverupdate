use std::path::PathBuf;

use crate::error::{NuError, Result};

pub fn program_data_dir() -> PathBuf {
    let base = std::env::var("PROGRAMDATA").unwrap_or_else(|_| String::from(r"C:\ProgramData"));
    PathBuf::from(base).join("NeverUpdate")
}

pub fn ensure_program_data_dir() -> Result<PathBuf> {
    let dir = program_data_dir();
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn state_file_path() -> Result<PathBuf> {
    Ok(ensure_program_data_dir()?.join("state.json"))
}

pub fn history_file_path() -> Result<PathBuf> {
    Ok(ensure_program_data_dir()?.join("history.jsonl"))
}

pub fn interception_file_path() -> Result<PathBuf> {
    Ok(ensure_program_data_dir()?.join("interceptions.jsonl"))
}

pub fn daemon_snapshot_file_path() -> Result<PathBuf> {
    Ok(ensure_program_data_dir()?.join("daemon-snapshot.json"))
}

pub fn hosts_file_path() -> Result<PathBuf> {
    let windir = std::env::var("WINDIR").unwrap_or_else(|_| String::from(r"C:\Windows"));
    let path = PathBuf::from(windir)
        .join("System32")
        .join("drivers")
        .join("etc")
        .join("hosts");

    if path.exists() {
        Ok(path)
    } else {
        Err(NuError::Unsupported("hosts file was not found".to_string()))
    }
}

pub fn software_distribution_path() -> PathBuf {
    let windir = std::env::var("WINDIR").unwrap_or_else(|_| String::from(r"C:\Windows"));
    PathBuf::from(windir).join("SoftwareDistribution")
}

pub fn task_root_paths() -> Vec<PathBuf> {
    let windir = std::env::var("WINDIR").unwrap_or_else(|_| String::from(r"C:\Windows"));
    let root = PathBuf::from(windir)
        .join("System32")
        .join("Tasks")
        .join("Microsoft")
        .join("Windows");

    vec![root.join("UpdateOrchestrator"), root.join("WaaSMedic")]
}
