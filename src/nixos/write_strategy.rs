use std::path::{Path, PathBuf};

use super::config::ConfigState;
use super::system::SystemState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageWriteStrategy {
    Noop {
        declared_in: Vec<PathBuf>,
    },
    SystemOnly,

    ExistingFile {
        path: PathBuf,
    },
    Ambiguous {
        candidates: Vec<PathBuf>,
    },
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceWriteStrategy {
    Noop {
        declared_in: Vec<PathBuf>,
    },
    SystemOnly,

    ExistingFile {
        path: PathBuf,
    },
    Ambiguous {
        candidates: Vec<PathBuf>,
    },
    Unsupported,
}

pub fn package_strategy(
    config: &ConfigState,
    system: &SystemState,
    package: &str,
) -> PackageWriteStrategy {
    let declared_in = config
        .package_declarations(package)
        .into_iter()
        .map(Path::to_path_buf)
        .collect::<Vec<_>>();

    if !declared_in.is_empty() {
        return PackageWriteStrategy::Noop { declared_in };
    }

    if system.has_command(package) {
        return PackageWriteStrategy::SystemOnly;
    }

    let candidates = config
        .package_write_targets()
        .into_iter()
        .map(Path::to_path_buf)
        .collect::<Vec<_>>();

    match candidates.as_slice() {
        [path] => PackageWriteStrategy::ExistingFile {
            path: path.clone(),
        },
        [] => PackageWriteStrategy::Unsupported,
        _ => PackageWriteStrategy::Ambiguous {
            candidates,
        },
    }
}

pub fn service_strategy(
    config: &ConfigState,
    system: &SystemState,
    service: &str,
) -> ServiceWriteStrategy {
    let declared_in = config
        .service_declarations(service)
        .into_iter()
        .map(Path::to_path_buf)
        .collect::<Vec<_>>();

    if !declared_in.is_empty() {
        return ServiceWriteStrategy::Noop { declared_in };
    }

    if system.is_service_enabled(service) {
        return ServiceWriteStrategy::SystemOnly;
    }

    let candidates = config
        .service_write_targets()
        .into_iter()
        .map(Path::to_path_buf)
        .collect::<Vec<_>>();

    match candidates.as_slice() {
        [path] => ServiceWriteStrategy::ExistingFile {
            path: path.clone(),
        },
        [] => ServiceWriteStrategy::Unsupported,
        _ => ServiceWriteStrategy::Ambiguous {
            candidates,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn config() -> ConfigState {
        ConfigState {
            files: vec![
                super::super::config::ConfigFile {
                    path: PathBuf::from("packages.nix"),
                    has_system_packages: true,
                    has_systemd_services: false,
                    declared_packages: BTreeSet::new(),
                    declared_services: BTreeSet::new(),
                },
                super::super::config::ConfigFile {
                    path: PathBuf::from("services.nix"),
                    has_system_packages: false,
                    has_systemd_services: true,
                    declared_packages: BTreeSet::new(),
                    declared_services: BTreeSet::new(),
                },
            ],
        }
    }

    fn empty_system() -> SystemState {
        SystemState {
            current_generation: PathBuf::from("/nix/store/example"),
            binaries: Default::default(),
            enabled_services: Default::default(),
        }
    }

    #[test]
    fn chooses_the_only_package_file() {
        let result = package_strategy(
            &config(),
            &empty_system(),
            "firefox",
        );

        assert_eq!(
            result,
            PackageWriteStrategy::ExistingFile {
                path: PathBuf::from("packages.nix"),
            }
        );
    }

    #[test]
    fn does_not_write_when_package_is_already_declared() {
        let mut config = config();
        config.files[0]
            .declared_packages
            .insert("firefox".to_string());

        let result = package_strategy(
            &config,
            &empty_system(),
            "firefox",
        );

        assert!(matches!(
            result,
            PackageWriteStrategy::Noop { .. }
        ));
    }

    #[test]
    fn reports_system_only_when_package_is_present_but_undeclared() {
        let mut system = empty_system();
        system
            .binaries
            .insert(
                "firefox".to_string(),
                PathBuf::from("/run/current-system/sw/bin/firefox"),
            );

        let result = package_strategy(
            &config(),
            &system,
            "firefox",
        );

        assert_eq!(result, PackageWriteStrategy::SystemOnly);
    }

    #[test]
    fn refuses_ambiguous_package_target() {
        let mut config = config();
        config.files.push(
            super::super::config::ConfigFile {
                path: PathBuf::from("desktop.nix"),
                has_system_packages: true,
                has_systemd_services: false,
                declared_packages: BTreeSet::new(),
                declared_services: BTreeSet::new(),
            },
        );

        let result = package_strategy(
            &config,
            &empty_system(),
            "firefox",
        );

        assert!(matches!(
            result,
            PackageWriteStrategy::Ambiguous { .. }
        ));
    }
}
