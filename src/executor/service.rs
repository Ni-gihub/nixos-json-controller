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
}

impl ServiceChange {
    pub fn is_changed(&self) -> bool {
        !self.backups.is_empty() || !self.created_files.is_empty()
    }

    pub fn rollback(&self) -> Result<(), String> {
        for backup in self.backups.iter().rev() {
            backup.restore()?;
        }

        for path in self.created_files.iter().rev() {
            if path.is_file() {
                std::fs::remove_file(path)
                    .map_err(|e| format!("failed to remove {}: {}", path.display(), e))?;
            }
        }

        Ok(())
    }
}

impl From<nixos::dedicated::DedicatedServiceChange> for ServiceChange {
    fn from(change: nixos::dedicated::DedicatedServiceChange) -> Self {
        Self {
            backups: change.backups,
            created_files: change.created_files,
        }
    }
}

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
                if provenance.contains_local_boolean(context.flake_root(), true) {
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

                return write_service_file(path, &plan.target.name, true);
            }

            match enable_service_strategy(&config, &system, &plan.target.name) {
                ServiceEnableStrategy::ExistingFile { path } => {
                    write_service_file(&path, &plan.target.name, true)
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
                if provenance.contains_local_boolean(context.flake_root(), false) {
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

                return write_service_file(path, &plan.target.name, false);
            }

            match disable_service_strategy(&config, &system, &plan.target.name) {
                ServiceDisableStrategy::Declared { paths } => {
                    let mut change = ServiceChange::default();

                    for path in paths {
                        let backup = nixos::dedicated::FileBackup::capture(&path)
                            .map_err(ExecutorError::NixosError)?;

                        if let Err(error) = nixos::writer::disable_service(&path, &plan.target.name)
                        {
                            let _ = change.rollback();
                            return Err(ExecutorError::NixosError(error));
                        }

                        change.backups.push(backup);
                    }

                    Ok(change)
                }
                ServiceDisableStrategy::ExistingFile { path } => {
                    write_service_file(&path, &plan.target.name, false)
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

fn write_service_file(
    path: &std::path::Path,
    service: &str,
    enabled: bool,
) -> Result<ServiceChange, ExecutorError> {
    let backup = nixos::dedicated::FileBackup::capture(path).map_err(ExecutorError::NixosError)?;

    let result = if enabled {
        nixos::writer::enable_service(path, service)
    } else {
        nixos::writer::disable_service(path, service)
    };

    if let Err(error) = result {
        return Err(ExecutorError::NixosError(error));
    }

    Ok(ServiceChange {
        backups: vec![backup],
        created_files: Vec::new(),
    })
}

fn install_service_in_dedicated_module(
    flake_root: &std::path::Path,
    config: &nixos::config::ConfigState,
    service: &str,
) -> Result<ServiceChange, ExecutorError> {
    println!("Using the NXC dedicated service module for '{}'.", service);

    nixos::dedicated::enable_service(flake_root, config, service)
        .map(Into::into)
        .map_err(ExecutorError::NixosError)
}

fn format_paths(paths: &[std::path::PathBuf]) -> String {
    paths
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}
