mod command;
mod daemon_service;
mod error;
mod extreme;
mod guards;
mod history;
mod model;
mod pathing;
pub(crate) mod ti_service;
mod state;
mod system_check;

pub use daemon_service::{
    daemon_service_exists, daemon_service_name, daemon_service_running, register_daemon_service,
    reregister_daemon_service, start_daemon_service, stop_daemon_service, unregister_daemon_service,
};
pub use error::{NuError, Result};
pub use model::{
    DaemonRuntimeStatus, DaemonSnapshot, GuardAction, GuardPointDefinition, GuardPointStatus,
    GuardSummary, HistoryEntry, InterceptionEntry, PreflightCheck, PreflightReport,
};
pub use system_check::{
    acquire_global_instance, has_admin_privilege, has_privileged_session, run_preflight_checks,
    verify_admin_writable,
};

use chrono::Utc;
use ti_service::TiService;

pub fn list_guard_points() -> Vec<GuardPointDefinition> {
    guards::definitions()
}

pub fn query_guard_states() -> Result<Vec<GuardPointStatus>> {
    let state = state::load_state()?;
    guards::registry()
        .iter()
        .map(|guard| guards::check_guard(guard.as_ref(), &state))
        .collect()
}

pub fn execute_guard_action(point_id: &str, action: GuardAction) -> Result<GuardPointStatus> {
    let ti = TiService::acquire()?;
    let mut state = state::load_state()?;
    let result = guards::run_point_action(point_id, action, &mut state, &ti);

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
    let ti = match TiService::acquire() {
        Ok(t) => t,
        Err(e) => {
            return GuardSummary {
                statuses: vec![],
                errors: vec![format!("TI acquire failed: {e}")],
            };
        }
    };

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

    for guard in guards::registry() {
        if !guard.batch_eligible() {
            continue;
        }

        let result = guards::execute_action(guard.as_ref(), action, &mut state, &ti);
        match result {
            Ok(status) => {
                let _ = history::append_history_entry(&HistoryEntry {
                    point_id: guard.id().to_string(),
                    action,
                    success: true,
                    timestamp: Utc::now(),
                    message: status.message.clone(),
                });
                statuses.push(status);
            }
            Err(error) => {
                let _ = history::append_history_entry(&HistoryEntry {
                    point_id: guard.id().to_string(),
                    action,
                    success: false,
                    timestamp: Utc::now(),
                    message: Some(error.to_string()),
                });
                errors.push(format!("{}: {}", guard.id(), error));
            }
        }
    }

    if let Err(error) = state::save_state(&state) {
        errors.push(format!("save state failed: {error}"));
    }

    GuardSummary { statuses, errors }
}

pub fn run_maintenance_cycle() -> GuardSummary {
    let ti = match TiService::acquire() {
        Ok(t) => t,
        Err(e) => {
            return GuardSummary {
                statuses: vec![],
                errors: vec![format!("TI acquire failed: {e}")],
            };
        }
    };

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

    for guard in guards::registry() {
        if !guard.batch_eligible() {
            continue;
        }

        let before = match guards::check_guard(guard.as_ref(), &state) {
            Ok(status) => status,
            Err(error) => {
                errors.push(format!("{}: pre-check failed: {}", guard.id(), error));
                continue;
            }
        };

        let result = guards::execute_action(guard.as_ref(), GuardAction::Guard, &mut state, &ti);
        match result {
            Ok(status) => {
                let _ = history::append_history_entry(&HistoryEntry {
                    point_id: guard.id().to_string(),
                    action: GuardAction::Guard,
                    success: true,
                    timestamp: Utc::now(),
                    message: status.message.clone(),
                });

                if !before.guarded {
                    if let Some(behavior) = guard.interception_behavior() {
                        let blocked = status.guarded && !status.breached;
                        if let Err(error) = history::append_interception_entry(&InterceptionEntry {
                            point_id: guard.id().to_string(),
                            behavior: behavior.to_string(),
                            blocked,
                            timestamp: Utc::now(),
                            message: status.message.clone(),
                        }) {
                            errors.push(format!("{}: interception append failed: {}", guard.id(), error));
                        }
                    }
                }

                statuses.push(status);
            }
            Err(error) => {
                let _ = history::append_history_entry(&HistoryEntry {
                    point_id: guard.id().to_string(),
                    action: GuardAction::Guard,
                    success: false,
                    timestamp: Utc::now(),
                    message: Some(error.to_string()),
                });

                if !before.guarded {
                    if let Some(behavior) = guard.interception_behavior() {
                        if let Err(write_error) = history::append_interception_entry(&InterceptionEntry {
                            point_id: guard.id().to_string(),
                            behavior: behavior.to_string(),
                            blocked: false,
                            timestamp: Utc::now(),
                            message: Some(error.to_string()),
                        }) {
                            errors.push(format!(
                                "{}: interception append failed: {}",
                                guard.id(),
                                write_error
                            ));
                        }
                    }
                }

                errors.push(format!("{}: {}", guard.id(), error));
            }
        }
    }

    if let Err(error) = state::save_state(&state) {
        errors.push(format!("save state failed: {error}"));
    }

    GuardSummary { statuses, errors }
}

pub fn read_history(limit: usize) -> Result<Vec<HistoryEntry>> {
    history::read_history(limit)
}

pub fn clear_history() -> Result<()> {
    history::clear_history()
}

pub fn read_interceptions(limit: usize) -> Result<Vec<InterceptionEntry>> {
    history::read_interceptions(limit)
}

pub fn clear_interceptions() -> Result<()> {
    history::clear_interceptions()
}

pub fn load_daemon_snapshot() -> Result<Option<DaemonSnapshot>> {
    let mut snap = match history::load_snapshot()? {
        Some(s) => s,
        None => DaemonSnapshot {
            timestamp: Utc::now(),
            statuses: vec![],
            message: None,
            runtime: DaemonRuntimeStatus {
                running: false,
                service_registered: false,
                service_name: String::new(),
            },
        },
    };
    snap.runtime = DaemonRuntimeStatus {
        running: daemon_service_running(),
        service_registered: daemon_service_exists(),
        service_name: daemon_service_name().to_string(),
    };
    Ok(Some(snap))
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
            running: daemon_service_running(),
            service_registered: daemon_service_exists(),
            service_name: daemon_service_name().to_string(),
        },
    };

    history::save_snapshot(&snapshot)
}

pub fn run_extreme_mode() -> Result<()> {
    let ti = TiService::acquire()?;
    let mut state = state::load_state()?;
    let result = extreme::run_extreme_mode(&mut state, &ti);

    match &result {
        Ok(()) => {
            let _ = state::save_state(&state);
            let _ = history::append_history_entry(&HistoryEntry {
                point_id: String::from("extreme_mode"),
                action: GuardAction::Guard,
                success: true,
                timestamp: Utc::now(),
                message: Some(String::from("extreme mode executed")),
            });
        }
        Err(error) => {
            let _ = state::save_state(&state);
            let _ = history::append_history_entry(&HistoryEntry {
                point_id: String::from("extreme_mode"),
                action: GuardAction::Guard,
                success: false,
                timestamp: Utc::now(),
                message: Some(error.to_string()),
            });
        }
    }

    result
}
