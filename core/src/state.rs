use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::pathing::state_file_path;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PersistedState {
    pub service_start_backup: HashMap<String, u32>,
    pub service_image_path_backup: HashMap<String, String>,
    pub task_command_backup: HashMap<String, String>,
    pub desired_guarded: HashMap<String, bool>,
}

pub fn load_state() -> Result<PersistedState> {
    let path = state_file_path()?;

    if !path.exists() {
        return Ok(PersistedState::default());
    }

    let content = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content).unwrap_or_default())
}

pub fn save_state(state: &PersistedState) -> Result<()> {
    let path = state_file_path()?;
    let content = serde_json::to_string_pretty(state)?;
    std::fs::write(path, content)?;
    Ok(())
}
