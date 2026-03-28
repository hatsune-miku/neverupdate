mod extreme_guard;
mod fallback_guard;
mod hosts_firewall_guard;
mod policy_guard;
pub(crate) mod service_guard;
mod task_guard;

use chrono::Utc;

use crate::error::{NuError, Result};
use crate::model::{GuardAction, GuardPointDefinition, GuardPointStatus};
use crate::state::PersistedState;
use crate::ti_service::TiService;

pub trait GuardPoint: Send + Sync {
    fn id(&self) -> &'static str;
    fn title(&self) -> &'static str;
    fn description(&self) -> &'static str;

    fn check(&self, state: &PersistedState) -> Result<GuardPointStatus>;
    fn guard(&self, state: &mut PersistedState, ti: &TiService) -> Result<GuardPointStatus>;
    fn release(&self, state: &mut PersistedState, ti: &TiService) -> Result<GuardPointStatus>;

    fn batch_eligible(&self) -> bool {
        true
    }

    fn tracks_desired(&self) -> bool {
        true
    }

    fn interception_behavior(&self) -> Option<&'static str> {
        None
    }

    fn build_status(&self, guarded: bool, message: Option<String>) -> GuardPointStatus {
        GuardPointStatus {
            id: self.id().to_string(),
            title: self.title().to_string(),
            guarded,
            breached: !guarded,
            message,
            checked_at: Utc::now(),
        }
    }

    fn definition(&self) -> GuardPointDefinition {
        GuardPointDefinition {
            id: self.id(),
            title: self.title(),
            description: self.description(),
        }
    }
}

pub fn registry() -> Vec<Box<dyn GuardPoint>> {
    vec![
        Box::new(service_guard::ServiceGuard),
        Box::new(hosts_firewall_guard::HostsFirewallGuard),
        Box::new(policy_guard::PolicyGuard),
        Box::new(task_guard::TaskGuard),
        Box::new(fallback_guard::FallbackGuard),
        Box::new(extreme_guard::ExtremeGuard),
    ]
}

pub fn definitions() -> Vec<GuardPointDefinition> {
    registry().iter().map(|g| g.definition()).collect()
}

pub fn check_guard(guard: &dyn GuardPoint, state: &PersistedState) -> Result<GuardPointStatus> {
    let status = guard.check(state)?;
    Ok(normalize_status(guard, state, status))
}

pub fn run_point_action(
    point_id: &str,
    action: GuardAction,
    state: &mut PersistedState,
    ti: &TiService,
) -> Result<GuardPointStatus> {
    let guards = registry();
    let guard = guards
        .iter()
        .find(|g| g.id() == point_id)
        .ok_or_else(|| NuError::InvalidOperation(format!("unknown point id: {point_id}")))?;
    execute_action(guard.as_ref(), action, state, ti)
}

pub fn execute_action(
    guard: &dyn GuardPoint,
    action: GuardAction,
    state: &mut PersistedState,
    ti: &TiService,
) -> Result<GuardPointStatus> {
    if guard.tracks_desired() {
        let desired = matches!(action, GuardAction::Guard | GuardAction::Repair);
        state
            .desired_guarded
            .insert(guard.id().to_string(), desired);
    }

    let status = match action {
        GuardAction::Guard | GuardAction::Repair => guard.guard(state, ti),
        GuardAction::Release => guard.release(state, ti),
    }?;

    Ok(normalize_status(guard, state, status))
}

fn normalize_status(
    guard: &dyn GuardPoint,
    state: &PersistedState,
    mut status: GuardPointStatus,
) -> GuardPointStatus {
    if !guard.tracks_desired() {
        status.breached = false;
        return status;
    }

    let expected = state
        .desired_guarded
        .get(guard.id())
        .copied()
        .unwrap_or(true);
    status.breached = expected && !status.guarded;
    status
}
