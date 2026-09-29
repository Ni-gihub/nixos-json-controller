use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

/// NixOSの現在世代から見えるsystem-wideな実行ファイル。
///
/// NixOSではsystem-wideなパッケージのバイナリが
/// /run/current-system/sw/bin に公開される。
#[derive(Debug, Clone)]
pub struct SystemState {
    pub current_generation: PathBuf,
    pub binaries: BTreeMap<String, PathBuf>,
    pub packages: BTreeSet<String>,
    pub enabled_services: BTreeSet<String>,
}

impl SystemState {
    /// 現在起動中のNixOS system generationから状態を取得する。
    ///
    /// 設定ファイルの再探索やNix評価は行わない。
    /// 現在のsystem generationとsystemdの状態だけを読む。
    pub fn discover() -> Result<Self, String> {
        let current_system = Path::new("/run/current-system");

        if !current_system.exists() {
            return Err(
                "current NixOS system generation was not found at /run/current-system".to_string(),
            );
        }

        let current_generation = std::fs::canonicalize(current_system)
            .map_err(|e| format!("failed to resolve /run/current-system: {}", e))?;

        let binaries = discover_binaries(Path::new("/run/current-system/sw/bin"))?;

        let packages = discover_system_packages()?;
        let enabled_services = discover_enabled_services()?;

        Ok(Self {
            current_generation,
            binaries,
            packages,
            enabled_services,
        })
    }

    /// system-wideな現在のsystem generationに指定したコマンドが存在するか確認する。
    pub fn has_command(&self, command: &str) -> bool {
        self.binaries.contains_key(command)
    }

    /// 指定したコマンドのsystem-wideな実体を取得する。
    pub fn command_path(&self, command: &str) -> Option<&Path> {
        self.binaries.get(command).map(PathBuf::as_path)
    }

    /// 現在のNixOS system generationに指定したpackageが含まれるか確認する。
    ///
    /// 実行ファイル名とpackage名が一致しない場合も、store path名から判定できる。
    pub fn has_package(&self, package: &str) -> bool {
        self.packages
            .iter()
            .any(|name| name == package || name.starts_with(&format!("{package}-")))
    }

    /// systemd上で指定したsystem serviceが有効になっているか確認する。
    pub fn is_service_enabled(&self, service: &str) -> bool {
        self.enabled_services.contains(service)
    }
}

/// /run/current-system/sw/bin の内容を取得する。
fn discover_binaries(bin_directory: &Path) -> Result<BTreeMap<String, PathBuf>, String> {
    let entries = std::fs::read_dir(bin_directory).map_err(|e| {
        format!(
            "failed to read system binary directory {}: {}",
            bin_directory.display(),
            e
        )
    })?;

    let mut binaries = BTreeMap::new();

    for entry in entries {
        let entry =
            entry.map_err(|e| format!("failed to read system binary directory entry: {}", e))?;

        let path = entry.path();

        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        if name.is_empty() {
            continue;
        }

        binaries.insert(name.into_owned(), path);
    }

    Ok(binaries)
}

/// 現在のsystem generationが直接参照しているNix store pathを取得する。
fn discover_system_packages() -> Result<BTreeSet<String>, String> {
    let output = Command::new("nix-store")
        .args(["--query", "--references", "/run/current-system/sw"])
        .output()
        .map_err(|e| format!("failed to execute nix-store: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "nix-store --query --references failed: {}",
            stderr.trim()
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let path = Path::new(line.trim());
            let name = path.file_name()?.to_string_lossy();
            let (_, name) = name.split_once('-')?;
            Some(name.to_string())
        })
        .collect())
}

/// systemdで現在enableされているsystem serviceを取得する。
fn discover_enabled_services() -> Result<BTreeSet<String>, String> {
    let output = Command::new("systemctl")
        .args([
            "list-unit-files",
            "--type=service",
            "--state=enabled",
            "--no-legend",
            "--no-pager",
        ])
        .output()
        .map_err(|e| format!("failed to execute systemctl: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        return Err(format!(
            "systemctl list-unit-files failed: {}",
            stderr.trim()
        ));
    }

    Ok(parse_enabled_services(&output.stdout))
}

/// systemctlの出力からservice名を抽出する。
///
/// 例:
/// sshd.service enabled enabled
/// → sshd
fn parse_enabled_services(output: &[u8]) -> BTreeSet<String> {
    let text = String::from_utf8_lossy(output);

    text.lines()
        .filter_map(|line| {
            let unit = line.split_whitespace().next()?;

            if !unit.ends_with(".service") {
                return None;
            }

            Some(unit.trim_end_matches(".service").to_string())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_enabled_services_extracts_service_names() {
        let output = br#"
sshd.service enabled enabled
NetworkManager.service enabled enabled
getty.service enabled enabled
"#;

        let services = parse_enabled_services(output);

        assert!(services.contains("sshd"));
        assert!(services.contains("NetworkManager"));
        assert!(services.contains("getty"));
        assert_eq!(services.len(), 3);
    }

    #[test]
    fn parse_enabled_services_ignores_non_service_units() {
        let output = br#"
default.target enabled enabled
sshd.service enabled enabled
"#;

        let services = parse_enabled_services(output);

        assert_eq!(services.len(), 1);
        assert!(services.contains("sshd"));
        assert!(!services.contains("default.target"));
    }

    #[test]
    fn system_state_queries_are_deterministic() {
        let mut binaries = BTreeMap::new();
        binaries.insert(
            "firefox".to_string(),
            PathBuf::from("/run/current-system/sw/bin/firefox"),
        );

        let mut services = BTreeSet::new();
        services.insert("sshd".to_string());

        let state = SystemState {
            current_generation: PathBuf::from("/nix/store/example-nixos-system"),
            binaries,
            packages: BTreeSet::from(["firefox-1.0".to_string()]),
            enabled_services: services,
        };

        assert!(state.has_command("firefox"));
        assert_eq!(
            state.command_path("firefox"),
            Some(Path::new("/run/current-system/sw/bin/firefox"))
        );
        assert!(!state.has_command("chromium"));
        assert!(state.has_package("firefox"));
        assert!(!state.has_package("chromium"));

        assert!(state.is_service_enabled("sshd"));
        assert!(!state.is_service_enabled("nginx"));
    }
}
