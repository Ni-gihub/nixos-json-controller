use std::process::Command;

use super::flake;

pub fn build_command() -> Result<Command, String> {
    let flake_path = flake::repository_path()?;

    let flake = format!("{}#laptop", flake_path.display());

    let mut command = Command::new("sudo");

    command.args(["nixos-rebuild", "switch", "--flake", &flake]);

    Ok(command)
}

pub fn switch() -> Result<(), String> {
    let status = build_command()?.status().map_err(|e| e.to_string())?;

    if status.success() {
        Ok(())
    } else {
        Err("nixos-rebuild failed".to_string())
    }
}
