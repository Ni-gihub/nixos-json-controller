use std::fs;
use std::path::PathBuf;

use super::DiscoveryResult;

const STATE_DIRECTORY: &str = "nxc";
const STATE_FILE: &str = "discovery.json";

/// Discovery状態の保存先を取得する
///
/// Linuxでは `XDG_CONFIG_HOME` が設定されていればそれを使用し、
/// 未設定の場合は `$HOME/.config` を使用する。
pub fn state_path() -> Result<PathBuf, String> {
    let config_home = match std::env::var_os("XDG_CONFIG_HOME") {
        Some(path) => PathBuf::from(path),

        None => {
            let home = std::env::var_os("HOME")
                .ok_or(
                    "HOME environment variable is not set"
                        .to_string()
                )?;

            PathBuf::from(home)
                .join(".config")
        }
    };

    Ok(config_home
        .join(STATE_DIRECTORY)
        .join(STATE_FILE))
}

/// DiscoveryResultを保存する
pub fn save_discovery(
    result: &DiscoveryResult,
) -> Result<(), String> {
    let path = state_path()?;

    let parent = path
        .parent()
        .ok_or(
            "failed to determine state directory"
                .to_string()
        )?;

    fs::create_dir_all(parent)
        .map_err(|e| {
            format!(
                "failed to create state directory {}: {}",
                parent.display(),
                e
            )
        })?;

    let json = serde_json::to_string_pretty(result)
        .map_err(|e| {
            format!(
                "failed to serialize discovery result: {}",
                e
            )
        })?;

    fs::write(&path, json)
        .map_err(|e| {
            format!(
                "failed to write discovery state {}: {}",
                path.display(),
                e
            )
        })?;

    Ok(())
}

/// 保存済みDiscoveryResultを読み込む
pub fn load_discovery() -> Result<DiscoveryResult, String> {
    let path = state_path()?;

    let content = fs::read_to_string(&path)
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                format!(
                    "discovery state not found: {}",
                    path.display()
                )
            } else {
                format!(
                    "failed to read discovery state {}: {}",
                    path.display(),
                    e
                )
            }
        })?;

    serde_json::from_str(&content)
        .map_err(|e| {
            format!(
                "failed to parse discovery state {}: {}",
                path.display(),
                e
            )
        })
}

/// 保存済みDiscoveryResultが存在するか確認する
pub fn discovery_exists() -> Result<bool, String> {
    Ok(state_path()?.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_path_is_under_config_directory() {
        let path =
            state_path()
                .expect(
                    "failed to get state path"
                );

        assert!(
            path.ends_with(
                "nxc/discovery.json"
            )
        );
    }
}