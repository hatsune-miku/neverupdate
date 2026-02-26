use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE};
use winreg::RegKey;

use crate::error::Result;
use crate::guards::status_fallback;
use crate::model::GuardPointStatus;

const KEY_UX_SETTINGS: &str = "SOFTWARE\\Microsoft\\WindowsUpdate\\UX\\Settings";
const KEY_UPDATE_POLICY_SETTINGS: &str =
    "SOFTWARE\\Microsoft\\WindowsUpdate\\UpdatePolicy\\Settings";
const KEY_POLICY_UPDATE_DEVICE: &str =
    "SOFTWARE\\Microsoft\\PolicyManager\\current\\device\\Update";
const KEY_POLICY_WU: &str = "SOFTWARE\\Policies\\Microsoft\\Windows\\WindowsUpdate";
const KEY_POLICY_WU_AU: &str = "SOFTWARE\\Policies\\Microsoft\\Windows\\WindowsUpdate\\AU";

pub fn check() -> Result<GuardPointStatus> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    let s_pause_end: String = hklm
        .open_subkey_with_flags(KEY_UX_SETTINGS, KEY_READ)
        .ok()
        .and_then(|key| key.get_value("PauseUpdatesExpiryTime").ok())
        .unwrap_or_default();

    let n_paused_quality: u32 = hklm
        .open_subkey_with_flags(KEY_UPDATE_POLICY_SETTINGS, KEY_READ)
        .ok()
        .and_then(|key| key.get_value("PausedQualityStatus").ok())
        .unwrap_or(0);

    let n_paused_feature: u32 = hklm
        .open_subkey_with_flags(KEY_UPDATE_POLICY_SETTINGS, KEY_READ)
        .ok()
        .and_then(|key| key.get_value("PausedFeatureStatus").ok())
        .unwrap_or(0);

    let n_driver_excluded: u32 = hklm
        .open_subkey_with_flags(KEY_POLICY_UPDATE_DEVICE, KEY_READ)
        .ok()
        .and_then(|key| key.get_value("ExcludeWUDriversInQualityUpdate").ok())
        .unwrap_or(0);

    let n_no_reboot: u32 = hklm
        .open_subkey_with_flags(KEY_POLICY_WU_AU, KEY_READ)
        .ok()
        .and_then(|key| key.get_value("NoAutoRebootWithLoggedOnUsers").ok())
        .unwrap_or(0);

    let n_optional_block: u32 = hklm
        .open_subkey_with_flags(KEY_POLICY_WU, KEY_READ)
        .ok()
        .and_then(|key| key.get_value("AllowOptionalContent").ok())
        .unwrap_or(0);

    let guarded = s_pause_end.starts_with("2051")
        && n_paused_quality == 1
        && n_paused_feature == 1
        && n_driver_excluded == 1
        && n_no_reboot == 1
        && n_optional_block == 0;

    let message = Some(format!(
    "PauseUntil={s_pause_end}, PausedQuality={n_paused_quality}, PausedFeature={n_paused_feature}, ExcludeDrivers={n_driver_excluded}, NoAutoReboot={n_no_reboot}, OptionalContent={n_optional_block}"
  ));

    Ok(status_fallback(guarded, message))
}

pub fn guard() -> Result<GuardPointStatus> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let (key_ux, _) = hklm.create_subkey(KEY_UX_SETTINGS)?;
    let (key_policy, _) = hklm.create_subkey(KEY_UPDATE_POLICY_SETTINGS)?;
    let (key_device, _) = hklm.create_subkey(KEY_POLICY_UPDATE_DEVICE)?;
    let (key_wu, _) = hklm.create_subkey(KEY_POLICY_WU)?;
    let (key_wu_au, _) = hklm.create_subkey(KEY_POLICY_WU_AU)?;

    key_ux.set_value("PauseFeatureUpdatesStartTime", &"2025-01-01T00:00:00Z")?;
    key_ux.set_value("PauseFeatureUpdatesEndTime", &"2051-12-31T00:00:00Z")?;
    key_ux.set_value("PauseQualityUpdatesStartTime", &"2025-01-01T00:00:00Z")?;
    key_ux.set_value("PauseQualityUpdatesEndTime", &"2051-12-31T00:00:00Z")?;
    key_ux.set_value("PauseUpdatesStartTime", &"2025-01-01T00:00:00Z")?;
    key_ux.set_value("PauseUpdatesExpiryTime", &"2051-12-31T00:00:00Z")?;
    key_ux.set_value("FlightSettingsMaxPauseDays", &0x2727u32)?;

    key_policy.set_value("PausedFeatureStatus", &1u32)?;
    key_policy.set_value("PausedQualityStatus", &1u32)?;
    key_policy.set_value("PausedQualityDate", &"2025-01-01T00:00:00Z")?;
    key_policy.set_value("PausedFeatureDate", &"2025-01-01T00:00:00Z")?;

    key_device.set_value("ExcludeWUDriversInQualityUpdate", &1u32)?;
    key_device.set_value("DisableOSUpgrade", &1u32)?;

    key_wu.set_value("AllowOptionalContent", &0u32)?;

    key_wu_au.set_value("NoAutoRebootWithLoggedOnUsers", &1u32)?;
    key_wu_au.set_value("AlwaysAutoRebootAtScheduledTime", &0u32)?;

    check()
}

pub fn release() -> Result<GuardPointStatus> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    if let Ok(key_wu_au) = hklm.open_subkey_with_flags(KEY_POLICY_WU_AU, KEY_READ | KEY_WRITE) {
        let _ = key_wu_au.set_value("NoAutoRebootWithLoggedOnUsers", &0u32);
    }

    if let Ok(key_wu) = hklm.open_subkey_with_flags(KEY_POLICY_WU, KEY_READ | KEY_WRITE) {
        let _ = key_wu.delete_value("AllowOptionalContent");
    }

    if let Ok(key_device) =
        hklm.open_subkey_with_flags(KEY_POLICY_UPDATE_DEVICE, KEY_READ | KEY_WRITE)
    {
        let _ = key_device.set_value("ExcludeWUDriversInQualityUpdate", &0u32);
        let _ = key_device.set_value("DisableOSUpgrade", &0u32);
    }

    if let Ok(key_policy) =
        hklm.open_subkey_with_flags(KEY_UPDATE_POLICY_SETTINGS, KEY_READ | KEY_WRITE)
    {
        let _ = key_policy.set_value("PausedFeatureStatus", &0u32);
        let _ = key_policy.set_value("PausedQualityStatus", &0u32);
    }

    check()
}
