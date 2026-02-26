use crate::command::run_command;
use crate::error::Result;
use crate::guards::status_hosts_firewall;
use crate::model::GuardPointStatus;
use crate::pathing::hosts_file_path;

const MARK_BEGIN: &str = "# NeverUpdate BEGIN";
const MARK_END: &str = "# NeverUpdate END";
const FIREWALL_RULE_NAME: &str = "NeverUpdate Block Windows Update Domains";

const DOMAINS: [&str; 3] = [
    "*.windowsupdate.com",
    "*.update.microsoft.com",
    "*.delivery.mp.microsoft.com",
];

pub fn check() -> Result<GuardPointStatus> {
    let hosts_ok = check_hosts_content()?;
    let firewall_ok = check_firewall_rule();
    let guarded = hosts_ok && firewall_ok;
    let detail = format!("hosts={hosts_ok}, firewall={firewall_ok}");

    Ok(status_hosts_firewall(guarded, Some(detail)))
}

pub fn guard() -> Result<GuardPointStatus> {
    ensure_hosts_entries()?;
    ensure_firewall_rule()?;
    check()
}

pub fn release() -> Result<GuardPointStatus> {
    remove_hosts_entries()?;
    remove_firewall_rule()?;
    check()
}

fn check_hosts_content() -> Result<bool> {
    let path = hosts_file_path()?;
    let content = std::fs::read_to_string(path)?;

    if !content.contains(MARK_BEGIN) || !content.contains(MARK_END) {
        return Ok(false);
    }

    Ok(DOMAINS
        .iter()
        .all(|domain| content.contains(&format!("127.0.0.1 {domain}"))))
}

fn ensure_hosts_entries() -> Result<()> {
    let path = hosts_file_path()?;
    let mut content = std::fs::read_to_string(&path)?;

    if content.contains(MARK_BEGIN) && content.contains(MARK_END) {
        return Ok(());
    }

    if !content.ends_with('\n') {
        content.push('\n');
    }

    content.push_str(MARK_BEGIN);
    content.push('\n');
    for domain in DOMAINS {
        content.push_str(&format!("127.0.0.1 {domain}\n"));
    }
    content.push_str(MARK_END);
    content.push('\n');

    std::fs::write(path, content)?;
    Ok(())
}

fn remove_hosts_entries() -> Result<()> {
    let path = hosts_file_path()?;
    let content = std::fs::read_to_string(&path)?;

    if !content.contains(MARK_BEGIN) || !content.contains(MARK_END) {
        return Ok(());
    }

    let mut result = Vec::new();
    let mut inside = false;
    for line in content.lines() {
        if line.trim() == MARK_BEGIN {
            inside = true;
            continue;
        }

        if line.trim() == MARK_END {
            inside = false;
            continue;
        }

        if !inside {
            result.push(line.to_string());
        }
    }

    let final_content = format!("{}\n", result.join("\n"));
    std::fs::write(path, final_content)?;
    Ok(())
}

fn check_firewall_rule() -> bool {
    let command = format!(
    "$rule = Get-NetFirewallRule -DisplayName '{FIREWALL_RULE_NAME}' -ErrorAction SilentlyContinue; if ($null -eq $rule) {{ '0' }} else {{ '1' }}"
  );

    if let Ok(output) = run_command("powershell", &["-NoProfile", "-Command", &command]) {
        return output.trim() == "1";
    }

    false
}

fn ensure_firewall_rule() -> Result<()> {
    let domains_joined = DOMAINS
        .iter()
        .map(|item| format!("'{item}'"))
        .collect::<Vec<String>>()
        .join(",");

    let command = format!(
    "if (-not (Get-NetFirewallRule -DisplayName '{FIREWALL_RULE_NAME}' -ErrorAction SilentlyContinue)) {{ New-NetFirewallRule -DisplayName '{FIREWALL_RULE_NAME}' -Direction Outbound -Action Block -RemoteFqdn {domains_joined} | Out-Null }}"
  );

    let _ = run_command("powershell", &["-NoProfile", "-Command", &command])?;
    Ok(())
}

fn remove_firewall_rule() -> Result<()> {
    let command = format!("Get-NetFirewallRule -DisplayName '{FIREWALL_RULE_NAME}' -ErrorAction SilentlyContinue | Remove-NetFirewallRule");
    let _ = run_command("powershell", &["-NoProfile", "-Command", &command])?;
    Ok(())
}
