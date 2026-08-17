use std::process::Command;

use super::flake;



pub fn build_command() -> Command {

    let mut command =
        Command::new(
            "sudo"
        );

    let flake =
        format!(
            "{}#laptop",
            flake::repository_path()
        );

    command.args([
        "nixos-rebuild",
        "switch",
        "--flake",
        &flake,
    ]);

    command
}


pub fn switch() -> Result<(), String> {

    let status =
        build_command()
            .status()
            .map_err(
                |e| e.to_string()
            )?;

    if status.success() {

        Ok(())

    } else {

        Err(
            "nixos-rebuild failed"
                .to_string()
        )

    }

}