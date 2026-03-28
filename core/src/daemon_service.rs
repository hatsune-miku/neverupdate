use std::mem::zeroed;
use std::ptr::null;

use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::System::Services::{
    ChangeServiceConfig2W, CloseServiceHandle, ControlService, CreateServiceW, DeleteService,
    OpenSCManagerW, OpenServiceW, QueryServiceStatus, StartServiceW, SC_HANDLE,
    SC_MANAGER_CONNECT, SC_MANAGER_CREATE_SERVICE, SERVICE_ALL_ACCESS, SERVICE_AUTO_START,
    SERVICE_CONFIG_DESCRIPTION, SERVICE_CONTROL_STOP, SERVICE_DESCRIPTIONW,
    SERVICE_ERROR_NORMAL, SERVICE_QUERY_STATUS, SERVICE_RUNNING, SERVICE_START, SERVICE_STATUS,
    SERVICE_STOP, SERVICE_WIN32_OWN_PROCESS,
};

use crate::error::{NuError, Result};

const DAEMON_SERVICE_NAME: &str = "NeverUpdateDaemon";
const DISPLAY_NAME: &str = "NeverUpdate Daemon";
const DESCRIPTION: &str = "NeverUpdate privileged maintenance daemon";

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

fn normalize_path(p: &str) -> String {
    p.strip_prefix(r"\\?\").unwrap_or(p).to_string()
}

struct ScHandle(SC_HANDLE);
impl Drop for ScHandle {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { CloseServiceHandle(self.0); }
        }
    }
}

fn open_scm(access: u32) -> Result<ScHandle> {
    let h = unsafe { OpenSCManagerW(null(), null(), access) };
    if h.is_null() {
        return Err(NuError::InvalidOperation("cannot open SCManager".into()));
    }
    Ok(ScHandle(h))
}

fn open_svc(scm: &ScHandle, access: u32) -> Option<ScHandle> {
    let h = unsafe { OpenServiceW(scm.0, wide(DAEMON_SERVICE_NAME).as_ptr(), access) };
    if h.is_null() { None } else { Some(ScHandle(h)) }
}

pub fn daemon_service_name() -> &'static str {
    DAEMON_SERVICE_NAME
}

pub fn daemon_service_exists() -> bool {
    let Ok(scm) = open_scm(SC_MANAGER_CONNECT) else { return false };
    open_svc(&scm, SERVICE_QUERY_STATUS).is_some()
}

pub fn daemon_service_running() -> bool {
    let Ok(scm) = open_scm(SC_MANAGER_CONNECT) else { return false };
    let Some(svc) = open_svc(&scm, SERVICE_QUERY_STATUS) else { return false };
    let mut status: SERVICE_STATUS = unsafe { zeroed() };
    if unsafe { QueryServiceStatus(svc.0, &mut status) } == 0 {
        return false;
    }
    status.dwCurrentState == SERVICE_RUNNING
}

pub fn register_daemon_service(exe_path: &str) -> Result<()> {
    let exe = normalize_path(exe_path);
    let bin_path = format!("\"{exe}\" run");

    let scm = open_scm(SC_MANAGER_CREATE_SERVICE)?;
    let h = unsafe {
        CreateServiceW(
            scm.0,
            wide(DAEMON_SERVICE_NAME).as_ptr(),
            wide(DISPLAY_NAME).as_ptr(),
            SERVICE_ALL_ACCESS,
            SERVICE_WIN32_OWN_PROCESS,
            SERVICE_AUTO_START,
            SERVICE_ERROR_NORMAL,
            wide(&bin_path).as_ptr(),
            null(),
            null::<u32>() as *mut u32,
            null(),
            null(),
            null(),
        )
    };
    if h.is_null() {
        let code = unsafe { GetLastError() };
        return Err(NuError::InvalidOperation(format!(
            "CreateServiceW failed (error {code})"
        )));
    }
    let svc = ScHandle(h);

    let mut desc_text = wide(DESCRIPTION);
    let mut desc = SERVICE_DESCRIPTIONW {
        lpDescription: desc_text.as_mut_ptr(),
    };
    unsafe {
        ChangeServiceConfig2W(
            svc.0,
            SERVICE_CONFIG_DESCRIPTION,
            &mut desc as *mut _ as *const _,
        );
    }
    Ok(())
}

pub fn unregister_daemon_service() -> Result<()> {
    let _ = stop_daemon_service();
    let scm = open_scm(SC_MANAGER_CONNECT)?;
    let svc = open_svc(&scm, SERVICE_ALL_ACCESS)
        .ok_or_else(|| NuError::InvalidOperation("service not found".into()))?;
    if unsafe { DeleteService(svc.0) } == 0 {
        let code = unsafe { GetLastError() };
        return Err(NuError::InvalidOperation(format!(
            "DeleteService failed (error {code})"
        )));
    }
    Ok(())
}

pub fn reregister_daemon_service(exe_path: &str) -> Result<()> {
    if daemon_service_exists() {
        let _ = unregister_daemon_service();
    }
    register_daemon_service(exe_path)
}

pub fn start_daemon_service() -> Result<()> {
    if !daemon_service_exists() {
        return Err(NuError::InvalidOperation("service not installed yet".into()));
    }
    let scm = open_scm(SC_MANAGER_CONNECT)?;
    let svc = open_svc(&scm, SERVICE_START)
        .ok_or_else(|| NuError::InvalidOperation("cannot open service".into()))?;
    if unsafe { StartServiceW(svc.0, 0, null()) } == 0 {
        let code = unsafe { GetLastError() };
        return Err(NuError::InvalidOperation(format!(
            "StartServiceW failed (error {code}; 1053=service did not respond in time — check Event Viewer / daemon preflight)"
        )));
    }
    Ok(())
}

pub fn stop_daemon_service() -> Result<()> {
    let scm = open_scm(SC_MANAGER_CONNECT)?;
    let svc = open_svc(&scm, SERVICE_STOP)
        .ok_or_else(|| NuError::InvalidOperation("service not found".into()))?;
    let mut status: SERVICE_STATUS = unsafe { zeroed() };
    if unsafe { ControlService(svc.0, SERVICE_CONTROL_STOP, &mut status) } == 0 {
        let code = unsafe { GetLastError() };
        return Err(NuError::InvalidOperation(format!(
            "ControlService STOP failed (error {code})"
        )));
    }
    Ok(())
}
