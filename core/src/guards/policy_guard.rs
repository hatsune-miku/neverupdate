use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE};
use winreg::RegKey;

use crate::error::Result;
use crate::guards::status_policy;
use crate::model::GuardPointStatus;

const KEY_WU: &str = "SOFTWARE\\Policies\\Microsoft\\Windows\\WindowsUpdate";
const KEY_WU_AU: &str = "SOFTWARE\\Policies\\Microsoft\\Windows\\WindowsUpdate\\AU";
const KEY_POLICY_UPDATE_DEVICE: &str =
    "SOFTWARE\\Microsoft\\PolicyManager\\current\\device\\Update";

pub fn check() -> Result<GuardPointStatus> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    let n_no_auto_update: u32 = hklm
        .open_subkey_with_flags(KEY_WU_AU, KEY_READ)
        .ok()
        .and_then(|key| key.get_value("NoAutoUpdate").ok())
        .unwrap_or(0);

    let n_disable_update_access: u32 = hklm
        .open_subkey_with_flags(KEY_WU, KEY_READ)
        .ok()
        .and_then(|key| key.get_value("DisableWindowsUpdateAccess").ok())
        .unwrap_or(0);

    let n_set_disable_ux: u32 = hklm
        .open_subkey_with_flags(KEY_POLICY_UPDATE_DEVICE, KEY_READ)
        .ok()
        .and_then(|key| key.get_value("SetDisableUXWUAccess").ok())
        .unwrap_or(0);

    let guarded = n_no_auto_update == 1 && n_disable_update_access == 1 && n_set_disable_ux == 1;
    let message = Some(format!(
    "NoAutoUpdate={n_no_auto_update}, DisableWindowsUpdateAccess={n_disable_update_access}, SetDisableUXWUAccess={n_set_disable_ux}; GPO paths: 计算机配置/管理模板/Windows 组件/Windows 更新/管理最终用户体验/配置自动更新 + 计算机配置/管理模板/Windows 组件/Windows 更新/删除使用所有 Windows 更新功能的访问权限"
  ));

    Ok(status_policy(guarded, message))
}

pub fn guard() -> Result<GuardPointStatus> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let (key_wu, _) = hklm.create_subkey(KEY_WU)?;
    let (key_au, _) = hklm.create_subkey(KEY_WU_AU)?;
    let (key_device, _) = hklm.create_subkey(KEY_POLICY_UPDATE_DEVICE)?;

    key_au.set_value("NoAutoUpdate", &1u32)?;
    key_au.set_value("AUOptions", &1u32)?;
    key_wu.set_value("DisableWindowsUpdateAccess", &1u32)?;
    key_device.set_value("SetDisableUXWUAccess", &1u32)?;

    check()
}

pub fn release() -> Result<GuardPointStatus> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    if let Ok(key_wu) = hklm.open_subkey_with_flags(KEY_WU, KEY_READ | KEY_WRITE) {
        let _ = key_wu.set_value("DisableWindowsUpdateAccess", &0u32);
    }

    if let Ok(key_au) = hklm.open_subkey_with_flags(KEY_WU_AU, KEY_READ | KEY_WRITE) {
        let _ = key_au.set_value("NoAutoUpdate", &0u32);
        let _ = key_au.set_value("AUOptions", &3u32);
    }

    if let Ok(key_device) =
        hklm.open_subkey_with_flags(KEY_POLICY_UPDATE_DEVICE, KEY_READ | KEY_WRITE)
    {
        let _ = key_device.set_value("SetDisableUXWUAccess", &0u32);
    }

    check()
}
