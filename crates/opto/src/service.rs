//! `opto setup service` and `opto remove service`: enable the collector's
//! systemd user service and start it, or stop and disable it.
//!
//! Setup restarts a running collector only when the installed binary differs
//! from the one it runs, since every restart leaves a gap in the charts.

use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::cli::Action;

const UNIT: &str = "opto-collect.service";

pub fn service(action: Action) -> i32 {
    match action {
        Action::Setup => setup(),
        Action::Remove => remove(),
    }
}

fn setup() -> i32 {
    for args in [&["daemon-reload"][..], &["enable", UNIT]] {
        if let Err(code) = run(args) {
            return code;
        }
    }
    let step = match running_pid() {
        None => Step::Start,
        Some(pid) if runs_installed_binary(pid) => Step::Leave,
        Some(_) => Step::Restart,
    };
    let message = match step {
        Step::Start => "started the collector, which now starts at login",
        Step::Restart => "restarted the collector on the new binary; it starts at login",
        Step::Leave => "the collector is already running the installed binary; left it running",
    };
    if let Some(verb) = step.verb()
        && let Err(code) = run(&[verb, UNIT])
    {
        return code;
    }
    println!("opto: {message}");
    0
}

fn remove() -> i32 {
    if let Err(code) = run(&["disable", "--now", UNIT]) {
        return code;
    }
    println!("opto: stopped the collector; its data is kept");
    0
}

#[derive(Debug, PartialEq)]
enum Step {
    Start,
    Restart,
    Leave,
}

impl Step {
    fn verb(&self) -> Option<&'static str> {
        match self {
            Step::Start => Some("start"),
            Step::Restart => Some("restart"),
            Step::Leave => None,
        }
    }
}

/// The collector's main process, if it is running.
fn running_pid() -> Option<u32> {
    show("MainPID")?.trim().parse().ok().filter(|&pid| pid != 0)
}

/// Whether the process runs the same bytes as the file the unit would start
/// now. `/proc/<pid>/exe` still reads the old binary after it was replaced on
/// disk. When in doubt the answer is no, and the collector is restarted.
fn runs_installed_binary(pid: u32) -> bool {
    let Some(installed) = show("ExecStart").and_then(|value| exec_path(&value)) else {
        return false;
    };
    same_contents(format!("/proc/{pid}/exe").as_ref(), &installed).unwrap_or(false)
}

/// Compares two files by size, then chunk by chunk, stopping at the first
/// difference.
fn same_contents(a: &Path, b: &Path) -> io::Result<bool> {
    let (mut a, mut b) = (File::open(a)?, File::open(b)?);
    if a.metadata()?.len() != b.metadata()?.len() {
        return Ok(false);
    }
    let (mut chunk_a, mut chunk_b) = ([0; 64 * 1024], [0; 64 * 1024]);
    loop {
        let n = fill(&mut a, &mut chunk_a)?;
        if n != fill(&mut b, &mut chunk_b)? || chunk_a[..n] != chunk_b[..n] {
            return Ok(false);
        }
        if n == 0 {
            return Ok(true);
        }
    }
}

/// Reads until `buf` is full or the file ends, returning how much was read.
fn fill(file: &mut File, buf: &mut [u8]) -> io::Result<usize> {
    let mut filled = 0;
    while filled < buf.len() {
        match file.read(&mut buf[filled..])? {
            0 => break,
            n => filled += n,
        }
    }
    Ok(filled)
}

/// The program in a `systemctl show -p ExecStart` value, which looks like
/// `{ path=/usr/bin/opto-collect ; argv[]=/usr/bin/opto-collect ; ... }`.
fn exec_path(value: &str) -> Option<PathBuf> {
    let rest = value.split_once("path=")?.1;
    let path = rest.split(" ;").next()?.trim();
    (!path.is_empty()).then(|| PathBuf::from(path))
}

fn show(property: &str) -> Option<String> {
    let output = Command::new("systemctl")
        .args(["--user", "show", "--value", "-p", property, UNIT])
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Runs `systemctl --user` with `args`, reporting a failure as exit code 1.
fn run(args: &[&str]) -> Result<(), i32> {
    systemctl(args).map_err(|e| {
        eprintln!("opto: systemctl --user {}: {e}", args.join(" "));
        1
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_program_from_exec_start() {
        let value = "{ path=/home/me/.local/bin/opto-collect ; argv[]=/home/me/.local/bin/opto-collect ; ignore_errors=no ; start_time=[n/a] ; stop_time=[n/a] ; pid=0 ; code=(null) ; status=0/0 }\n";
        assert_eq!(
            exec_path(value),
            Some(PathBuf::from("/home/me/.local/bin/opto-collect"))
        );
    }

    #[test]
    fn has_no_program_without_exec_start() {
        assert_eq!(exec_path(""), None);
        assert_eq!(exec_path("\n"), None);
        assert_eq!(exec_path("{ path= ; argv[]= }"), None);
    }

    fn temp_file(name: &str, contents: &[u8]) -> PathBuf {
        let path = std::env::temp_dir().join(format!("opto-{}-{name}", std::process::id()));
        std::fs::write(&path, contents).unwrap();
        path
    }

    #[test]
    fn compares_files_by_contents() {
        let big: Vec<u8> = (0..200_000).map(|i| i as u8).collect();
        let mut late_change = big.clone();
        *late_change.last_mut().unwrap() ^= 1;
        let original = temp_file("original", &big);
        let copy = temp_file("copy", &big);
        let changed = temp_file("changed", &late_change);
        let shorter = temp_file("shorter", &big[..big.len() - 1]);

        assert!(same_contents(&original, &copy).unwrap());
        assert!(!same_contents(&original, &changed).unwrap());
        assert!(!same_contents(&original, &shorter).unwrap());
        assert!(same_contents(&original, Path::new("/nonexistent/opto")).is_err());

        for path in [original, copy, changed, shorter] {
            std::fs::remove_file(path).unwrap();
        }
    }
}
