use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::guards::GuardPoint;
use crate::model::GuardPointStatus;
use crate::pathing::task_root_paths;
use crate::state::PersistedState;
use crate::ti_service::TiService;

const DISABLE_PREFIX: &str = "DISABLE:";

pub struct TaskGuard;

impl GuardPoint for TaskGuard {
    fn id(&self) -> &'static str {
        "scheduled_tasks"
    }

    fn title(&self) -> &'static str {
        "计划任务"
    }

    fn description(&self) -> &'static str {
        "禁用 UpdateOrchestrator/WaaSMedic 任务并修改 Command"
    }

    fn interception_behavior(&self) -> Option<&'static str> {
        Some("系统试图重建更新计划任务")
    }

    fn check(&self, _state: &PersistedState) -> Result<GuardPointStatus> {
        let files = list_target_task_files();
        if files.is_empty() {
            return Ok(self.build_status(
                true,
                Some(String::from("no task files found (safe)")),
            ));
        }

        let mut all_guarded = true;
        let mut details = Vec::new();

        for file in files {
            let name = scheduler_task_name(&file);
            match read_task_file_direct(&file) {
                Ok(content) => {
                    let disabled = is_xml_disabled(&content.text);
                    let has_cmd = content.text.contains("<Command>");
                    let prefixed = if has_cmd { is_commands_prefixed(&content.text) } else { true };
                    let guarded = disabled && prefixed;
                    all_guarded = all_guarded && guarded;
                    details.push(format!("{name}: disabled={disabled}, prefixed={prefixed}"));
                }
                Err(e) => {
                    all_guarded = false;
                    details.push(format!("{name}: read error: {e}"));
                }
            }
        }

        Ok(self.build_status(all_guarded, Some(details.join(" | "))))
    }

    fn guard(&self, state: &mut PersistedState, ti: &TiService) -> Result<GuardPointStatus> {
        let mut errors = Vec::new();
        for file in list_target_task_files() {
            if let Err(e) = patch_task_file(&file, true, state, ti) {
                errors.push(format!("{}: {e}", scheduler_task_name(&file)));
            }
        }
        let mut status = ti.as_admin(|| self.check(state))?;
        if !errors.is_empty() {
            let prev = status.message.unwrap_or_default();
            status.message = Some(format!("{prev} | ERRORS: {}", errors.join("; ")));
        }
        Ok(status)
    }

    fn release(&self, state: &mut PersistedState, ti: &TiService) -> Result<GuardPointStatus> {
        let mut errors = Vec::new();
        for file in list_target_task_files() {
            if let Err(e) = patch_task_file(&file, false, state, ti) {
                errors.push(format!("{}: {e}", scheduler_task_name(&file)));
            }
        }
        let mut status = ti.as_admin(|| self.check(state))?;
        if !errors.is_empty() {
            let prev = status.message.unwrap_or_default();
            status.message = Some(format!("{prev} | ERRORS: {}", errors.join("; ")));
        }
        Ok(status)
    }
}

fn list_target_task_files() -> Vec<PathBuf> {
    let mut out = Vec::new();
    for root in task_root_paths() {
        let _ = collect_files(&root, &mut out);
    }
    out
}

fn collect_files(root: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_files(&path, out)?;
        } else {
            out.push(path);
        }
    }
    Ok(())
}

fn scheduler_task_name(path: &Path) -> String {
    let s = path.to_string_lossy().replace('/', "\\");
    let marker = "\\System32\\Tasks\\";
    if let Some(pos) = s.find(marker) {
        return format!("\\{}", &s[pos + marker.len()..]);
    }
    let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| String::from("unknown"));
    format!("\\{name}")
}

// ── XML content checks (no external commands) ──

fn is_xml_disabled(xml: &str) -> bool {
    xml.contains("<Enabled>false</Enabled>")
}

fn is_commands_prefixed(xml: &str) -> bool {
    let mut saw = false;
    for seg in xml.split("<Command>").skip(1) {
        if let Some((val, _)) = seg.split_once("</Command>") {
            saw = true;
            if !val.trim().starts_with(DISABLE_PREFIX) {
                return false;
            }
        }
    }
    saw
}

// ── XML patching (all via TI file I/O, no schtasks) ──

fn patch_task_file(path: &Path, guard_mode: bool, state: &mut PersistedState, ti: &TiService) -> Result<()> {
    let key = path.to_string_lossy().to_string();
    let bytes = ti.read_file(path)?;
    let original = decode_task_bytes(&bytes);
    let utf16le = bytes.starts_with(&[0xFF, 0xFE]);

    let updated = if guard_mode {
        let s = prefix_commands(&original);
        set_xml_enabled(&s, false)
    } else {
        let s = restore_commands(&original, state.task_command_backup.get(&key));
        set_xml_enabled(&s, true)
    };

    if guard_mode {
        state.task_command_backup.entry(key).or_insert(original);
    }

    let out_bytes = if utf16le {
        encode_utf16le(&updated)
    } else {
        updated.into_bytes()
    };

    ti.write_file(path, &out_bytes)
}

fn set_xml_enabled(xml: &str, enabled: bool) -> String {
    let val = if enabled { "true" } else { "false" };
    let target = format!("<Enabled>{val}</Enabled>");

    if xml.contains("<Enabled>true</Enabled>") {
        return xml.replace("<Enabled>true</Enabled>", &target);
    }
    if xml.contains("<Enabled>false</Enabled>") {
        return xml.replace("<Enabled>false</Enabled>", &target);
    }
    // No <Enabled> tag — insert after <Settings>
    if let Some(pos) = xml.find("<Settings>") {
        let insert = pos + "<Settings>".len();
        return format!("{}\n    {}{}", &xml[..insert], target, &xml[insert..]);
    }
    xml.to_string()
}

fn prefix_commands(content: &str) -> String {
    let mut result = String::new();
    let mut rem = content;
    while let Some(start) = rem.find("<Command>") {
        result.push_str(&rem[..start + "<Command>".len()]);
        rem = &rem[start + "<Command>".len()..];
        if let Some(end) = rem.find("</Command>") {
            let cmd = &rem[..end];
            if cmd.trim().starts_with(DISABLE_PREFIX) {
                result.push_str(cmd);
            } else {
                result.push_str(&format!("{DISABLE_PREFIX}{cmd}"));
            }
            result.push_str("</Command>");
            rem = &rem[end + "</Command>".len()..];
        } else {
            result.push_str(rem);
            rem = "";
            break;
        }
    }
    result.push_str(rem);
    result
}

fn restore_commands(current: &str, backup: Option<&String>) -> String {
    if let Some(b) = backup {
        return b.clone();
    }
    current.replace("<Command>DISABLE:", "<Command>")
}

struct TaskContent { text: String }

fn read_task_file_direct(path: &Path) -> Result<TaskContent> {
    let bytes = std::fs::read(path)?;
    Ok(TaskContent { text: decode_task_bytes(&bytes) })
}

fn decode_task_bytes(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0xFF, 0xFE]) {
        let units: Vec<u16> = bytes[2..].chunks(2)
            .filter_map(|c| (c.len() == 2).then(|| u16::from_le_bytes([c[0], c[1]])))
            .collect();
        String::from_utf16_lossy(&units)
    } else {
        String::from_utf8_lossy(bytes).to_string()
    }
}

fn encode_utf16le(s: &str) -> Vec<u8> {
    let mut out = vec![0xFF, 0xFE];
    for unit in s.encode_utf16() {
        out.extend_from_slice(&unit.to_le_bytes());
    }
    out
}
