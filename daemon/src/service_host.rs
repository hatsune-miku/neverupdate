//! Windows SCM: `StartServiceCtrlDispatcher` + `SetServiceStatus(RUNNING)` fixes error 1053.

use std::mem::zeroed;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::thread;

use nu_core::daemon_service_name;
use windows_sys::Win32::Foundation::{GetLastError, ERROR_FAILED_SERVICE_CONTROLLER_CONNECT};
use windows_sys::Win32::System::Services::{
    RegisterServiceCtrlHandlerW, SetServiceStatus, StartServiceCtrlDispatcherW, SERVICE_ACCEPT_STOP,
    SERVICE_CONTROL_STOP, SERVICE_RUNNING, SERVICE_START_PENDING, SERVICE_STATUS,
    SERVICE_STATUS_HANDLE, SERVICE_STOP_PENDING, SERVICE_STOPPED, SERVICE_TABLE_ENTRYW,
    SERVICE_WIN32_OWN_PROCESS,
};

static STOP_REQUESTED: AtomicBool = AtomicBool::new(false);
/// Stored as `usize` because raw service handles are not `Send`/`Sync` for static `Mutex`.
static STATUS_HANDLE: Mutex<Option<usize>> = Mutex::new(None);
static NAME_WIDE: OnceLock<Vec<u16>> = OnceLock::new();

fn status_handle_get() -> Option<SERVICE_STATUS_HANDLE> {
    STATUS_HANDLE
        .lock()
        .ok()
        .and_then(|g| (*g).map(|bits| bits as SERVICE_STATUS_HANDLE))
}

fn service_name_wide_ptr() -> *mut u16 {
    let v = NAME_WIDE.get_or_init(|| {
        daemon_service_name()
            .encode_utf16()
            .chain(Some(0))
            .collect()
    });
    v.as_ptr() as *mut u16
}

pub enum DispatcherOutcome {
    Console,
    ServiceStopped,
}

unsafe extern "system" fn control_handler(ctrl: u32) {
    if ctrl != SERVICE_CONTROL_STOP {
        return;
    }
    STOP_REQUESTED.store(true, Ordering::SeqCst);
    if let Some(h) = status_handle_get() {
        let mut st: SERVICE_STATUS = zeroed();
        st.dwServiceType = SERVICE_WIN32_OWN_PROCESS;
        st.dwCurrentState = SERVICE_STOP_PENDING;
        st.dwControlsAccepted = 0;
        unsafe { SetServiceStatus(h, &st) };
    }
}

unsafe extern "system" fn service_main(_argc: u32, _argv: *mut *mut u16) {
    let h = RegisterServiceCtrlHandlerW(service_name_wide_ptr(), Some(control_handler));
    if h.is_null() {
        return;
    }
    if let Ok(mut g) = STATUS_HANDLE.lock() {
        *g = Some(h as usize);
    }

    let mut st: SERVICE_STATUS = zeroed();
    st.dwServiceType = SERVICE_WIN32_OWN_PROCESS;
    st.dwCurrentState = SERVICE_START_PENDING;
    st.dwControlsAccepted = 0;
    SetServiceStatus(h, &st);

    let worker = thread::spawn(|| {
        if let Err(e) = crate::run_daemon_work() {
            eprintln!("{e}");
        }
    });

    st.dwCurrentState = SERVICE_RUNNING;
    st.dwControlsAccepted = SERVICE_ACCEPT_STOP;
    SetServiceStatus(h, &st);

    let _ = worker.join();

    st.dwCurrentState = SERVICE_STOPPED;
    st.dwControlsAccepted = 0;
    SetServiceStatus(h, &st);
}

pub fn enter_dispatcher() -> DispatcherOutcome {
    let table = [
        SERVICE_TABLE_ENTRYW {
            lpServiceName: service_name_wide_ptr(),
            lpServiceProc: Some(service_main),
        },
        SERVICE_TABLE_ENTRYW {
            lpServiceName: null_mut(),
            lpServiceProc: None,
        },
    ];

    unsafe {
        if StartServiceCtrlDispatcherW(table.as_ptr()) == 0 {
            let e = GetLastError();
            if e == ERROR_FAILED_SERVICE_CONTROLLER_CONNECT {
                return DispatcherOutcome::Console;
            }
            eprintln!("StartServiceCtrlDispatcherW failed (error {e})");
            std::process::exit(1);
        }
    }
    DispatcherOutcome::ServiceStopped
}

pub fn stop_requested() -> bool {
    STOP_REQUESTED.load(Ordering::SeqCst)
}
