//! `opto setup service` and `opto remove service`: enable the collector's
//! systemd user service and restart it, or stop and disable it.

use std::process::Command;

use crate::cli::Action;

const UNIT: &str = "opto-collect.service";

pub fn service(action: Action) -> i32 {
    let steps: &[&[&str]] = match action {
        Action::Setup => &[&["daemon-reload"], &["enable", UNIT], &["restart", UNIT]],
        Action::Remove => &[&["disable", "--now", UNIT]],
    };
    for args in steps {
        if let Err(e) = systemctl(args) {
            eprintln!("opto: systemctl --user {}: {e}", args.join(" "));
            return 1;
        }
    }
    match action {
        Action::Setup => println!("opto: started the collector, which now starts at login"),
        Action::Remove => println!("opto: stopped the collector; its data is kept"),
    }
    0
}

fn systemctl(args: &[&str]) -> Result<(), String> {
    let status = Command::new("systemctl")
        .arg("--user")
        .args(args)
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(status.to_string())
    }
}
