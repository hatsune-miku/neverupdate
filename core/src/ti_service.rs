use std::mem::{size_of, zeroed};
use std::path::Path;
use std::ptr::{null, null_mut};

use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE};
use winreg::RegKey;

use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Security::{
    AdjustTokenPrivileges, DuplicateTokenEx, ImpersonateLoggedOnUser, LookupPrivilegeValueW,
    RevertToSelf, SecurityImpersonation, TokenImpersonation, SE_PRIVILEGE_ENABLED,
    TOKEN_ADJUST_PRIVILEGES, TOKEN_DUPLICATE, TOKEN_IMPERSONATE, TOKEN_PRIVILEGES, TOKEN_QUERY,
};
use windows_sys::Win32::System::Services::{
    CloseServiceHandle, OpenSCManagerW, OpenServiceW, QueryServiceStatusEx, StartServiceW,
    SC_HANDLE, SC_MANAGER_CONNECT, SC_STATUS_PROCESS_INFO, SERVICE_QUERY_STATUS, SERVICE_RUNNING,
    SERVICE_START, SERVICE_STATUS_PROCESS,
};
use windows_sys::Win32::System::Threading::{
    GetCurrentProcess, OpenProcess, OpenProcessToken, PROCESS_QUERY_INFORMATION,
};

use crate::error::{NuError, Result};

#[link(name = "kernel32")]
extern "system" {
    fn CreateNamedPipeW(
        lpName: *const u16,
        dwOpenMode: u32,
        dwPipeMode: u32,
        nMaxInstances: u32,
        nOutBufferSize: u32,
        nInBufferSize: u32,
        nDefaultTimeOut: u32,
        lpSecurityAttributes: *const u8,
    ) -> HANDLE;
    fn ConnectNamedPipe(hNamedPipe: HANDLE, lpOverlapped: *mut u8) -> i32;
    fn DisconnectNamedPipe(hNamedPipe: HANDLE) -> i32;
    fn ReadFile(
        hFile: HANDLE,
        lpBuffer: *mut u8,
        nNumberOfBytesToRead: u32,
        lpNumberOfBytesRead: *mut u32,
        lpOverlapped: *mut u8,
    ) -> i32;
}

#[link(name = "advapi32")]
extern "system" {
    fn ImpersonateNamedPipeClient(hNamedPipe: HANDLE) -> i32;
}

const MAXIMUM_ALLOWED: u32 = 0x0200_0000;
const PIPE_ACCESS_DUPLEX: u32 = 0x0000_0003;
const PIPE_NAME: &str = "\\\\.\\pipe\\NuTiSystemHelper";
const TASK_NAME: &str = "NuTiHelper";

const ALLOWED_REG_PREFIXES: &[&str] = &[
    "SYSTEM\\CurrentControlSet\\Services\\WaaSMedicSvc",
    "SYSTEM\\CurrentControlSet\\Services\\UsoSvc",
    "SYSTEM\\CurrentControlSet\\Services\\uhssvc",
    "SYSTEM\\CurrentControlSet\\Services\\wuauserv",
    "SOFTWARE\\Policies\\Microsoft\\Windows\\WindowsUpdate",
    "SOFTWARE\\Microsoft\\WindowsUpdate",
    "SOFTWARE\\Microsoft\\PolicyManager\\current\\device\\Update",
];

// ── public API ──────────────────────────────────────────────────────

pub struct TiService {
    _token: TiToken,
}

impl TiService {
    pub fn acquire() -> Result<Self> {
        Ok(Self {
            _token: acquire_token()?,
        })
    }

    pub fn probe() -> Result<()> {
        let _token = acquire_token()?;
        Ok(())
    }

    /// Temporarily revert to admin context for child-process spawning
    /// or registry reads that don't need TI privileges.
    pub fn as_admin<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        unsafe { RevertToSelf(); }
        let r = f();
        unsafe { ImpersonateLoggedOnUser(self._token.h); }
        r
    }

    // ── registry (HKLM only, allowlist-gated) ──

    pub fn open_key(&self, subkey: &str, writable: bool) -> Result<RegKey> {
        validate_key(subkey)?;
        let flags = if writable {
            KEY_READ | KEY_WRITE
        } else {
            KEY_READ
        };
        Ok(RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey_with_flags(subkey, flags)?)
    }

    pub fn create_key(&self, subkey: &str) -> Result<RegKey> {
        validate_key(subkey)?;
        let (key, _) = RegKey::predef(HKEY_LOCAL_MACHINE).create_subkey(subkey)?;
        Ok(key)
    }

    // ── file I/O (allowlist-gated) ──

    pub fn read_file(&self, path: &Path) -> Result<Vec<u8>> {
        validate_file(path)?;
        Ok(std::fs::read(path)?)
    }

    pub fn write_file(&self, path: &Path, content: &[u8]) -> Result<()> {
        validate_file(path)?;
        Ok(std::fs::write(path, content)?)
    }

    pub fn read_file_string(&self, path: &Path) -> Result<String> {
        validate_file(path)?;
        Ok(std::fs::read_to_string(path)?)
    }

    pub fn write_file_string(&self, path: &Path, content: &str) -> Result<()> {
        validate_file(path)?;
        Ok(std::fs::write(path, content)?)
    }

    pub fn remove_file(&self, path: &Path) -> Result<()> {
        validate_file(path)?;
        Ok(std::fs::remove_file(path)?)
    }

    pub fn remove_dir_all(&self, path: &Path) -> Result<()> {
        validate_file(path)?;
        Ok(std::fs::remove_dir_all(path)?)
    }
}

