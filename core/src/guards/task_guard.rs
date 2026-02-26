use std::path::{Path, PathBuf};

use crate::command::run_command;
use crate::error::Result;
use crate::guards::status_tasks;
use crate::model::GuardPointStatus;
use crate::pathing::task_root_paths;
use crate::state::PersistedState;

const DISABLE_PREFIX: &str = "DISABLE:";

pub fn check(_state: &PersistedState) -> Result<GuardPointStatus> {
    let files = list_target_task_files()?;
    if files.is_empty() {
        return Ok(status_tasks(
            false,
            Some(String::from("no task files found under target folders")),
        ));
    }

    let mut all_guarded = true;
    let mut details = Vec::new();

    for file in files {
        let command_prefixed = task_command_prefixed(&file)?;
        let scheduler_name = scheduler_task_name_from_file(&file);
        let disabled = check_task_disabled(&scheduler_name);
        all_guarded = all_guarded && command_prefixed && disabled;
        details.push(format!(
            "{}: command_prefixed={}, disabled={}",
            scheduler_name, command_prefixed, disabled
        ));
    }

    Ok(status_tasks(all_guarded, Some(details.join(" | "))))
}

pub fn guard(state: &mut PersistedState) -> Result<GuardPointStatus> {
    let files = list_target_task_files()?;

    for file in files {
        patch_task_command(&file, true, state)?;
        let scheduler_name = scheduler_task_name_from_file(&file);
        let _ = run_command("schtasks", &["/Change", "/TN", &scheduler_name, "/DISABLE"]);
    }

    check(state)
}

pub fn release(state: &mut PersistedState) -> Result<GuardPointStatus> {
    let files = list_target_task_files()?;

    for file in files {
        patch_task_command(&file, false, state)?;
        let scheduler_name = scheduler_task_name_from_file(&file);
        let _ = run_command("schtasks", &["/Change", "/TN", &scheduler_name, "/ENABLE"]);
    }

    check(state)
}

fn list_target_task_files() -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for root in task_root_paths() {
        collect_files_recursively(&root, &mut files)?;
    }

    Ok(files)
}

fn collect_files_recursively(root: &Path, output: &mut Vec<PathBuf>) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }

    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            collect_files_recursively(&path, output)?;
        } else {
            output.push(path);
        }
    }

    Ok(())
}

fn scheduler_task_name_from_file(path: &Path) -> String {
    let lower = path.to_string_lossy().replace('/', "\\");
    let marker = "\\System32\\Tasks\\";
    if let Some(pos) = lower.find(marker) {
        let tail = &lower[pos + marker.len()..];
        return format!("\\{}", tail);
    }

    let name = path
        .file_name()
        .map(|item| item.to_string_lossy().to_string())
        .unwrap_or_else(|| String::from("unknown"));
    format!("\\{name}")
}

fn check_task_disabled(task_name: &str) -> bool {
    let output = run_command(
        "schtasks",
        &["/Query", "/TN", task_name, "/FO", "LIST", "/V"],
    );
    if let Ok(text) = output {
        let lower = text.to_ascii_lowercase();
        return lower.contains("disabled") || text.contains("已禁用");
    }

    false
}

fn task_command_prefixed(path: &Path) -> Result<bool> {
    let content = read_task_file(path)?;

    let mut saw_command = false;
    let mut all_prefixed = true;

    for segment in content.text.split("<Command>").skip(1) {
        if let Some((value, _)) = segment.split_once("</Command>") {
            saw_command = true;
            let trimmed = value.trim();
            if !trimmed.starts_with(DISABLE_PREFIX) {
                all_prefixed = false;
            }
        }
    }

    Ok(saw_command && all_prefixed)
}

fn patch_task_command(path: &Path, guard_mode: bool, state: &mut PersistedState) -> Result<()> {
    let key = path.to_string_lossy().to_string();
    let original = read_task_file(path)?;
    let updated = if guard_mode {
        prefix_task_commands(&original.text)
    } else {
        restore_task_commands(&original.text, state.task_command_backup.get(&key))
    };

    if guard_mode {
        state
            .task_command_backup
            .entry(key.clone())
            .or_insert(original.text.clone());
    }

    write_task_file(path, &updated, original.utf16le)
}

fn prefix_task_commands(content: &str) -> String {
    let mut result = String::new();
    let mut remaining = content;

    while let Some(start) = remaining.find("<Command>") {
        let before = &remaining[..start + "<Command>".len()];
        result.push_str(before);
        remaining = &remaining[start + "<Command>".len()..];

        if let Some(end) = remaining.find("</Command>") {
            let command = &remaining[..end];
            let trimmed = command.trim();
            if trimmed.starts_with(DISABLE_PREFIX) {
                result.push_str(command);
            } else {
                result.push_str(&format!("{DISABLE_PREFIX}{command}"));
            }
            result.push_str("</Command>");
            remaining = &remaining[end + "</Command>".len()..];
        } else {
            result.push_str(remaining);
            remaining = "";
            break;
        }
    }

    result.push_str(remaining);
    result
}

fn restore_task_commands(current: &str, backup: Option<&String>) -> String {
    if let Some(content) = backup {
        return content.clone();
    }

    current.replace("<Command>DISABLE:", "<Command>")
}

struct TaskContent {
    text: String,
    utf16le: bool,
}

fn read_task_file(path: &Path) -> Result<TaskContent> {
    let bytes = std::fs::read(path)?;

    if bytes.starts_with(&[0xFF, 0xFE]) {
        let mut units = Vec::new();
        for chunk in bytes[2..].chunks(2) {
            if chunk.len() == 2 {
                units.push(u16::from_le_bytes([chunk[0], chunk[1]]));
            }
        }
        return Ok(TaskContent {
            text: String::from_utf16_lossy(&units),
            utf16le: true,
        });
    }

    Ok(TaskContent {
        text: String::from_utf8_lossy(&bytes).to_string(),
        utf16le: false,
    })
}

fn write_task_file(path: &Path, content: &str, utf16le: bool) -> Result<()> {
    if utf16le {
        let mut bytes = vec![0xFF, 0xFE];
        for unit in content.encode_utf16() {
            let pair = unit.to_le_bytes();
            bytes.extend_from_slice(&pair);
        }
        std::fs::write(path, bytes)?;
        return Ok(());
    }

    std::fs::write(path, content)?;
    Ok(())
}
