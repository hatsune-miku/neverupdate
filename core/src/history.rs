use std::io::Write;

use crate::error::Result;
use crate::model::{DaemonSnapshot, HistoryEntry};
use crate::pathing::{daemon_snapshot_file_path, history_file_path};

pub fn append_history_entry(entry: &HistoryEntry) -> Result<()> {
    let path = history_file_path()?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    let line = serde_json::to_string(entry)?;
    file.write_all(line.as_bytes())?;
    file.write_all(b"\n")?;
    Ok(())
}

pub fn read_history(limit: usize) -> Result<Vec<HistoryEntry>> {
    let path = history_file_path()?;
    if !path.exists() {
        return Ok(vec![]);
    }

    let content = std::fs::read_to_string(path)?;
    let mut list: Vec<HistoryEntry> = content
        .lines()
        .filter_map(|line| serde_json::from_str::<HistoryEntry>(line).ok())
        .collect();

    list.reverse();
    if list.len() > limit {
        list.truncate(limit);
    }

    Ok(list)
}

pub fn save_snapshot(snapshot: &DaemonSnapshot) -> Result<()> {
    let path = daemon_snapshot_file_path()?;
    let content = serde_json::to_string_pretty(snapshot)?;
    std::fs::write(path, content)?;
    Ok(())
}

pub fn load_snapshot() -> Result<Option<DaemonSnapshot>> {
    let path = daemon_snapshot_file_path()?;
    if !path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(path)?;
    let snapshot = serde_json::from_str::<DaemonSnapshot>(&content)?;
    Ok(Some(snapshot))
}
