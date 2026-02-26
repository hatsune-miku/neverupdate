use crate::command::run_command;
use crate::error::Result;

const DAEMON_SERVICE_NAME: &str = "NeverUpdateDaemon";

pub fn daemon_service_name() -> &'static str {
    DAEMON_SERVICE_NAME
}

pub fn daemon_service_exists() -> bool {
    run_command("sc", &["query", DAEMON_SERVICE_NAME]).is_ok()
}

pub fn register_daemon_service(exe_path: &str) -> Result<()> {
    let bin = format!("\"{}\" run", exe_path);

    let _ = run_command(
        "sc",
        &[
            "create",
            DAEMON_SERVICE_NAME,
            &format!("binPath= {bin}"),
            "start= auto",
            "DisplayName= NeverUpdate Daemon",
        ],
    )?;

    let _ = run_command(
        "sc",
        &[
            "description",
            DAEMON_SERVICE_NAME,
            "NeverUpdate privileged maintenance daemon",
        ],
    );
    Ok(())
}

pub fn unregister_daemon_service() -> Result<()> {
    let _ = stop_daemon_service();
    let _ = run_command("sc", &["delete", DAEMON_SERVICE_NAME])?;
    Ok(())
}

pub fn reregister_daemon_service(exe_path: &str) -> Result<()> {
    if daemon_service_exists() {
        let _ = unregister_daemon_service();
    }

    register_daemon_service(exe_path)
}

pub fn start_daemon_service() -> Result<()> {
    let _ = run_command("sc", &["start", DAEMON_SERVICE_NAME])?;
    Ok(())
}

pub fn stop_daemon_service() -> Result<()> {
    let _ = run_command("sc", &["stop", DAEMON_SERVICE_NAME])?;
    Ok(())
}
