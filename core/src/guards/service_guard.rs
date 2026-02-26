use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE};
use winreg::RegKey;

use crate::error::Result;
use crate::guards::status_service;
use crate::model::GuardPointStatus;
use crate::state::PersistedState;

const DISABLE_PREFIX: &str = "DISABLE:";
const SERVICES: [&str; 3] = ["WaaSMedicSvc", "UsoSvc", "uhssvc"];
const EXTRA_SERVICE: &str = "wuauserv";

pub fn check(_state: &PersistedState) -> Result<GuardPointStatus> {
    let mut guarded = true;
    let mut detail = Vec::new();

    for service in SERVICES {
        let (item_guarded, item_detail) = check_single_service(service)?;
        guarded = guarded && item_guarded;
        detail.push(item_detail);
    }

    Ok(status_service(guarded, Some(detail.join(" | "))))
}

pub fn guard(state: &mut PersistedState) -> Result<GuardPointStatus> {
    for service in SERVICES {
        guard_single_service(service, state)?;
    }

    check(state)
}

pub fn release(state: &mut PersistedState) -> Result<GuardPointStatus> {
    for service in SERVICES {
        release_single_service(service, state)?;
    }

    check(state)
}

pub fn extreme_disable_all_services(state: &mut PersistedState) -> Result<()> {
    for service in SERVICES.iter().chain([EXTRA_SERVICE].iter()) {
        guard_single_service(service, state)?;
    }

    Ok(())
}

fn open_service_key(service_name: &str, writable: bool) -> Result<RegKey> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let path = format!("SYSTEM\\CurrentControlSet\\Services\\{service_name}");
    let flags = if writable {
        KEY_READ | KEY_WRITE
    } else {
        KEY_READ
    };
    Ok(hklm.open_subkey_with_flags(path, flags)?)
}

fn check_single_service(service_name: &str) -> Result<(bool, String)> {
    let key = match open_service_key(service_name, false) {
        Ok(value) => value,
        Err(_) => {
            return Ok((false, format!("{service_name}: service key not found")));
        }
    };

    let n_start: u32 = key.get_value("Start").unwrap_or(3);
    let s_image_path: String = key.get_value("ImagePath").unwrap_or_default();
    let guarded = n_start == 4 && s_image_path.starts_with(DISABLE_PREFIX);

    Ok((
        guarded,
        format!(
            "{service_name}: Start={n_start}, ImagePathPrefixed={}",
            s_image_path.starts_with(DISABLE_PREFIX)
        ),
    ))
}

fn guard_single_service(service_name: &str, state: &mut PersistedState) -> Result<()> {
    let key = open_service_key(service_name, true)?;

    let n_start: u32 = key.get_value("Start").unwrap_or(3);
    state
        .service_start_backup
        .entry(service_name.to_string())
        .or_insert(n_start);
    key.set_value("Start", &4u32)?;

    let s_image_path: String = key.get_value("ImagePath").unwrap_or_default();
    if !s_image_path.is_empty() && !s_image_path.starts_with(DISABLE_PREFIX) {
        state
            .service_image_path_backup
            .entry(service_name.to_string())
            .or_insert(s_image_path.clone());
        key.set_value("ImagePath", &format!("{DISABLE_PREFIX}{s_image_path}"))?;
    }

    Ok(())
}

fn release_single_service(service_name: &str, state: &mut PersistedState) -> Result<()> {
    let key = open_service_key(service_name, true)?;

    if let Some(n_start) = state.service_start_backup.get(service_name) {
        key.set_value("Start", n_start)?;
    } else {
        key.set_value("Start", &3u32)?;
    }

    let s_image_path: String = key.get_value("ImagePath").unwrap_or_default();
    if let Some(original) = state.service_image_path_backup.get(service_name) {
        key.set_value("ImagePath", original)?;
    } else if s_image_path.starts_with(DISABLE_PREFIX) {
        let restored = s_image_path.trim_start_matches(DISABLE_PREFIX).to_string();
        key.set_value("ImagePath", &restored)?;
    }

    Ok(())
}
