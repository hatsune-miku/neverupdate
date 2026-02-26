mod command;
mod daemon_service;
mod error;
mod extreme;
mod guards;
mod history;
mod model;
mod pathing;
mod state;
mod system_check;

pub use daemon_service::{
    daemon_service_exists, daemon_service_name, register_daemon_service, reregister_daemon_service,
    start_daemon_service, stop_daemon_service, unregister_daemon_service,
};
pub use error::{NuError, Result};
pub use model::{
    DaemonRuntimeStatus, DaemonSnapshot, GuardAction, GuardPointDefinition, GuardPointStatus,
    GuardSummary, HistoryEntry, PreflightCheck, PreflightReport,
};
pub use system_check::{
    acquire_global_instance, has_admin_privilege, run_preflight_checks, verify_admin_writable,
};

use chrono::Utc;

pub fn list_guard_points() -> Vec<GuardPointDefinition> {
    guards::definitions()
}

pub fn query_guard_states() -> Result<Vec<GuardPointStatus>> {
    let state = state::load_state()?;
    guards::definitions()
        .iter()
        .map(|point| guards::check_point(point.id, &state))
        .collect()
}

pub fn execute_guard_action(point_id: &str, action: GuardAction) -> Result<GuardPointStatus> {
    let mut state = state::load_state()?;
    let result = guards::run_point_action(point_id, action, &mut state);

    match result {
        Ok(status) => {
            state::save_state(&state)?;
            history::append_history_entry(&HistoryEntry {
                point_id: point_id.to_string(),
                action,
                success: true,
                timestamp: Utc::now(),
                message: status.message.clone(),
            })?;
            Ok(status)
        }
        Err(err) => {
            history::append_history_entry(&HistoryEntry {
                point_id: point_id.to_string(),
                action,
                success: false,
                timestamp: Utc::now(),
                message: Some(err.to_string()),
            })?;
            Err(err)
        }
    }
}

pub fn execute_all(action: GuardAction) -> GuardSummary {
    let mut state = match state::load_state() {
        Ok(value) => value,
        Err(error) => {
            return GuardSummary {
                statuses: vec![],
                errors: vec![format!("failed to load state: {error}")],
            };
        }
    };

    let mut statuses = Vec::new();
    let mut errors = Vec::new();

    for point in guards::definitions() {
        if point.id == "extreme_mode" {
            continue;
        }

        let result = guards::run_point_action(point.id, action, &mut state);
        match result {
            Ok(status) => {
                let _ = history::append_history_entry(&HistoryEntry {
                    point_id: point.id.to_string(),
                    action,
                    success: true,
                    timestamp: Utc::now(),
                    message: status.message.clone(),
                });
                statuses.push(status);
            }
            Err(error) => {
                let _ = history::append_history_entry(&HistoryEntry {
                    point_id: point.id.to_string(),
                    action,
                    success: false,
                    timestamp: Utc::now(),
                    message: Some(error.to_string()),
                });
                errors.push(format!("{}: {}", point.id, error));
            }
        }
    }

    if let Err(error) = state::save_state(&state) {
        errors.push(format!("save state failed: {error}"));
    }

    GuardSummary { statuses, errors }
}

pub fn run_maintenance_cycle() -> GuardSummary {
    execute_all(GuardAction::Guard)
}

pub fn read_history(limit: usize) -> Result<Vec<HistoryEntry>> {
    history::read_history(limit)
}

pub fn load_daemon_snapshot() -> Result<Option<DaemonSnapshot>> {
    history::load_snapshot()
}

pub fn store_daemon_snapshot(
    statuses: Vec<GuardPointStatus>,
    message: Option<String>,
) -> Result<()> {
    let snapshot = DaemonSnapshot {
        timestamp: Utc::now(),
        statuses,
        message,
        runtime: DaemonRuntimeStatus {
            running: true,
            service_registered: daemon_service_exists(),
            service_name: daemon_service_name().to_string(),
        },
    };

    history::save_snapshot(&snapshot)
}

pub fn run_extreme_mode() -> Result<()> {
    let result = extreme::run_extreme_mode();

    match result {
        Ok(()) => {
            let _ = history::append_history_entry(&HistoryEntry {
                point_id: String::from("extreme_mode"),
                action: GuardAction::Guard,
                success: true,
                timestamp: Utc::now(),
                message: Some(String::from("extreme mode executed")),
            });
            Ok(())
        }
        Err(error) => {
            let _ = history::append_history_entry(&HistoryEntry {
                point_id: String::from("extreme_mode"),
                action: GuardAction::Guard,
                success: false,
                timestamp: Utc::now(),
                message: Some(error.to_string()),
            });
            Err(error)
        }
    }
}
