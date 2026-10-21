use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

use crate::dictionary::Dictionary;

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

/// Immutable active-system package outputs used by App Store status checks.
/// This deliberately avoids scanning binaries or querying systemd services.
#[derive(Debug, Clone)]
pub(crate) struct PackageSystemSnapshot {
    pub(crate) packages: BTreeSet<String>,
}

#[derive(Debug, Clone)]
struct CachedPackageSnapshot {
    generation: PathBuf,
    packages: BTreeSet<String>,
}

static PACKAGE_SNAPSHOT_CACHE: OnceLock<Mutex<Option<CachedPackageSnapshot>>> =
    OnceLock::new();

impl PackageSystemSnapshot {
    pub(crate) fn discover() -> Result<Self, String> {
        let started = Instant::now();
        let current_generation = resolve_current_generation()?;
        let cache = PACKAGE_SNAPSHOT_CACHE.get_or_init(|| Mutex::new(None));
        let (packages, cache_hit) = cached_packages_for_generation(
            cache,
            &current_generation,
            || {
                let query_started = Instant::now();
                let result = discover_system_packages();
                log_perf_duration("nix_store_references", query_started.elapsed());
                result
            },
        )?;

        if perf_debug_enabled() {
            eprintln!(
                "[nxc-perf] package_system_snapshot elapsed_ms={} cache_hit={}",
                started.elapsed().as_millis(),
                cache_hit
            );
        }

        Ok(Self { packages })
    }

    pub(crate) fn has_any_package_output(&self, output_names: &[String]) -> bool {
        output_names
            .iter()
            .any(|output_name| self.packages.contains(output_name))
    }
}

