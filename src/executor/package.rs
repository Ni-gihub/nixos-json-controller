use std::path::{Path, PathBuf};

use crate::command::Action;
use crate::nixos;
use crate::nixos::application::ApplicationState;
use crate::nixos::dedicated::{DedicatedPackageChange, FileBackup};
use crate::nixos::write_strategy::{
    PackageInstallStrategy, PackageRemoveStrategy, install_package_strategy,
    remove_package_strategy,
};
use crate::planner::ExecutionPlan;

use super::error::ExecutorError;

#[derive(Debug, Clone, Default)]
pub struct PackageChange {
    pub backups: Vec<FileBackup>,
    pub created_files: Vec<PathBuf>,
    pub staged_files: Vec<nixos::dedicated::StagedFile>,
}

impl PackageChange {
    pub fn is_changed(&self) -> bool {
        !self.backups.is_empty() || !self.created_files.is_empty()
    }

    pub fn rollback(&self) -> Result<(), String> {
        nixos::dedicated::rollback_changes(&self.staged_files, &self.backups, &self.created_files)
    }
}

impl From<DedicatedPackageChange> for PackageChange {
    fn from(change: DedicatedPackageChange) -> Self {
        Self {
            backups: change.backups,
            created_files: change.created_files,
            staged_files: change.staged_files,
        }
    }
}

pub fn execute(plan: ExecutionPlan) -> Result<bool, ExecutorError> {
    Ok(execute_with_change(plan)?.is_changed())
}

pub fn execute_with_change(plan: ExecutionPlan) -> Result<PackageChange, ExecutorError> {
    let context = nixos::flake::discovery_context().map_err(ExecutorError::NixosError)?;
    let config = nixos::config::ConfigState::discover(context.flake_root())
        .map_err(ExecutorError::NixosError)?;
    let system = nixos::system::SystemState::discover().map_err(ExecutorError::NixosError)?;

    match plan.action {
        Action::InstallPackage => {
            let state = ApplicationState::inspect(&context, &plan.target.name, &system)
                .map_err(ExecutorError::NixosError)?;

            if state.system_present {
                println!("{} is already installed.", plan.target.name);
                return Ok(PackageChange::default());
            }

            if state.declared_in_config {
                println!(
                    "{} is already declared in the NixOS configuration but is not present in the active system.",
                    plan.target.name
                );
                return Ok(PackageChange::default());
            }

            match install_package_strategy(&config, &system, &plan.target.name) {
                PackageInstallStrategy::ExistingFile { path } => {
                    let backup = FileBackup::capture(context.flake_root(), &path).map_err(ExecutorError::NixosError)?;

                    if let Err(error) = nixos::writer::install_package(context.flake_root(), &path, &plan.target.name) {
                        return install_package_in_dedicated_module(
                            context.flake_root(),
                            &config,
                            &plan.target.name,
                            error,
                        );
                    }

                    let change = PackageChange {
                        backups: vec![backup],
                        created_files: Vec::new(),
                        staged_files: Vec::new(),
                    };

                    // Configuration evaluation is intentionally deferred until rebuild.
                    // Provenance matching of environment.systemPackages is not a reliable
                    // pre-rebuild installation check.
                    Ok(change)
                }
                PackageInstallStrategy::AlreadyDeclared { .. }
                | PackageInstallStrategy::AlreadyPresentInSystem => Ok(PackageChange::default()),
                PackageInstallStrategy::Ambiguous { .. } | PackageInstallStrategy::Unsupported => {
                    install_package_in_dedicated_module(
                        context.flake_root(),
                        &config,
                        &plan.target.name,
                        "existing NixOS configuration has no uniquely editable package target"
                            .to_string(),
                    )
                }
            }
        }

        Action::RemovePackage => {
            let paths = plan
                .details
                .as_ref()
                .map(|details| details.affected_files.clone())
                .unwrap_or_default();

            if !paths.is_empty() {
                let backups = nixos::dedicated::apply_file_changes(
                    context.flake_root(),
                    &paths,
                    |path| {
                        if config.is_safe_write_target(path) {
                            Ok(())
                        } else {
                            Err(format!(
                                "package '{}' is declared in a configuration file outside the safe write boundary: {}",
                                plan.target.name,
                                path.display(),
                            ))
                        }
                    },
                    |path| nixos::writer::remove_package(context.flake_root(), path, &plan.target.name),
                )
                .map_err(ExecutorError::NixosError)?;

                Ok(PackageChange {
                    backups,
                    ..PackageChange::default()
                })
            } else {
                match remove_package_strategy(&config, &system, &plan.target.name) {
                    PackageRemoveStrategy::Declared { paths } => {
                        let backups = nixos::dedicated::apply_file_changes(
                            &paths,
                            |path| {
                                if config.is_safe_write_target(path) {
                                    Ok(())
                                } else {
                                    Err(format!(
                                        "package '{}' is declared in a configuration file outside the safe write boundary: {}",
                                        plan.target.name,
                                        path.display(),
                                    ))
                                }
                            },
                            |path| nixos::writer::remove_package(context.flake_root(), path, &plan.target.name),
                        )
                        .map_err(ExecutorError::NixosError)?;

                        Ok(PackageChange {
                            backups,
                            ..PackageChange::default()
                        })
                    }
                    PackageRemoveStrategy::NotDeclared | PackageRemoveStrategy::SystemOnly => {
                        Ok(PackageChange::default())
                    }
                    PackageRemoveStrategy::Ambiguous { candidates } => {
                        Err(ExecutorError::NixosError(format!(
                            "multiple package configuration files found: {}",
                            format_paths(&candidates),
                        )))
                    }
                }
            }
        }

        _ => Ok(PackageChange::default()),
    }
}

fn install_package_in_dedicated_module(
    flake_root: &Path,
    config: &nixos::config::ConfigState,
    package: &str,
    reason: String,
) -> Result<PackageChange, ExecutorError> {
    println!(
        "Falling back to the NXC dedicated package module: {}",
        reason
    );

    let change = nixos::dedicated::install_package(flake_root, config, package)
        .map_err(ExecutorError::NixosError)?;

    let change: PackageChange = change.into();

    Ok(change)
}

fn format_paths(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}
