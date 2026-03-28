use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ};
use winreg::RegKey;

use crate::error::Result;
use crate::guards::GuardPoint;
use crate::model::GuardPointStatus;
use crate::state::PersistedState;
use crate::ti_service::TiService;

const DISABLE_PREFIX: &str = "DISABLE:";
const SERVICES: [&str; 3] = ["WaaSMedicSvc", "UsoSvc", "uhssvc"];
const EXTRA_SERVICE: &str = "wuauserv";

pub struct ServiceGuard;

impl GuardPoint for ServiceGuard {
    fn id(&self) -> &'static str {
        "service_watchdog"
    }

    fn title(&self) -> &'static str {
        "更新干扰服务"
    }

    fn description(&self) -> &'static str {
        "禁用 WaaSMedicSvc / UsoSvc / uhssvc，并将 ImagePath 前置 DISABLE:"
    }

    fn interception_behavior(&self) -> Option<&'static str> {
        Some("系统试图重建更新服务")
    }

    fn check(&self, _state: &PersistedState) -> Result<GuardPointStatus> {
        let mut guarded = true;
        let mut detail = Vec::new();

        for service in SERVICES {
            let (ok, d) = check_single_service(service);
            guarded = guarded && ok;
            detail.push(d);
        }

        Ok(self.build_status(guarded, Some(detail.join(" | "))))
    }

    fn guard(&self, state: &mut PersistedState, ti: &TiService) -> Result<GuardPointStatus> {
        let mut errors = Vec::new();
        for service in SERVICES {
            if let Err(e) = guard_single_service(service, state, ti) {
                errors.push(format!("{service}: {e}"));
            }
        }
        let mut status = ti.as_admin(|| self.check(state))?;
        if !errors.is_empty() {
            let prev = status.message.unwrap_or_default();
            status.message = Some(format!("{prev} | WRITE_ERR: {}", errors.join("; ")));
        }
        Ok(status)
    }

    fn release(&self, state: &mut PersistedState, ti: &TiService) -> Result<GuardPointStatus> {
        let mut errors = Vec::new();
        for service in SERVICES {
            if let Err(e) = release_single_service(service, state, ti) {
                errors.push(format!("{service}: {e}"));
            }
        }
        let mut status = ti.as_admin(|| self.check(state))?;
        if !errors.is_empty() {
            let prev = status.message.unwrap_or_default();
            status.message = Some(format!("{prev} | WRITE_ERR: {}", errors.join("; ")));
        }
        Ok(status)
    }
}

pub fn extreme_disable_all_services(state: &mut PersistedState, ti: &TiService) {
    for service in SERVICES.iter().chain([EXTRA_SERVICE].iter()) {
        let _ = guard_single_service(service, state, ti);
    }
}

fn service_key_path(name: &str) -> String {
    format!("SYSTEM\\CurrentControlSet\\Services\\{name}")
}

fn service_exists(name: &str) -> bool {
    RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(service_key_path(name), KEY_READ)
        .is_ok()
}

fn check_single_service(name: &str) -> (bool, String) {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = match hklm.open_subkey_with_flags(service_key_path(name), KEY_READ) {
        Ok(k) => k,
        Err(_) => return (true, format!("{name}: not installed")),
    };

    let n_start: u32 = key.get_value("Start").unwrap_or(3);
    let img: String = key.get_value("ImagePath").unwrap_or_default();
    let guarded = n_start == 4 && img.starts_with(DISABLE_PREFIX);

    (
        guarded,
        format!("{name}: Start={n_start}, Prefixed={}", img.starts_with(DISABLE_PREFIX)),
    )
}

fn guard_single_service(name: &str, state: &mut PersistedState, ti: &TiService) -> Result<()> {
    if !service_exists(name) {
        return Ok(());
    }
    let key = ti.open_key(&service_key_path(name), true)?;

    let n_start: u32 = key.get_value("Start").unwrap_or(3);
    state.service_start_backup.entry(name.to_string()).or_insert(n_start);
    key.set_value("Start", &4u32)?;

    let img: String = key.get_value("ImagePath").unwrap_or_default();
    if !img.is_empty() && !img.starts_with(DISABLE_PREFIX) {
        state.service_image_path_backup.entry(name.to_string()).or_insert(img.clone());
        key.set_value("ImagePath", &format!("{DISABLE_PREFIX}{img}"))?;
    }
    Ok(())
}

fn release_single_service(name: &str, state: &mut PersistedState, ti: &TiService) -> Result<()> {
    if !service_exists(name) {
        return Ok(());
    }
    let key = ti.open_key(&service_key_path(name), true)?;

    if let Some(v) = state.service_start_backup.get(name) {
        key.set_value("Start", v)?;
    } else {
        key.set_value("Start", &3u32)?;
    }

    let img: String = key.get_value("ImagePath").unwrap_or_default();
    if let Some(original) = state.service_image_path_backup.get(name) {
        key.set_value("ImagePath", original)?;
    } else if img.starts_with(DISABLE_PREFIX) {
        key.set_value("ImagePath", &img.trim_start_matches(DISABLE_PREFIX))?;
    }
    Ok(())
}
