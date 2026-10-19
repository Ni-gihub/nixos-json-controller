use crate::command::Action;
use crate::nixos;
use crate::nixos::write_strategy::{
    ServiceDisableStrategy, ServiceEnableStrategy, disable_service_strategy,
    enable_service_strategy,
};
use crate::planner::ExecutionPlan;

use super::error::ExecutorError;

#[derive(Debug, Clone, Default)]
pub struct ServiceChange {
    pub backups: Vec<nixos::dedicated::FileBackup>,
    pub created_files: Vec<std::path::PathBuf>,
    pub staged_files: Vec<nixos::dedicated::StagedFile>,
    pub verification_option: Option<String>,
}

impl ServiceChange {
    pub fn is_changed(&self) -> bool {
        !self.backups.is_empty() || !self.created_files.is_empty()
    }

    pub fn rollback(&self) -> Result<(), String> {
        nixos::dedicated::rollback_changes(&self.staged_files, &self.backups, &self.created_files)
    }
}

impl From<nixos::dedicated::DedicatedServiceChange> for ServiceChange {
    fn from(change: nixos::dedicated::DedicatedServiceChange) -> Self {
        Self {
            backups: change.backups,
            created_files: change.created_files,
            staged_files: change.staged_files,
            verification_option: None,
        }
    }
}

/// Apply a planned service enable/disable change and capture rollback data.
pub fn execute(plan: ExecutionPlan) -> Result<ServiceChange, ExecutorError> {
    let context = nixos::flake::discovery_context().map_err(ExecutorError::NixosError)?;
    let config = nixos::config::ConfigState::discover(context.flake_root())
        .map_err(ExecutorError::NixosError)?;
    let system = nixos::system::SystemState::discover().map_err(ExecutorError::NixosError)?;

    let provenance = nixos::provenance::evaluate_service_provenance(
        context.flake_root(),
        context.configuration_name(),
        &plan.target.name,
    )
    .map_err(ExecutorError::NixosError)?;

    let provenance_paths = provenance.local_files(context.flake_root());

    match plan.action {
        Action::EnableService => {
            if !provenance_paths.is_empty() {
                let effective_enabled = nixos::provenance::evaluate_boolean_option(
                    context.flake_root(),
                    context.configuration_name(),
                    &provenance.option,
                )
                .map_err(ExecutorError::NixosError)?;

                if effective_enabled {
                    return Ok(ServiceChange::default());
                }

                if provenance_paths.len() != 1 {
                    return Err(ExecutorError::NixosError(format!(
                        "multiple local definitions found for service '{}': {}",
                        plan.target.name,
                        format_paths(&provenance_paths),
                    )));
                }

                let path = &provenance_paths[0];
                if !config.is_safe_write_target(path) {
                    return Err(ExecutorError::NixosError(format!(
                        "service '{}' is declared in a configuration file outside the safe write boundary: {}",
                        plan.target.name,
                        path.display(),
                    )));
                }

                return write_service_file(context.flake_root(), path, &plan.target.name, true);
            }

            match enable_service_strategy(&config, &system, &plan.target.name) {
                ServiceEnableStrategy::ExistingFile { path } => {
                    write_service_file(context.flake_root(), &path, &plan.target.name, true)
                }
                ServiceEnableStrategy::AlreadyDeclared { .. }
                | ServiceEnableStrategy::AlreadyEnabledInSystem => Ok(ServiceChange::default()),
                ServiceEnableStrategy::Ambiguous { candidates } => {
                    Err(ExecutorError::NixosError(format!(
                        "multiple service configuration files found: {}",
                        format_paths(&candidates),
                    )))
                }
                ServiceEnableStrategy::Unsupported => install_service_in_dedicated_module(
                    context.flake_root(),
                    &config,
                    &plan.target.name,
                ),
            }
        }

        Action::DisableService => {
            if !provenance_paths.is_empty() {
                let effective_enabled = nixos::provenance::evaluate_boolean_option(
                    context.flake_root(),
                    context.configuration_name(),
                    &provenance.option,
                )
                .map_err(ExecutorError::NixosError)?;

                if !effective_enabled {
                    return Ok(ServiceChange::default());
                }

                if provenance_paths.len() != 1 {
                    return Err(ExecutorError::NixosError(format!(
                        "multiple local definitions found for service '{}': {}",
                        plan.target.name,
                        format_paths(&provenance_paths),
                    )));
                }

                let path = &provenance_paths[0];
                if !config.is_safe_write_target(path) {
                    return Err(ExecutorError::NixosError(format!(
                        "service '{}' is declared in a configuration file outside the safe write boundary: {}",
                        plan.target.name,
                        path.display(),
                    )));
                }

                return write_service_file(context.flake_root(), path, &plan.target.name, false);
            }

            match disable_service_strategy(&config, &system, &plan.target.name) {
                ServiceDisableStrategy::Declared { paths } => {
                    let verification_option =
                        service_option_for_paths(context.flake_root(), &paths, &plan.target.name)?;
                    let backups = nixos::dedicated::apply_file_changes(
                        context.flake_root(),
                        &paths,
                        |path| {
                            if config.is_safe_write_target(path) {
                                Ok(())
                            } else {
                                Err(format!(
                                    "service '{}' is declared in a configuration file outside the safe write boundary: {}",
                                    plan.target.name,
                                    path.display(),
                                ))
                            }
                        },
                        |path| nixos::writer::disable_service(context.flake_root(), path, &plan.target.name),
                    )
                    .map_err(ExecutorError::NixosError)?;

                    Ok(ServiceChange {
                        backups,
                        verification_option: Some(verification_option),
                        ..ServiceChange::default()
                    })
                }
                ServiceDisableStrategy::ExistingFile { path } => {
                    write_service_file(context.flake_root(), &path, &plan.target.name, false)
                }
                ServiceDisableStrategy::NotDeclared => Ok(ServiceChange::default()),
                ServiceDisableStrategy::Ambiguous { candidates } => {
                    Err(ExecutorError::NixosError(format!(
                        "multiple service configuration files found: {}",
                        format_paths(&candidates),
                    )))
                }
            }
        }

        _ => Ok(ServiceChange::default()),
    }
}

