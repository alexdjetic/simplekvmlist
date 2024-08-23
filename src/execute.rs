use std::process::{Command, Output};
use std::str;

/// A structure to hold the result of a command execution.
pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub status: i32,
}

/// Executes a given command and captures its output and status.
pub fn execute_cmd(command: &str) -> CommandResult {
    // Create a new command and execute it, capturing the output
    let output: Output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("failed to execute process");

    // Convert the stdout and stderr from bytes to strings
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Get the exit status code, defaulting to -1 if not available
    let status = output.status.code().unwrap_or(-1);

    // Return a CommandResult struct containing the command's output and status
    CommandResult {
        stdout,
        stderr,
        status,
    }
}

pub fn get_euid() -> Option<u32> {
    // Execute the 'id -u' command to get the effective user ID
    let result = execute_cmd("id -u");

    // Check if the command was successful
    if result.status == 0 {
        // Convert the output to a string and trim whitespace
        let euid_str = result.stdout.trim();

        // Parse the string to a u32
        euid_str.parse().ok()
    } else {
        None
    }
}

pub fn get_permission_valid() -> bool {
    match get_euid() {
        Some(euid) => euid == 0,
        None => false, // Failed to retrieve EUID
    }
}
