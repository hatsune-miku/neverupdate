use std::process::{Command, Stdio};

use crate::error::{NuError, Result};

pub fn run_command(program: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
    }

    let error = String::from_utf8_lossy(&output.stderr).to_string();
    Err(NuError::CommandFailed(format!(
        "{program} {:?} => {}",
        args,
        error.trim()
    )))
}