// ── validation ──────────────────────────────────────────────────────

fn validate_key(subkey: &str) -> Result<()> {
    let n = subkey.replace('/', "\\");
    if ALLOWED_REG_PREFIXES.iter().any(|p| n.starts_with(p)) {
        return Ok(());
    }
    Err(NuError::InvalidOperation(format!(
        "registry path blocked by allowlist: {subkey}"
    )))
}

fn validate_file(path: &Path) -> Result<()> {
    let s = path.to_string_lossy().to_lowercase();
    let ok = s.contains("\\system32\\drivers\\etc\\hosts")
        || s.contains("\\system32\\tasks\\microsoft\\windows\\updateorchestrator")
        || s.contains("\\system32\\tasks\\microsoft\\windows\\waasmedic")
        || s.contains("\\softwaredistribution");
    if ok {
        return Ok(());
    }
    Err(NuError::InvalidOperation(format!(
        "file path blocked by allowlist: {}",
        path.display()
    )))
}

// ── token acquisition: Admin → SYSTEM → TI ─────────────────────────
// Admin cannot directly open TI's process token because the token
// object DACL only grants TOKEN_QUERY to Administrators, and
// SeDebugPrivilege only bypasses process/thread object checks.
//
// Two-step approach:
//   1. Impersonate SYSTEM via named-pipe + scheduled-task trick
//   2. As SYSTEM, open TI process token normally (SYSTEM has full access)

struct TiToken {
    h: HANDLE,
}

impl Drop for TiToken {
    fn drop(&mut self) {
        unsafe {
            RevertToSelf();
            if !self.h.is_null() {
                CloseHandle(self.h);
            }
        }
    }
}

fn acquire_token() -> Result<TiToken> {
    enable_debug_privilege();

    // Phase 1: become SYSTEM via named pipe
    impersonate_system()?;

    // Phase 2: as SYSTEM, grab the TI token
    let pid = match start_and_get_ti_pid() {
        Ok(p) => p,
        Err(e) => {
            unsafe { RevertToSelf(); }
            return Err(e);
        }
    };

    let ti_token = match unsafe { open_and_dup_ti_token(pid) } {
        Ok(t) => t,
        Err(e) => {
            unsafe { RevertToSelf(); }
            return Err(e);
        }
    };

    // Phase 3: drop SYSTEM, switch to TI
    unsafe {
        RevertToSelf();
        if ImpersonateLoggedOnUser(ti_token) == 0 {
            CloseHandle(ti_token);
            return Err(NuError::InvalidOperation(
                "TI impersonation failed".into(),
            ));
        }
    }

    Ok(TiToken { h: ti_token })
}

// ── Phase 1: SYSTEM impersonation via named pipe ────────────────────

fn impersonate_system() -> Result<()> {
    unsafe {
        let pipe_name = wide(PIPE_NAME);
        let h_pipe = CreateNamedPipeW(
            pipe_name.as_ptr(),
            PIPE_ACCESS_DUPLEX | 0x00080000, // FILE_FLAG_FIRST_PIPE_INSTANCE
            0,                                // byte-type, byte-read, blocking
            1,
            256,
            256,
            10000,
            null(),
        );

        if h_pipe == INVALID_HANDLE_VALUE {
            return Err(NuError::InvalidOperation(
                "cannot create named pipe for SYSTEM impersonation".into(),
            ));
        }

        let task_cmd = format!("cmd /c echo 1 > {PIPE_NAME}");
        let create = crate::command::run_command(
            "schtasks",
            &[
                "/Create", "/TN", TASK_NAME, "/TR", &task_cmd,
                "/SC", "ONCE", "/ST", "00:00", "/RU", "SYSTEM", "/F",
            ],
        );
        if let Err(e) = create {
            CloseHandle(h_pipe);
            return Err(NuError::InvalidOperation(format!(
                "cannot create SYSTEM helper task: {e}"
            )));
        }

        let run = crate::command::run_command("schtasks", &["/Run", "/TN", TASK_NAME]);
        if let Err(e) = run {
            cleanup_task();
            CloseHandle(h_pipe);
            return Err(NuError::InvalidOperation(format!(
                "cannot start SYSTEM helper task: {e}"
            )));
        }

        // Block until the SYSTEM task connects (timeout is the pipe's default 10 s)
        ConnectNamedPipe(h_pipe, null_mut());

        // Read at least 1 byte — required before ImpersonateNamedPipeClient
        let mut buf = [0u8; 16];
        let mut n: u32 = 0;
        ReadFile(h_pipe, buf.as_mut_ptr(), 16, &mut n, null_mut());

        let ok = ImpersonateNamedPipeClient(h_pipe);

        DisconnectNamedPipe(h_pipe);
        CloseHandle(h_pipe);
        cleanup_task();

        if ok == 0 {
            return Err(NuError::InvalidOperation(
                "SYSTEM pipe impersonation failed".into(),
            ));
        }
        Ok(())
    }
}

