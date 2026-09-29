use std::path::{Path, PathBuf};

use super::config::ConfigState;
use super::system::SystemState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageInstallStrategy {
    AlreadyDeclared { paths: Vec<PathBuf> },
    AlreadyPresentInSystem,
    ExistingFile { path: PathBuf },
    Ambiguous { candidates: Vec<PathBuf> },
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageRemoveStrategy {
    Declared { paths: Vec<PathBuf> },
    NotDeclared,
    SystemOnly,
    Ambiguous { candidates: Vec<PathBuf> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceEnableStrategy {
    AlreadyDeclared { paths: Vec<PathBuf> },
    AlreadyEnabledInSystem,
    ExistingFile { path: PathBuf },
    Ambiguous { candidates: Vec<PathBuf> },
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceDisableStrategy {
    Declared { paths: Vec<PathBuf> },
    ExistingFile { path: PathBuf },
    NotDeclared,
    Ambiguous { candidates: Vec<PathBuf> },
}

pub fn install_package_strategy(
    config: &ConfigState,
    system: &SystemState,
    package: &str,
) -> PackageInstallStrategy {
    let declared = paths(config.package_declarations(package));

    if !declared.is_empty() {
        return PackageInstallStrategy::AlreadyDeclared { paths: declared };
    }

    if system.has_command(package) {
        return PackageInstallStrategy::AlreadyPresentInSystem;
    }

    match paths(config.package_write_targets()).as_slice() {
        [path] => PackageInstallStrategy::ExistingFile { path: path.clone() },
        [] => PackageInstallStrategy::Unsupported,
        candidates => PackageInstallStrategy::Ambiguous {
            candidates: candidates.to_vec(),
        },
    }
}

pub fn remove_package_strategy(
    config: &ConfigState,
    system: &SystemState,
    package: &str,
) -> PackageRemoveStrategy {
    let declared = paths(config.package_declarations(package));

    if !declared.is_empty() {
        return PackageRemoveStrategy::Declared { paths: declared };
    }

    if system.has_command(package) {
        return PackageRemoveStrategy::SystemOnly;
    }

    PackageRemoveStrategy::NotDeclared
}

pub fn enable_service_strategy(
    config: &ConfigState,
    system: &SystemState,
    service: &str,
) -> ServiceEnableStrategy {
    let declared = paths(config.service_declarations(service));

    if !declared.is_empty() {
        return ServiceEnableStrategy::AlreadyDeclared { paths: declared };
    }

    if system.is_service_enabled(service) {
        return ServiceEnableStrategy::AlreadyEnabledInSystem;
    }

    match paths(config.service_write_targets()).as_slice() {
        [path] => ServiceEnableStrategy::ExistingFile { path: path.clone() },
        [] => ServiceEnableStrategy::Unsupported,
        candidates => ServiceEnableStrategy::Ambiguous {
            candidates: candidates.to_vec(),
        },
    }
}

pub fn disable_service_strategy(
    config: &ConfigState,
    system: &SystemState,
    service: &str,
) -> ServiceDisableStrategy {
    let declared = paths(config.service_declarations(service));

    if !declared.is_empty() {
        return ServiceDisableStrategy::Declared { paths: declared };
    }

    let candidates = paths(config.service_write_targets());

    if system.is_service_enabled(service) {
        return match candidates.as_slice() {
            [path] => ServiceDisableStrategy::ExistingFile { path: path.clone() },
            [] => ServiceDisableStrategy::NotDeclared,
            candidates => ServiceDisableStrategy::Ambiguous {
                candidates: candidates.to_vec(),
            },
        };
    }

    ServiceDisableStrategy::NotDeclared
}

fn paths<'a>(paths: impl IntoIterator<Item = &'a Path>) -> Vec<PathBuf> {
    paths.into_iter().map(Path::to_path_buf).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nixos::config::ConfigFile;
    use std::collections::BTreeSet;

    fn config() -> ConfigState {
        ConfigState {
            files: vec![
                ConfigFile {
                    path: PathBuf::from("packages.nix"),
                    has_system_packages: true,
                    has_systemd_services: false,
                    has_service_options: false,
                    declared_packages: BTreeSet::new(),
                    declared_services: BTreeSet::new(),
                    imports: Vec::new(),
                    write_safety: crate::nixos::config::WriteSafety::Safe,
                },
                ConfigFile {
                    path: PathBuf::from("services.nix"),
                    has_system_packages: false,
                    has_systemd_services: true,
                    has_service_options: true,
                    declared_packages: BTreeSet::new(),
                    declared_services: BTreeSet::new(),
                    imports: Vec::new(),
                    write_safety: crate::nixos::config::WriteSafety::Safe,
                },
            ],
        }
    }

    fn system() -> SystemState {
        SystemState {
            current_generation: PathBuf::from("/nix/store/example"),
            binaries: Default::default(),
            packages: Default::default(),
            enabled_services: Default::default(),
        }
    }

    #[test]
    fn install_chooses_the_only_package_file() {
        assert_eq!(
            install_package_strategy(&config(), &system(), "firefox"),
            PackageInstallStrategy::ExistingFile {
                path: PathBuf::from("packages.nix"),
            }
        );
    }

    #[test]
    fn install_reports_declared_package_without_writing() {
        let mut config = config();
        config.files[0]
            .declared_packages
            .insert("firefox".to_string());

        assert!(matches!(
            install_package_strategy(&config, &system(), "firefox"),
            PackageInstallStrategy::AlreadyDeclared { .. }
        ));
    }

    #[test]
    fn install_distinguishes_system_only_package() {
        let mut system = system();
        system.binaries.insert(
            "firefox".to_string(),
            PathBuf::from("/run/current-system/sw/bin/firefox"),
        );

        assert_eq!(
            install_package_strategy(&config(), &system, "firefox"),
            PackageInstallStrategy::AlreadyPresentInSystem
        );
    }

    #[test]
    fn remove_uses_declared_package_locations() {
        let mut config = config();
        config.files[0]
            .declared_packages
            .insert("firefox".to_string());

        assert_eq!(
            remove_package_strategy(&config, &system(), "firefox"),
            PackageRemoveStrategy::Declared {
                paths: vec![PathBuf::from("packages.nix")],
            }
        );
    }

    #[test]
    fn enable_chooses_the_only_service_file() {
        assert_eq!(
            enable_service_strategy(&config(), &system(), "sshd"),
            ServiceEnableStrategy::ExistingFile {
                path: PathBuf::from("services.nix"),
            }
        );
    }

    #[test]
    fn disable_creates_declaration_for_system_only_service() {
        let mut system = system();
        system.enabled_services.insert("sshd".to_string());

        assert_eq!(
            disable_service_strategy(&config(), &system, "sshd"),
            ServiceDisableStrategy::ExistingFile {
                path: PathBuf::from("services.nix"),
            }
        );
    }
}