/// Safely update one service declaration and retain its original file contents.
fn write_service_file(
    flake_root: &std::path::Path,
    path: &std::path::Path,
    service: &str,
    enabled: bool,
) -> Result<ServiceChange, ExecutorError> {
    let target =
        nixos::writer::resolve_write_target(flake_root, path).map_err(ExecutorError::NixosError)?;
    let content = std::fs::read_to_string(&target).map_err(|error| {
        ExecutorError::NixosError(format!("failed to read {}: {}", target.display(), error))
    })?;
    let verification_option = nixos::module::service_option_for_content(&content, service)
        .map_err(ExecutorError::NixosError)?;
    let backup = nixos::dedicated::FileBackup::capture(flake_root, path)
        .map_err(ExecutorError::NixosError)?;

    let result = if enabled {
        nixos::writer::enable_service(flake_root, path, service)
    } else {
        nixos::writer::disable_service(flake_root, path, service)
    };

    if let Err(error) = result {
        return Err(ExecutorError::NixosError(error));
    }

    Ok(ServiceChange {
        backups: vec![backup],
        created_files: Vec::new(),
        staged_files: Vec::new(),
        verification_option: Some(verification_option),
    })
}

/// Add a service declaration to NXC's dedicated module when no safe existing target exists.
fn install_service_in_dedicated_module(
    flake_root: &std::path::Path,
    config: &nixos::config::ConfigState,
    service: &str,
) -> Result<ServiceChange, ExecutorError> {
    println!("Using the NXC dedicated service module for '{}'.", service);

    let mut change: ServiceChange = nixos::dedicated::enable_service(flake_root, config, service)
        .map(Into::into)
        .map_err(ExecutorError::NixosError)?;
    let module_path = flake_root.join(nixos::dedicated::DEDICATED_SERVICE_MODULE);
    let content = std::fs::read_to_string(&module_path).map_err(|error| {
        ExecutorError::NixosError(format!(
            "failed to read {}: {}",
            module_path.display(),
            error
        ))
    })?;
    change.verification_option = Some(
        nixos::module::service_option_for_content(&content, service)
            .map_err(ExecutorError::NixosError)?,
    );
    Ok(change)
}

fn service_option_for_paths(
    flake_root: &std::path::Path,
    paths: &[std::path::PathBuf],
    service: &str,
) -> Result<String, ExecutorError> {
    let mut selected: Option<String> = None;

    for path in paths {
        let target = nixos::writer::resolve_write_target(flake_root, path)
            .map_err(ExecutorError::NixosError)?;
        let content = std::fs::read_to_string(&target).map_err(|error| {
            ExecutorError::NixosError(format!("failed to read {}: {}", target.display(), error))
        })?;
        let option = nixos::module::service_option_for_content(&content, service)
            .map_err(ExecutorError::NixosError)?;

        if let Some(previous) = &selected {
            if previous != &option {
                return Err(ExecutorError::NixosError(format!(
                    "service '{}' is declared through multiple option paths ('{}' and '{}'); refusing to verify an arbitrary option",
                    service, previous, option
                )));
            }
        } else {
            selected = Some(option);
        }
    }

    Ok(selected.unwrap_or_else(|| format!("services.{service}.enable")))
}

fn format_paths(paths: &[std::path::PathBuf]) -> String {
    paths
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}
