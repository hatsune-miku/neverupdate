use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ};
use winreg::RegKey;

use crate::error::Result;
use crate::guards::GuardPoint;
use crate::model::GuardPointStatus;
use crate::state::PersistedState;
use crate::ti_service::TiService;

const KEY_UX: &str = "SOFTWARE\\Microsoft\\WindowsUpdate\\UX\\Settings";
const KEY_POLICY: &str = "SOFTWARE\\Microsoft\\WindowsUpdate\\UpdatePolicy\\Settings";
const KEY_DEVICE: &str = "SOFTWARE\\Microsoft\\PolicyManager\\current\\device\\Update";
const KEY_WU: &str = "SOFTWARE\\Policies\\Microsoft\\Windows\\WindowsUpdate";
const KEY_WU_AU: &str = "SOFTWARE\\Policies\\Microsoft\\Windows\\WindowsUpdate\\AU";

pub struct FallbackGuard;

impl GuardPoint for FallbackGuard {
    fn id(&self) -> &'static str {
        "fallback_settings"
    }

    fn title(&self) -> &'static str {
        "兜底更新设置"
    }

    fn description(&self) -> &'static str {
        "长暂停更新、禁驱动/可选更新、抑制自动重启"
    }

    fn interception_behavior(&self) -> Option<&'static str> {
        Some("系统试图暗改更新设置")
    }

    fn check(&self, _state: &PersistedState) -> Result<GuardPointStatus> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

        let pause: String = hklm.open_subkey_with_flags(KEY_UX, KEY_READ).ok()
            .and_then(|k| k.get_value("PauseUpdatesExpiryTime").ok()).unwrap_or_default();
        let pq: u32 = hklm.open_subkey_with_flags(KEY_POLICY, KEY_READ).ok()
            .and_then(|k| k.get_value("PausedQualityStatus").ok()).unwrap_or(0);
        let pf: u32 = hklm.open_subkey_with_flags(KEY_POLICY, KEY_READ).ok()
            .and_then(|k| k.get_value("PausedFeatureStatus").ok()).unwrap_or(0);
        let drv: u32 = hklm.open_subkey_with_flags(KEY_DEVICE, KEY_READ).ok()
            .and_then(|k| k.get_value("ExcludeWUDriversInQualityUpdate").ok()).unwrap_or(0);
        let reboot: u32 = hklm.open_subkey_with_flags(KEY_WU_AU, KEY_READ).ok()
            .and_then(|k| k.get_value("NoAutoRebootWithLoggedOnUsers").ok()).unwrap_or(0);
        let opt: u32 = hklm.open_subkey_with_flags(KEY_WU, KEY_READ).ok()
            .and_then(|k| k.get_value("AllowOptionalContent").ok()).unwrap_or(0);

        let guarded = pause.starts_with("2051") && pq == 1 && pf == 1 && drv == 1 && reboot == 1 && opt == 0;
        let msg = format!(
            "PauseUntil={pause}, PausedQ={pq}, PausedF={pf}, Driver={drv}, Reboot={reboot}, Opt={opt}"
        );

        Ok(self.build_status(guarded, Some(msg)))
    }

    fn guard(&self, _state: &mut PersistedState, ti: &TiService) -> Result<GuardPointStatus> {
        let k_ux = ti.create_key(KEY_UX)?;
        let k_pol = ti.create_key(KEY_POLICY)?;
        let k_dev = ti.create_key(KEY_DEVICE)?;
        let k_wu = ti.create_key(KEY_WU)?;
        let k_au = ti.create_key(KEY_WU_AU)?;

        k_ux.set_value("PauseFeatureUpdatesStartTime", &"2025-01-01T00:00:00Z")?;
        k_ux.set_value("PauseFeatureUpdatesEndTime", &"2051-12-31T00:00:00Z")?;
        k_ux.set_value("PauseQualityUpdatesStartTime", &"2025-01-01T00:00:00Z")?;
        k_ux.set_value("PauseQualityUpdatesEndTime", &"2051-12-31T00:00:00Z")?;
        k_ux.set_value("PauseUpdatesStartTime", &"2025-01-01T00:00:00Z")?;
        k_ux.set_value("PauseUpdatesExpiryTime", &"2051-12-31T00:00:00Z")?;
        k_ux.set_value("FlightSettingsMaxPauseDays", &0x2727u32)?;

        k_pol.set_value("PausedFeatureStatus", &1u32)?;
        k_pol.set_value("PausedQualityStatus", &1u32)?;
        k_pol.set_value("PausedQualityDate", &"2025-01-01T00:00:00Z")?;
        k_pol.set_value("PausedFeatureDate", &"2025-01-01T00:00:00Z")?;

        k_dev.set_value("ExcludeWUDriversInQualityUpdate", &1u32)?;
        k_dev.set_value("DisableOSUpgrade", &1u32)?;

        k_wu.set_value("AllowOptionalContent", &0u32)?;

        k_au.set_value("NoAutoRebootWithLoggedOnUsers", &1u32)?;
        k_au.set_value("AlwaysAutoRebootAtScheduledTime", &0u32)?;

        ti.as_admin(|| self.check(_state))
    }

    fn release(&self, _state: &mut PersistedState, ti: &TiService) -> Result<GuardPointStatus> {
        if let Ok(k) = ti.open_key(KEY_WU_AU, true) {
            let _ = k.set_value("NoAutoRebootWithLoggedOnUsers", &0u32);
        }
        if let Ok(k) = ti.open_key(KEY_WU, true) {
            let _ = k.delete_value("AllowOptionalContent");
        }
        if let Ok(k) = ti.open_key(KEY_DEVICE, true) {
            let _ = k.set_value("ExcludeWUDriversInQualityUpdate", &0u32);
            let _ = k.set_value("DisableOSUpgrade", &0u32);
        }
        if let Ok(k) = ti.open_key(KEY_POLICY, true) {
            let _ = k.set_value("PausedFeatureStatus", &0u32);
            let _ = k.set_value("PausedQualityStatus", &0u32);
        }

        ti.as_admin(|| self.check(_state))
    }
}
