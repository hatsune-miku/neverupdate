use std::io::Write;
use std::path::Path;

use crate::error::Result;
use crate::model::{DaemonSnapshot, HistoryEntry, InterceptionEntry};
use crate::pathing::{daemon_snapshot_file_path, history_file_path, interception_file_path};

const MAX_HISTORY_ITEMS: usize = 500;

pub fn append_history_entry(entry: &HistoryEntry) -> Result<()> {
    let path = history_file_path()?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    let line = serde_json::to_string(entry)?;
    file.write_all(line.as_bytes())?;
    file.write_all(b"\n")?;
    trim_jsonl_tail::<HistoryEntry>(&path, MAX_HISTORY_ITEMS)?;
    Ok(())
}

pub fn read_history(limit: usize) -> Result<Vec<HistoryEntry>> {
    let path = history_file_path()?;
    let safe_limit = limit.min(MAX_HISTORY_ITEMS);
    read_jsonl_latest::<HistoryEntry>(&path, safe_limit)
}

pub fn append_interception_entry(entry: &InterceptionEntry) -> Result<()> {
    let path = interception_file_path()?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    let line = serde_json::to_string(entry)?;
    file.write_all(line.as_bytes())?;
    file.write_all(b"\n")?;
    trim_jsonl_tail::<InterceptionEntry>(&path, MAX_HISTORY_ITEMS)?;
    Ok(())
}

pub fn read_interceptions(limit: usize) -> Result<Vec<InterceptionEntry>> {
    let path = interception_file_path()?;
    let safe_limit = limit.min(MAX_HISTORY_ITEMS);
    read_jsonl_latest::<InterceptionEntry>(&path, safe_limit)
}

pub fn clear_interceptions() -> Result<()> {
    let path = interception_file_path()?;
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
}

pub fn clear_history() -> Result<()> {
    let path = history_file_path()?;
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(())
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

fn read_jsonl_latest<T>(path: &Path, limit: usize) -> Result<Vec<T>>
where
    T: serde::de::DeserializeOwned,
{
    if !path.exists() {
        return Ok(vec![]);
    }

    let content = std::fs::read_to_string(path)?;
    let mut list: Vec<T> = content
        .lines()
        .filter_map(|line| serde_json::from_str::<T>(line).ok())
        .collect();
    list.reverse();
    if list.len() > limit {
        list.truncate(limit);
    }
    Ok(list)
}

fn trim_jsonl_tail<T>(path: &Path, max_items: usize) -> Result<()>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    let content = std::fs::read_to_string(path)?;
    let mut list: Vec<T> = content
        .lines()
        .filter_map(|line| serde_json::from_str::<T>(line).ok())
        .collect();
    if list.len() <= max_items {
        return Ok(());
    }

    let keep_from = list.len() - max_items;
    list.drain(..keep_from);

    let mut out = String::new();
    for item in list {
        out.push_str(&serde_json::to_string(&item)?);
        out.push('\n');
    }
    std::fs::write(path, out)?;
    Ok(())
}
