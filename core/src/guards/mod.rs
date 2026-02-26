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

const POINT_SERVICE: GuardPointDefinition = GuardPointDefinition {
    id: "service_watchdog",
    title: "更新干扰服务",
    description: "禁用 WaaSMedicSvc / UsoSvc / uhssvc，并将 ImagePath 前置 DISABLE:",
};

const POINT_HOSTS_FIREWALL: GuardPointDefinition = GuardPointDefinition {
    id: "hosts_firewall",
    title: "Hosts 与防火墙",
    description: "锁定更新域名到 127.0.0.1，并维护防火墙 FQDN 拒绝规则",
};

const POINT_POLICY: GuardPointDefinition = GuardPointDefinition {
    id: "group_policy",
    title: "组策略",
    description: "配置自动更新=禁用，删除访问更新功能=启用",
};

const POINT_TASKS: GuardPointDefinition = GuardPointDefinition {
    id: "scheduled_tasks",
    title: "计划任务",
    description: "禁用 UpdateOrchestrator/WaaSMedic 任务并修改 Command",
};

const POINT_FALLBACK: GuardPointDefinition = GuardPointDefinition {
    id: "fallback_settings",
    title: "兜底更新设置",
    description: "长暂停更新、禁驱动/可选更新、抑制自动重启",
};

const POINT_EXTREME: GuardPointDefinition = GuardPointDefinition {
    id: "extreme_mode",
    title: "极端手段",
    description: "彻底破坏 Windows Update 数据目录（不参与一键操作，执行要求二次确认）",
};

const DEFINITIONS: [GuardPointDefinition; 6] = [
    POINT_SERVICE,
    POINT_HOSTS_FIREWALL,
    POINT_POLICY,
    POINT_TASKS,
    POINT_FALLBACK,
    POINT_EXTREME,
];

pub fn definitions() -> Vec<GuardPointDefinition> {
    DEFINITIONS.to_vec()
}

pub fn check_point(point_id: &str, state: &PersistedState) -> Result<GuardPointStatus> {
    let status = match point_id {
        "service_watchdog" => service_guard::check(state),
        "hosts_firewall" => hosts_firewall_guard::check(),
        "group_policy" => policy_guard::check(),
        "scheduled_tasks" => task_guard::check(state),
        "fallback_settings" => fallback_guard::check(),
        "extreme_mode" => extreme_guard::check(),
        other => Err(NuError::InvalidOperation(format!(
            "unknown point id: {other}"
        ))),
    }?;

    Ok(normalize_status(point_id, state, status))
}

pub fn run_point_action(
    point_id: &str,
    action: GuardAction,
    state: &mut PersistedState,
) -> Result<GuardPointStatus> {
    let known = DEFINITIONS.iter().any(|item| item.id == point_id);
    if !known {
        return Err(NuError::InvalidOperation(format!(
            "unknown point id: {point_id}"
        )));
    }

    if point_id != "extreme_mode" {
        let desired_guarded = match action {
            GuardAction::Guard | GuardAction::Repair => true,
            GuardAction::Release => false,
        };
        state
            .desired_guarded
            .insert(point_id.to_string(), desired_guarded);
    }

    let status =
        match (point_id, action) {
            ("service_watchdog", GuardAction::Guard)
            | ("service_watchdog", GuardAction::Repair) => service_guard::guard(state),
            ("service_watchdog", GuardAction::Release) => service_guard::release(state),

            ("hosts_firewall", GuardAction::Guard) | ("hosts_firewall", GuardAction::Repair) => {
                hosts_firewall_guard::guard()
            }
            ("hosts_firewall", GuardAction::Release) => hosts_firewall_guard::release(),

            ("group_policy", GuardAction::Guard) | ("group_policy", GuardAction::Repair) => {
                policy_guard::guard()
            }
            ("group_policy", GuardAction::Release) => policy_guard::release(),

            ("scheduled_tasks", GuardAction::Guard) | ("scheduled_tasks", GuardAction::Repair) => {
                task_guard::guard(state)
            }
            ("scheduled_tasks", GuardAction::Release) => task_guard::release(state),

            ("fallback_settings", GuardAction::Guard)
            | ("fallback_settings", GuardAction::Repair) => fallback_guard::guard(),
            ("fallback_settings", GuardAction::Release) => fallback_guard::release(),

            ("extreme_mode", GuardAction::Guard) | ("extreme_mode", GuardAction::Repair) => {
                extreme_guard::guard()
            }
            ("extreme_mode", GuardAction::Release) => extreme_guard::release(),

            (other, _) => Err(NuError::InvalidOperation(format!(
                "unknown point id: {other}"
            ))),
        }?;

    Ok(normalize_status(point_id, state, status))
}

fn normalize_status(
    point_id: &str,
    state: &PersistedState,
    mut status: GuardPointStatus,
) -> GuardPointStatus {
    if point_id == "extreme_mode" {
        status.breached = false;
        return status;
    }

    let expected_guarded = state.desired_guarded.get(point_id).copied().unwrap_or(true);
    status.breached = expected_guarded && !status.guarded;
    status
}

fn build_status(
    def: GuardPointDefinition,
    guarded: bool,
    message: Option<String>,
) -> GuardPointStatus {
    GuardPointStatus {
        id: def.id.to_string(),
        title: def.title.to_string(),
        guarded,
        breached: !guarded,
        message,
        checked_at: Utc::now(),
    }
}

pub fn status_service(guarded: bool, message: Option<String>) -> GuardPointStatus {
    build_status(POINT_SERVICE, guarded, message)
}

pub fn status_hosts_firewall(guarded: bool, message: Option<String>) -> GuardPointStatus {
    build_status(POINT_HOSTS_FIREWALL, guarded, message)
}

pub fn status_policy(guarded: bool, message: Option<String>) -> GuardPointStatus {
    build_status(POINT_POLICY, guarded, message)
}

pub fn status_tasks(guarded: bool, message: Option<String>) -> GuardPointStatus {
    build_status(POINT_TASKS, guarded, message)
}

pub fn status_fallback(guarded: bool, message: Option<String>) -> GuardPointStatus {
    build_status(POINT_FALLBACK, guarded, message)
}

pub fn status_extreme(guarded: bool, message: Option<String>) -> GuardPointStatus {
    build_status(POINT_EXTREME, guarded, message)
}
