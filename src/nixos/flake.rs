use std::{
    fs,
    path::PathBuf,
};

use super::discovery::DiscoveryContext;

/// Discoveryが利用できない場合のメッセージ。
const DISCOVERY_ERROR_MESSAGE: &str =
    "NixOS flake discovery is not available. Run `nxc discover` first.";

/// 保存済みDiscoveryResultからNixOS flakeのパスを取得する。
///
/// 通常操作では自動探索を行わない。
pub fn repository_path()
    -> Result<PathBuf, String>
{
    let context =
        DiscoveryContext::load()
            .map_err(|_| {
                DISCOVERY_ERROR_MESSAGE
                    .to_string()
            })?;

    Ok(
        context
            .flake_root_buf()
    )
}

/// 保存済みDiscoveryResultからconfiguration名を取得する。
pub fn configuration_name()
    -> Result<String, String>
{
    let context =
        DiscoveryContext::load()
            .map_err(|_| {
                DISCOVERY_ERROR_MESSAGE
                    .to_string()
            })?;

    Ok(
        context
            .configuration_name()
            .to_string()
    )
}

/// DiscoveryContextを取得する。
///
/// 新しいコードではこちらを利用する。
pub fn discovery_context()
    -> Result<DiscoveryContext, String>
{
    DiscoveryContext::load()
}

/// packages.nix のパス。
pub fn pkgs_path()
    -> Result<PathBuf, String>
{
    Ok(
        repository_path()?
            .join("modules")
            .join("pkgs.nix")
    )
}

/// core.nix のパス。
fn core_path()
    -> Result<PathBuf, String>
{
    Ok(
        repository_path()?
            .join("modules")
            .join("core.nix")
    )
}

/// pkgs.nixを書き込む。
pub fn write_pkgs(
    content: &str,
) -> Result<(), String>
{
    fs::write(
        pkgs_path()?,
        content,
    )
    .map_err(|e| e.to_string())
}

/// core.nixを書き込む。
pub fn write_core(
    content: &str,
) -> Result<(), String>
{
    fs::write(
        core_path()?,
        content,
    )
    .map_err(|e| e.to_string())
}

/// pkgs.nixを読む。
pub fn read_pkgs()
    -> Result<String, String>
{
    fs::read_to_string(
        pkgs_path()?
    )
    .map_err(|e| e.to_string())
}

/// core.nixを読む。
pub fn read_core()
    -> Result<String, String>
{
    fs::read_to_string(
        core_path()?
    )
    .map_err(|e| e.to_string())
}