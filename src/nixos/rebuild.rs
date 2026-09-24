use std::process::Command;

use super::discovery::DiscoveryContext;

/// DiscoveryContextからrebuildコマンドを作る。
pub fn build_command_with_context(
    context: &DiscoveryContext,
) -> Result<Command, String> {
    let flake =
        context.flake_reference();

    let mut command =
        Command::new("sudo");

    command.args([
        "nixos-rebuild",
        "switch",
        "--flake",
        &flake,
    ]);

    Ok(command)
}

/// 保存済みDiscoveryからrebuildコマンドを作る。
pub fn build_command()
    -> Result<Command, String>
{
    let context =
        DiscoveryContext::load()?;

    build_command_with_context(
        &context
    )
}

/// NixOSをswitchする。
pub fn switch()
    -> Result<(), String>
{
    let context =
        DiscoveryContext::load()?;

    switch_with_context(
        &context
    )
}

/// DiscoveryContextを再利用してswitchする。
pub fn switch_with_context(
    context: &DiscoveryContext,
) -> Result<(), String>
{
    let status =
        build_command_with_context(
            context
        )?
        .status()
        .map_err(|e| e.to_string())?;

    if status.success() {
        Ok(())
    } else {
        Err(
            "nixos-rebuild failed"
                .to_string()
        )
    }
}