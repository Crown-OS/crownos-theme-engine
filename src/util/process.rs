use std::env;
use std::ffi::OsStr;
use std::process::{Command, Stdio};

use crate::error::TargetError;

pub fn is_on_path(program: &str) -> bool {
    env::var_os("PATH")
        .is_some_and(|paths| env::split_paths(&paths).any(|dir| dir.join(program).is_file()))
}

/// Runs a program to completion with no stdin, failing on a non-zero exit.
pub fn run(
    program: &str,
    args: impl IntoIterator<Item = impl AsRef<OsStr>>,
) -> Result<(), TargetError> {
    let failed = |detail: String| TargetError::Command {
        program: program.to_owned(),
        detail,
    };
    let output = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| failed(error.to_string()))?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(failed(match stderr.trim() {
        "" => output.status.to_string(),
        message => message.to_owned(),
    }))
}

/// Sends `signal` to every process named exactly `name`; none running is fine.
pub fn signal_all(name: &str, signal: &str) -> Result<(), TargetError> {
    const NO_PROCESS_MATCHED: i32 = 1;
    let status = Command::new("pkill")
        .args([&format!("-{signal}"), "-x", name])
        .stdin(Stdio::null())
        .status()
        .map_err(|error| TargetError::Command {
            program: "pkill".to_owned(),
            detail: error.to_string(),
        })?;
    match status.code() {
        Some(0 | NO_PROCESS_MATCHED) => Ok(()),
        _ => Err(TargetError::Command {
            program: "pkill".to_owned(),
            detail: status.to_string(),
        }),
    }
}