fn cleanup_task() {
    let _ = crate::command::run_command("schtasks", &["/Delete", "/TN", TASK_NAME, "/F"]);
}

// ── Phase 2: open TI token (requires SYSTEM context) ───────────────

unsafe fn open_and_dup_ti_token(pid: u32) -> Result<HANDLE> {
    let h_proc = OpenProcess(PROCESS_QUERY_INFORMATION, 0, pid);
    if h_proc.is_null() {
        return Err(NuError::InvalidOperation(
            "cannot open TI process (as SYSTEM)".into(),
        ));
    }

    let mut h_tok: HANDLE = null_mut();
    if OpenProcessToken(h_proc, TOKEN_DUPLICATE | TOKEN_IMPERSONATE, &mut h_tok) == 0 {
        CloseHandle(h_proc);
        return Err(NuError::InvalidOperation(
            "cannot open TI token (as SYSTEM)".into(),
        ));
    }

    let mut h_dup: HANDLE = null_mut();
    let ok = DuplicateTokenEx(
        h_tok,
        MAXIMUM_ALLOWED,
        null(),
        SecurityImpersonation,
        TokenImpersonation,
        &mut h_dup,
    );
    CloseHandle(h_tok);
    CloseHandle(h_proc);

    if ok == 0 {
        return Err(NuError::InvalidOperation(
            "cannot duplicate TI token".into(),
        ));
    }
    Ok(h_dup)
}

// ── TI service start / PID polling ──────────────────────────────────

fn start_and_get_ti_pid() -> Result<u32> {
    unsafe {
        let h_scm = OpenSCManagerW(null(), null(), SC_MANAGER_CONNECT);
        if h_scm.is_null() {
            return Err(NuError::InvalidOperation("cannot open SCM".into()));
        }

        let name = wide("TrustedInstaller");
        let h_svc = OpenServiceW(
            h_scm,
            name.as_ptr(),
            SERVICE_START | SERVICE_QUERY_STATUS,
        );
        if h_svc.is_null() {
            CloseServiceHandle(h_scm);
            return Err(NuError::InvalidOperation(
                "cannot open TrustedInstaller service".into(),
            ));
        }

        let _ = StartServiceW(h_svc, 0, null());
        let result = poll_service_pid(h_svc);

        CloseServiceHandle(h_svc);
        CloseServiceHandle(h_scm);
        result
    }
}

unsafe fn poll_service_pid(h_svc: SC_HANDLE) -> Result<u32> {
    for _ in 0..50 {
        let mut st: SERVICE_STATUS_PROCESS = zeroed();
        let mut needed: u32 = 0;

        let ok = QueryServiceStatusEx(
            h_svc,
            SC_STATUS_PROCESS_INFO,
            &mut st as *mut _ as *mut u8,
            size_of::<SERVICE_STATUS_PROCESS>() as u32,
            &mut needed,
        );

        if ok != 0 && st.dwCurrentState == SERVICE_RUNNING && st.dwProcessId != 0 {
            return Ok(st.dwProcessId);
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    Err(NuError::InvalidOperation(
        "TrustedInstaller service did not start in time".into(),
    ))
}

// ── privilege / util ────────────────────────────────────────────────

fn enable_debug_privilege() {
    unsafe {
        let mut h_tok: HANDLE = null_mut();
        if OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut h_tok,
        ) == 0
        {
            return;
        }

        let name = wide("SeDebugPrivilege");
        let mut tp: TOKEN_PRIVILEGES = zeroed();
        tp.PrivilegeCount = 1;
        LookupPrivilegeValueW(null(), name.as_ptr(), &mut tp.Privileges[0].Luid);
        tp.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;

        AdjustTokenPrivileges(h_tok, 0, &tp, 0, null_mut(), null_mut());
        CloseHandle(h_tok);
    }
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
