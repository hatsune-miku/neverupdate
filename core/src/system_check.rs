use std::ptr;

use is_elevated::is_elevated;
use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE};
use windows_sys::Win32::System::Threading::CreateMutexW;
use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ, KEY_SET_VALUE};
use winreg::RegKey;

use crate::error::{NuError, Result};
use crate::model::{PreflightCheck, PreflightReport};

pub struct GlobalInstanceGuard {
    h_mutex: HANDLE,
}

impl Drop for GlobalInstanceGuard {
    fn drop(&mut self) {
        if !self.h_mutex.is_null() {
            unsafe {
                let _ = CloseHandle(self.h_mutex);
            }
        }
    }
}

pub fn acquire_global_instance(name: &str) -> Result<GlobalInstanceGuard> {
    let mut wide: Vec<u16> = name.encode_utf16().collect();
    wide.push(0);

    let h_mutex = unsafe { CreateMutexW(ptr::null(), 0, wide.as_ptr()) };
    if h_mutex.is_null() {
        return Err(NuError::InvalidOperation(
            "failed to create instance mutex".to_string(),
        ));
    }

    let n_last_error = unsafe { GetLastError() };
    if n_last_error == ERROR_ALREADY_EXISTS {
        unsafe {
            let _ = CloseHandle(h_mutex);
        }
        return Err(NuError::InvalidOperation(
            "another instance is already running".to_string(),
        ));
    }

    Ok(GlobalInstanceGuard { h_mutex })
}

fn read_windows_identity() -> Result<(String, String, u32)> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key =
        hklm.open_subkey_with_flags("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion", KEY_READ)?;

    let s_product_name: String = key.get_value("ProductName").unwrap_or_default();
    let s_installation_type: String = key.get_value("InstallationType").unwrap_or_default();

    let s_build_number: String = key
        .get_value("CurrentBuildNumber")
        .unwrap_or_else(|_| String::from("0"));
    let n_build = s_build_number.parse::<u32>().unwrap_or(0);

    Ok((s_product_name, s_installation_type, n_build))
}

pub fn has_admin_privilege() -> bool {
    is_elevated()
}

pub fn verify_admin_writable() -> bool {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm.open_subkey_with_flags("SOFTWARE", KEY_SET_VALUE);

    match key {
        Ok(value) => {
            let write_result = value.set_value("NeverUpdatePermissionProbe", &"1");
            let _ = value.delete_value("NeverUpdatePermissionProbe");
            write_result.is_ok()
        }
        Err(_) => false,
    }
}

pub fn run_preflight_checks() -> PreflightReport {
    let mut checks = Vec::new();

    let (s_product_name, s_installation_type, n_build) =
        read_windows_identity().unwrap_or((String::new(), String::new(), 0));
    let b_windows11 = s_product_name.contains("Windows 11") || n_build >= 22000;
    checks.push(PreflightCheck {
        id: String::from("windows11"),
        title: String::from("系统必须是 Windows 11"),
        passed: b_windows11,
        detail: format!("ProductName={s_product_name}, Build={n_build}"),
    });

    let b_not_server = !s_installation_type.to_ascii_lowercase().contains("server");
    checks.push(PreflightCheck {
        id: String::from("not_server"),
        title: String::from("系统不能是 Windows Server"),
        passed: b_not_server,
        detail: format!("InstallationType={s_installation_type}"),
    });

    let b_not_ltsc = !s_product_name.to_ascii_lowercase().contains("ltsc");
    checks.push(PreflightCheck {
        id: String::from("not_ltsc"),
        title: String::from("系统不能是 LTSC"),
        passed: b_not_ltsc,
        detail: format!("ProductName={s_product_name}"),
    });

    let b_elevated = has_admin_privilege();
    checks.push(PreflightCheck {
        id: String::from("admin"),
        title: String::from("需要管理员权限"),
        passed: b_elevated,
        detail: if b_elevated {
            String::from("is_elevated=true")
        } else {
            String::from("is_elevated=false")
        },
    });

    let b_elevated_verified = verify_admin_writable();
    checks.push(PreflightCheck {
        id: String::from("admin_verified"),
        title: String::from("管理员权限可实际写入系统设置"),
        passed: b_elevated_verified,
        detail: if b_elevated_verified {
            String::from("registry write test passed")
        } else {
            String::from("registry write test failed")
        },
    });

    let b_single_instance = acquire_global_instance("Global\\NeverUpdatePreflightProbe").is_ok();
    checks.push(PreflightCheck {
        id: String::from("single_instance_probe"),
        title: String::from("没有重复运行"),
        passed: b_single_instance,
        detail: if b_single_instance {
            String::from("mutex available")
        } else {
            String::from("mutex unavailable")
        },
    });

    let passed = checks.iter().all(|item| item.passed);

    PreflightReport { passed, checks }
}
