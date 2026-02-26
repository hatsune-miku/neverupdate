use std::process::Command;

use crate::error::{NuError, Result};
use crate::guards::service_guard;
use crate::pathing::software_distribution_path;
use crate::state::{load_state, save_state};

pub fn run_extreme_mode() -> Result<()> {
    let mut state = load_state()?;
    service_guard::extreme_disable_all_services(&mut state)?;
    save_state(&state)?;

    let target = software_distribution_path();
    if target.exists() {
        if target.is_dir() {
            std::fs::remove_dir_all(&target)?;
        } else {
            std::fs::remove_file(&target)?;
        }
    }

    std::fs::write(
        &target,
        b"NeverUpdate keeps this path unavailable for Windows Update",
    )?;

    let s_target = target.to_string_lossy().to_string();
    let s_user = std::env::var("USERNAME").unwrap_or_else(|_| String::from("Users"));

    run_icacls(&[&s_target, "/inheritance:r"])?;
    run_icacls(&[&s_target, "/setowner", &s_user])?;
    run_icacls(&[&s_target, "/grant:r", &format!("{s_user}:F")])?;

    Ok(())
}

fn run_icacls(args: &[&str]) -> Result<()> {
    let output = Command::new("icacls").args(args).output()?;
    if output.status.success() {
        return Ok(());
    }

    let error = String::from_utf8_lossy(&output.stderr).to_string();
    Err(NuError::CommandFailed(format!(
        "icacls {:?} => {}",
        args,
        error.trim()
    )))
}
