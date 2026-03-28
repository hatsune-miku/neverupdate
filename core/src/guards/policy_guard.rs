use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ};
use winreg::RegKey;

use crate::error::Result;
use crate::guards::GuardPoint;
use crate::model::GuardPointStatus;
use crate::state::PersistedState;
use crate::ti_service::TiService;

const KEY_WU: &str = "SOFTWARE\\Policies\\Microsoft\\Windows\\WindowsUpdate";
const KEY_WU_AU: &str = "SOFTWARE\\Policies\\Microsoft\\Windows\\WindowsUpdate\\AU";
const KEY_POLICY_UPDATE_DEVICE: &str =
    "SOFTWARE\\Microsoft\\PolicyManager\\current\\device\\Update";

pub struct PolicyGuard;

impl GuardPoint for PolicyGuard {
    fn id(&self) -> &'static str {
        "group_policy"
    }

    fn title(&self) -> &'static str {
        "组策略"
    }

    fn description(&self) -> &'static str {
        "配置自动更新=禁用，删除访问更新功能=启用"
    }

    fn interception_behavior(&self) -> Option<&'static str> {
        Some("系统试图暗改组策略以恢复更新")
    }

    fn check(&self, _state: &PersistedState) -> Result<GuardPointStatus> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

        let v1: u32 = hklm.open_subkey_with_flags(KEY_WU_AU, KEY_READ).ok()
            .and_then(|k| k.get_value("NoAutoUpdate").ok()).unwrap_or(0);
        let v2: u32 = hklm.open_subkey_with_flags(KEY_WU, KEY_READ).ok()
            .and_then(|k| k.get_value("DisableWindowsUpdateAccess").ok()).unwrap_or(0);
        let v3: u32 = hklm.open_subkey_with_flags(KEY_POLICY_UPDATE_DEVICE, KEY_READ).ok()
            .and_then(|k| k.get_value("SetDisableUXWUAccess").ok()).unwrap_or(0);

        let guarded = v1 == 1 && v2 == 1 && v3 == 1;
        let msg = format!(
            "NoAutoUpdate={v1}, DisableWindowsUpdateAccess={v2}, SetDisableUXWUAccess={v3}"
        );

        Ok(self.build_status(guarded, Some(msg)))
    }

    fn guard(&self, _state: &mut PersistedState, ti: &TiService) -> Result<GuardPointStatus> {
        let k_wu = ti.create_key(KEY_WU)?;
        let k_au = ti.create_key(KEY_WU_AU)?;
        let k_dev = ti.create_key(KEY_POLICY_UPDATE_DEVICE)?;

        k_au.set_value("NoAutoUpdate", &1u32)?;
        k_au.set_value("AUOptions", &1u32)?;
        k_wu.set_value("DisableWindowsUpdateAccess", &1u32)?;
        k_dev.set_value("SetDisableUXWUAccess", &1u32)?;

        ti.as_admin(|| self.check(_state))
    }

    fn release(&self, _state: &mut PersistedState, ti: &TiService) -> Result<GuardPointStatus> {
        if let Ok(k) = ti.open_key(KEY_WU, true) {
            let _ = k.set_value("DisableWindowsUpdateAccess", &0u32);
        }
        if let Ok(k) = ti.open_key(KEY_WU_AU, true) {
            let _ = k.set_value("NoAutoUpdate", &0u32);
            let _ = k.set_value("AUOptions", &3u32);
        }
        if let Ok(k) = ti.open_key(KEY_POLICY_UPDATE_DEVICE, true) {
            let _ = k.set_value("SetDisableUXWUAccess", &0u32);
        }

        ti.as_admin(|| self.check(_state))
    }
}
