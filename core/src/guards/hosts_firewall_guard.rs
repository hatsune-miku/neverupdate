use crate::command::run_command;
use crate::error::Result;
use crate::guards::GuardPoint;
use crate::model::GuardPointStatus;
use crate::pathing::hosts_file_path;
use crate::state::PersistedState;
use crate::ti_service::TiService;

const MARK_BEGIN: &str = "# NeverUpdate BEGIN";
const MARK_END: &str = "# NeverUpdate END";

const DOMAINS: [&str; 3] = [
    "*.windowsupdate.com",
    "*.update.microsoft.com",
    "*.delivery.mp.microsoft.com",
];

const RULE_PREFIX: &str = "NeverUpdate_Block_";
const BLOCKED_PROGRAMS: [(&str, &str); 3] = [
    ("WaaSMedic", "%SystemRoot%\\System32\\WaaSMedicAgent.exe"),
    ("UsoClient", "%SystemRoot%\\System32\\UsoClient.exe"),
    ("musNotify", "%SystemRoot%\\System32\\musNotification.exe"),
];

pub struct HostsFirewallGuard;

impl GuardPoint for HostsFirewallGuard {
    fn id(&self) -> &'static str {
        "hosts_firewall"
    }

    fn title(&self) -> &'static str {
        "Hosts 与防火墙"
    }

    fn description(&self) -> &'static str {
        "锁定更新域名到 127.0.0.1，并阻止更新程序出站"
    }

    fn interception_behavior(&self) -> Option<&'static str> {
        Some("系统试图暗改Hosts以恢复更新")
    }

    fn check(&self, _state: &PersistedState) -> Result<GuardPointStatus> {
        let hosts_ok = check_hosts_content().unwrap_or(false);
        let firewall_ok = check_firewall_rules();
        let guarded = hosts_ok && firewall_ok;
        Ok(self.build_status(guarded, Some(format!("hosts={hosts_ok}, firewall={firewall_ok}"))))
    }

    fn guard(&self, _state: &mut PersistedState, ti: &TiService) -> Result<GuardPointStatus> {
        ensure_hosts_entries(ti)?;
        ti.as_admin(|| ensure_firewall_rules())?;
        ti.as_admin(|| self.check(_state))
    }

    fn release(&self, _state: &mut PersistedState, ti: &TiService) -> Result<GuardPointStatus> {
        remove_hosts_entries(ti)?;
        ti.as_admin(|| remove_firewall_rules())?;
        ti.as_admin(|| self.check(_state))
    }
}

// ── hosts ──

fn check_hosts_content() -> Result<bool> {
    let path = hosts_file_path()?;
    let content = std::fs::read_to_string(path)?;

    if !content.contains(MARK_BEGIN) || !content.contains(MARK_END) {
        return Ok(false);
    }

    Ok(DOMAINS
        .iter()
        .all(|d| content.contains(&format!("127.0.0.1 {d}"))))
}

fn ensure_hosts_entries(ti: &TiService) -> Result<()> {
    let path = hosts_file_path()?;
    let mut content = ti.read_file_string(&path)?;

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

    ti.write_file_string(&path, &content)
}

fn remove_hosts_entries(ti: &TiService) -> Result<()> {
    let path = hosts_file_path()?;
    let content = ti.read_file_string(&path)?;

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

    ti.write_file_string(&path, &format!("{}\n", result.join("\n")))
}

// ── firewall via netsh (program-based outbound block) ──

fn rule_name(tag: &str) -> String {
    format!("{RULE_PREFIX}{tag}")
}

fn check_firewall_rules() -> bool {
    BLOCKED_PROGRAMS.iter().all(|(tag, _)| {
        run_command("netsh", &[
            "advfirewall", "firewall", "show", "rule",
            &format!("name={}", rule_name(tag)),
        ]).is_ok()
    })
}

fn ensure_firewall_rules() -> Result<()> {
    for (tag, prog) in BLOCKED_PROGRAMS {
        let name = rule_name(tag);
        if run_command("netsh", &[
            "advfirewall", "firewall", "show", "rule",
            &format!("name={name}"),
        ]).is_ok() {
            continue;
        }
        run_command("netsh", &[
            "advfirewall", "firewall", "add", "rule",
            &format!("name={name}"),
            "dir=out", "action=block",
            &format!("program={prog}"),
        ])?;
    }
    Ok(())
}

fn remove_firewall_rules() -> Result<()> {
    for (tag, _) in BLOCKED_PROGRAMS {
        let _ = run_command("netsh", &[
            "advfirewall", "firewall", "delete", "rule",
            &format!("name={}", rule_name(tag)),
        ]);
    }
    Ok(())
}