impl SystemState {
    /// 現在起動中のNixOS system generationから状態を取得する。
    ///
    /// 設定ファイルの再探索やNix評価は行わない。
    /// 現在のsystem generationとsystemdの状態だけを読む。
    pub fn discover() -> Result<Self, String> {
        let current_generation = resolve_current_generation()?;
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

    /// Check an already-resolved derivation output name against active store outputs.
    ///
    /// Unlike unresolved package names, derivation output names already contain
    /// the full name (usually including the version), so they must match exactly.
    pub fn has_package_output(&self, output_name: &str) -> bool {
        self.packages.iter().any(|name| name == output_name)
    }

    /// Check whether any selected output for a package is part of the active system.
    pub fn has_any_package_output(&self, output_names: &[String]) -> bool {
        output_names
            .iter()
            .any(|output_name| self.has_package_output(output_name))
    }

    /// Check a Nixpkgs package reference against active store outputs.
    ///
    /// Package attributes do not necessarily match store output names; always
    /// resolve the full derivation name through the selected flake and compare
    /// it exactly. Binary names are checked separately through `has_command`.
    pub fn has_package_reference(
        &self,
        flake_root: &Path,
        configuration_name: &str,
        package: &str,
    ) -> Result<bool, String> {
        let output_names = super::provenance::evaluate_package_output_names_for_package(
            flake_root,
            configuration_name,
            package,
        )?;
        Ok(self.has_any_package_output(&output_names))
    }

    /// systemd上で指定したsystem serviceが有効になっているか確認する。
    pub fn is_service_enabled(&self, service: &str) -> bool {
        if self.enabled_services.contains(service) {
            return true;
        }

        let Ok(dictionary) = Dictionary::load() else {
            return false;
        };

        dictionary.services().any(|(canonical, aliases)| {
            canonical == service
                && aliases
                    .iter()
                    .any(|alias| self.enabled_services.contains(alias))
        })
    }
}

fn resolve_current_generation() -> Result<PathBuf, String> {
    let current_system = Path::new("/run/current-system");

    if !current_system.exists() {
        return Err(
            "current NixOS system generation was not found at /run/current-system".to_string(),
        );
    }

    std::fs::canonicalize(current_system)
        .map_err(|e| format!("failed to resolve /run/current-system: {}", e))
}

fn cached_packages_for_generation<F>(
    cache: &Mutex<Option<CachedPackageSnapshot>>,
    generation: &Path,
    load_packages: F,
) -> Result<(BTreeSet<String>, bool), String>
where
    F: FnOnce() -> Result<BTreeSet<String>, String>,
{
    // Keep the lock while loading so simultaneous requests for the same new
    // generation do not start duplicate nix-store processes.
    let mut cached = cache
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(snapshot) = cached.as_ref() {
        if snapshot.generation.as_path() == generation {
            return Ok((snapshot.packages.clone(), true));
        }
    }

    let packages = load_packages()?;
    *cached = Some(CachedPackageSnapshot {
        generation: generation.to_path_buf(),
        packages: packages.clone(),
    });
    Ok((packages, false))
}

fn perf_debug_enabled() -> bool {
    matches!(std::env::var("NXC_PERF_DEBUG").as_deref(), Ok("1"))
}

fn log_perf_duration(stage: &str, elapsed: std::time::Duration) {
    if perf_debug_enabled() {
        eprintln!(
            "[nxc-perf] {stage} elapsed_ms={}",
            elapsed.as_millis()
        );
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
    fn package_snapshot_cache_reuses_same_generation_and_invalidates_on_change() {
        let cache = Mutex::new(None);
        let mut loads = 0;
        let packages = BTreeSet::from(["firefox-1.0".to_string()]);

        let (first, first_hit) = cached_packages_for_generation(
            &cache,
            Path::new("/nix/store/system-generation-a"),
            || {
                loads += 1;
                Ok(packages.clone())
            },
        )
        .unwrap();
        assert_eq!(first, packages);
        assert!(!first_hit);

        let (second, second_hit) = cached_packages_for_generation(
            &cache,
            Path::new("/nix/store/system-generation-a"),
            || {
                loads += 1;
                Ok(BTreeSet::new())
            },
        )
        .unwrap();
        assert_eq!(second, packages);
        assert!(second_hit);
        assert_eq!(loads, 1, "same generation should not query nix-store twice");

        let (third, third_hit) = cached_packages_for_generation(
            &cache,
            Path::new("/nix/store/system-generation-b"),
            || {
                loads += 1;
                Ok(BTreeSet::from(["chromium-2.0".to_string()]))
            },
        )
        .unwrap();
        assert_eq!(third, BTreeSet::from(["chromium-2.0".to_string()]));
        assert!(!third_hit);
        assert_eq!(loads, 2, "a new generation must refresh the snapshot");
    }

    #[test]
    fn failed_snapshot_refresh_preserves_previous_generation_cache() {
        let cached_packages = BTreeSet::from(["firefox-1.0".to_string()]);
        let cache = Mutex::new(Some(CachedPackageSnapshot {
            generation: PathBuf::from("/nix/store/system-generation-a"),
            packages: cached_packages.clone(),
        }));

        let result = cached_packages_for_generation(
            &cache,
            Path::new("/nix/store/system-generation-b"),
            || Err("nix-store failed".to_string()),
        );
        assert_eq!(result.unwrap_err(), "nix-store failed");

        let (loaded, cache_hit) = cached_packages_for_generation(
            &cache,
            Path::new("/nix/store/system-generation-a"),
            || panic!("valid cached snapshot should remain available"),
        )
        .unwrap();
        assert_eq!(loaded, cached_packages);
        assert!(cache_hit);
    }

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
    fn service_state_resolves_canonical_name_to_systemd_alias() {
        let state = SystemState {
            current_generation: PathBuf::from("/nix/store/example-nixos-system"),
            binaries: BTreeMap::new(),
            packages: BTreeSet::new(),
            enabled_services: BTreeSet::from(["sshd".to_string()]),
        };

        assert!(state.is_service_enabled("openssh"));
    }

    #[test]
    fn service_state_rejects_unrelated_enabled_service() {
        let state = SystemState {
            current_generation: PathBuf::from("/nix/store/example-nixos-system"),
            binaries: BTreeMap::new(),
            packages: BTreeSet::new(),
            enabled_services: BTreeSet::from(["nginx".to_string()]),
        };

        assert!(!state.is_service_enabled("openssh"));
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
        assert!(state.has_package_output("firefox-1.0"));
        assert!(!state.has_package_output("firefox"));
        assert!(!state.has_package_output("chromium"));

        assert!(state.is_service_enabled("sshd"));
        assert!(!state.is_service_enabled("nginx"));
    }

    #[test]
    fn different_versions_are_not_exact_package_output_matches() {
        let state = SystemState {
            current_generation: PathBuf::from("/nix/store/example-nixos-system"),
            binaries: BTreeMap::new(),
            packages: BTreeSet::from(["firefox-123.0".to_string()]),
            enabled_services: BTreeSet::new(),
        };

        assert!(state.has_package_output("firefox-123.0"));
        assert!(!state.has_package_output("firefox-124.0"));
    }

    #[test]
    fn resolved_package_output_requires_exact_match() {
        let state = SystemState {
            current_generation: PathBuf::from("/nix/store/example-nixos-system"),
            binaries: BTreeMap::new(),
            packages: BTreeSet::from(["foo-1.2-tools".to_string()]),
            enabled_services: BTreeSet::new(),
        };

        assert!(!state.has_package_output("foo-1.2"));
        assert!(state.has_package_output("foo-1.2-tools"));
    }

    #[test]
    fn similar_package_names_do_not_match_without_the_resolved_output_name() {
        let state = SystemState {
            current_generation: PathBuf::from("/nix/store/example-nixos-system"),
            binaries: BTreeMap::new(),
            packages: BTreeSet::from(["ripgrep-all-1.0".to_string(), "foo-bar-2.0".to_string()]),
            enabled_services: BTreeSet::new(),
        };

        assert!(!state.has_package_output("ripgrep"));
        assert!(!state.has_package_output("foo"));
        assert!(state.has_package_output("ripgrep-all-1.0"));
        assert!(state.has_package_output("foo-bar-2.0"));
    }

    #[test]
    fn resolved_package_output_matches_exact_entry_when_similar_names_exist() {
        let state = SystemState {
            current_generation: PathBuf::from("/nix/store/example-nixos-system"),
            binaries: BTreeMap::new(),
            packages: BTreeSet::from(["foo-1.2".to_string(), "foo-1.2-tools".to_string()]),
            enabled_services: BTreeSet::new(),
        };

        assert!(state.has_package_output("foo-1.2"));
        assert!(state.has_package_output("foo-1.2-tools"));
    }
}
