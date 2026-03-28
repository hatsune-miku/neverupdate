mod service_host;

use std::time::Duration;

use nu_core::{
    acquire_global_instance, daemon_service_name, load_daemon_snapshot, run_maintenance_cycle,
    register_daemon_service, reregister_daemon_service, run_preflight_checks, start_daemon_service,
    stop_daemon_service, store_daemon_snapshot, unregister_daemon_service,
};

use service_host::DispatcherOutcome;

enum Command {
    Run,
    Once,
    Snapshot,
    ServiceInstall,
    ServiceReinstall,
    ServiceUninstall,
    ServiceStart,
    ServiceStop,
}

fn parse_args() -> Command {
    let mut args = pico_args::Arguments::from_env();
    let sub = args.subcommand().ok().flatten();

    match sub.as_deref() {
        Some("once") => Command::Once,
        Some("snapshot") => Command::Snapshot,
        Some("service-install") => Command::ServiceInstall,
        Some("service-reinstall") => Command::ServiceReinstall,
        Some("service-uninstall") => Command::ServiceUninstall,
        Some("service-start") => Command::ServiceStart,
        Some("service-stop") => Command::ServiceStop,
        _ => Command::Run,
    }
}

fn print_help() {
    println!("NeverUpdate daemon");
    println!("Usage:");
    println!("  neverupdate-daemon run");
    println!("  neverupdate-daemon once");
    println!("  neverupdate-daemon snapshot");
    println!("  neverupdate-daemon service-install");
    println!("  neverupdate-daemon service-reinstall");
    println!("  neverupdate-daemon service-uninstall");
    println!("  neverupdate-daemon service-start");
    println!("  neverupdate-daemon service-stop");
}

fn current_exe_string() -> std::result::Result<String, String> {
    std::env::current_exe()
        .map(|path| path.to_string_lossy().to_string())
        .map_err(|error| error.to_string())
}

fn run_once_cycle() -> std::result::Result<(), String> {
    let report = run_preflight_checks();
    if !report.passed {
        return Err(format!("preflight failed: {:?}", report.checks));
    }

    let _instance = acquire_global_instance("Global\\NeverUpdateDaemonMain")
        .map_err(|error| error.to_string())?;
    let summary = run_maintenance_cycle();

    let message = if summary.errors.is_empty() {
        None
    } else {
        Some(summary.errors.join(" | "))
    };

    store_daemon_snapshot(summary.statuses.clone(), message).map_err(|error| error.to_string())?;
    println!(
        "{}",
        serde_json::to_string_pretty(&summary).unwrap_or_else(|_| String::from("{}"))
    );
    Ok(())
}

/// Main loop for both SCM service and interactive `run` (after dispatcher returns 1063).
pub fn run_daemon_work() -> std::result::Result<(), String> {
    let report = run_preflight_checks();
    if !report.passed {
        return Err(format!("preflight failed: {:?}", report.checks));
    }

    let _instance = acquire_global_instance("Global\\NeverUpdateDaemonMain")
        .map_err(|error| error.to_string())?;

    loop {
        if service_host::stop_requested() {
            break;
        }

        let summary = run_maintenance_cycle();
        let message = if summary.errors.is_empty() {
            None
        } else {
            Some(summary.errors.join(" | "))
        };

        let _ = store_daemon_snapshot(summary.statuses.clone(), message);

        for _ in 0..45 {
            if service_host::stop_requested() {
                return Ok(());
            }
            std::thread::sleep(Duration::from_secs(1));
        }
    }
    Ok(())
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let command = parse_args();

    let result = match command {
        Command::Run => match service_host::enter_dispatcher() {
            DispatcherOutcome::Console => run_daemon_work(),
            DispatcherOutcome::ServiceStopped => Ok(()),
        },
        Command::Once => run_once_cycle(),
        Command::Snapshot => match load_daemon_snapshot() {
            Ok(Some(snapshot)) => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&snapshot).unwrap_or_else(|_| String::from("{}"))
                );
                Ok(())
            }
            Ok(None) => {
                println!("null");
                Ok(())
            }
            Err(error) => Err(error.to_string()),
        },
        Command::ServiceInstall => {
            let exe = current_exe_string()?;
            register_daemon_service(&exe).map_err(|error| error.to_string())?;
            println!("service {} installed", daemon_service_name());
            Ok(())
        }
        Command::ServiceReinstall => {
            let exe = current_exe_string()?;
            reregister_daemon_service(&exe).map_err(|error| error.to_string())?;
            println!("service {} reinstalled", daemon_service_name());
            Ok(())
        }
        Command::ServiceUninstall => {
            unregister_daemon_service().map_err(|error| error.to_string())?;
            println!("service {} uninstalled", daemon_service_name());
            Ok(())
        }
        Command::ServiceStart => {
            start_daemon_service().map_err(|error| error.to_string())?;
            println!("service {} started", daemon_service_name());
            Ok(())
        }
        Command::ServiceStop => {
            stop_daemon_service().map_err(|error| error.to_string())?;
            println!("service {} stopped", daemon_service_name());
            Ok(())
        }
    };

    if let Err(error) = result {
        eprintln!("{error}");
        print_help();
        std::process::exit(1);
    }

    Ok(())
}
