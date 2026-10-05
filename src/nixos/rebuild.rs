use std::io::Write;
use std::process::{Command, Stdio};

use super::discovery::DiscoveryContext;

/// DiscoveryContextからrebuildコマンドを作る。
pub fn build_command_with_context(context: &DiscoveryContext) -> Result<Command, String> {
    let flake = context.flake_reference();

    let mut command = Command::new("sudo");

    command.args(["nixos-rebuild", "switch", "--flake", &flake]);

    Ok(command)
}

/// 保存済みDiscoveryからrebuildコマンドを作る。
pub fn build_command() -> Result<Command, String> {
    let context = DiscoveryContext::load()?;

    build_command_with_context(&context)
}

/// NixOSをswitchする。
pub fn switch() -> Result<(), String> {
    let context = DiscoveryContext::load()?;

    switch_with_context(&context)
}

/// DiscoveryContextを再利用してswitchする。
pub fn switch_with_context(context: &DiscoveryContext) -> Result<(), String> {
    switch_with_password_with_context(context, None)
}

/// 保存済みDiscoveryを使い、必要ならsudoへパスワードを標準入力で渡してswitchする。
pub fn switch_with_password(password: Option<&str>) -> Result<(), String> {
    let context = DiscoveryContext::load()?;

    switch_with_password_with_context(&context, password)
}

/// DiscoveryContextを再利用してswitchする。
pub fn switch_with_password_with_context(
    context: &DiscoveryContext,
    password: Option<&str>,
) -> Result<(), String> {
    let flake = context.flake_reference();

    let mut command = Command::new("sudo");

    if password.is_some() {
        command.args(["-S", "-p", ""]);
    }

    command.args(["nixos-rebuild", "switch", "--flake", &flake]);

    if let Some(password) = password {
        command.stdin(Stdio::piped());

        let mut child = command.spawn().map_err(|e| e.to_string())?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(format!("{password}\n").as_bytes())
                .map_err(|e| e.to_string())?;
        }

        let status = child.wait().map_err(|e| e.to_string())?;

        if status.success() {
            Ok(())
        } else {
            Err("nixos-rebuild failed".to_string())
        }
    } else {
        let status = command.status().map_err(|e| e.to_string())?;

        if status.success() {
            Ok(())
        } else {
            Err("nixos-rebuild failed".to_string())
        }
    }
}

/// sudoが現在パスワードなしで利用可能か確認する。
pub fn sudo_cached() -> bool {
    Command::new("sudo")
        .args(["-n", "-v"])
        .status()
        .is_ok_and(|status| status.success())
}
