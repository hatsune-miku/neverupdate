use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuardAction {
    Guard,
    Release,
    Repair,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardPointDefinition {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardPointStatus {
    pub id: String,
    pub title: String,
    pub guarded: bool,
    pub breached: bool,
    pub message: Option<String>,
    pub checked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardSummary {
    pub statuses: Vec<GuardPointStatus>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightCheck {
    pub id: String,
    pub title: String,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightReport {
    pub passed: bool,
    pub checks: Vec<PreflightCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub point_id: String,
    pub action: GuardAction,
    pub success: bool,
    pub timestamp: DateTime<Utc>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonRuntimeStatus {
    pub running: bool,
    pub service_registered: bool,
    pub service_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonSnapshot {
    pub timestamp: DateTime<Utc>,
    pub statuses: Vec<GuardPointStatus>,
    pub message: Option<String>,
    pub runtime: DaemonRuntimeStatus,
}
