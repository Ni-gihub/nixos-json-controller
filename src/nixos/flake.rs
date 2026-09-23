use std::{
    fs,
    path::PathBuf,
};

use super::discovery::load_discovery;

const DISCOVERY_ERROR_MESSAGE: &str =
    "NixOS flake discovery is not available. Run `nxc discover` first.";

/// 保存済みDiscoveryResultからNixOS configuration flakeのパスを取得する。
///
/// 通常のnxc操作では自動探索を行わない。
/// `nxc discover`で保存されたDiscoveryResultだけを使用する。
pub fn repository_path() -> Result<PathBuf, String> {
    let discovery = load_discovery().map_err(|_| {
        DISCOVERY_ERROR_MESSAGE.to_string()
    })?;

    let path = discovery.flake_root;

    if !path.is_dir() {
        return Err(format!(
            "discovered NixOS flake directory does not exist: {}. Run `nxc discover` again.",
            path.display()
        ));
    }

    let flake_file = path.join("flake.nix");

    if !flake_file.is_file() {
        return Err(format!(
            "discovered NixOS flake file does not exist: {}. Run `nxc discover` again.",
            flake_file.display()
        ));
    }

    Ok(path)
}

/// 保存済みDiscoveryResultから選択されたNixOS configuration名を取得する。
pub fn configuration_name() -> Result<String, String> {
    let discovery = load_discovery().map_err(|_| {
        DISCOVERY_ERROR_MESSAGE.to_string()
    })?;

    let name = discovery
        .selected_configuration
        .name;

    if name.trim().is_empty() {
        return Err(
            "discovered NixOS configuration name is empty. Run `nxc discover` again."
                .to_string()
        );
    }

    Ok(name)
}

/// packages.nix のパス。
pub fn pkgs_path() -> Result<PathBuf, String> {
    Ok(
        repository_path()?
            .join("modules")
            .join("pkgs.nix")
    )
}

/// core.nix のパス。
fn core_path() -> Result<PathBuf, String> {
    Ok(
        repository_path()?
            .join("modules")
            .join("core.nix")
    )
}

/// pkgs.nix を書き込む。
pub fn write_pkgs(
    content: &str,
) -> Result<(), String> {
    fs::write(
        pkgs_path()?,
        content,
    )
    .map_err(|e| e.to_string())
}

/// core.nix を書き込む。
pub fn write_core(
    content: &str,
) -> Result<(), String> {
    fs::write(
        core_path()?,
        content,
    )
    .map_err(|e| e.to_string())
}

/// pkgs.nix を読む。
pub fn read_pkgs() -> Result<String, String> {
    fs::read_to_string(
        pkgs_path()?
    )
    .map_err(|e| e.to_string())
}

/// core.nix を読む。
pub fn read_core() -> Result<String, String> {
    fs::read_to_string(
        core_path()?
    )
    .map_err(|e| e.to_string())
}