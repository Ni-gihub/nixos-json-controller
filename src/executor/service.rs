use crate::command::Action;
use crate::nixos;
use crate::nixos::write_strategy::{
    disable_service_strategy, enable_service_strategy, ServiceDisableStrategy, ServiceEnableStrategy,
};
use crate::planner::ExecutionPlan;

use super::error::ExecutorError;

pub fn execute(plan: ExecutionPlan) -> Result<bool, ExecutorError> {
    let context = nixos::flake::discovery_context().map_err(ExecutorError::NixosError)?;
    let config = nixos::config::ConfigState::discover(context.flake_root())
        .map_err(ExecutorError::NixosError)?;
    let system =
        nixos::system::SystemState::discover().map_err(ExecutorError::NixosError)?;

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
                    return Ok(false);
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
                    return install_service_in_dedicated_module(
                        context.flake_root(),
                        &config,
                        &plan.target.name,
                        "existing service declaration is outside the safe write boundary",
                    );
                }

                nixos::writer::enable_service(path, &plan.target.name)
                    .map_err(ExecutorError::NixosError)?;
                return Ok(true);
            }

            match enable_service_strategy(&config, &system, &plan.target.name) {
                ServiceEnableStrategy::ExistingFile { path } => {
                    nixos::writer::enable_service(&path, &plan.target.name)
                        .map_err(ExecutorError::NixosError)?;
                    Ok(true)
                }
                ServiceEnableStrategy::AlreadyDeclared { .. }
                | ServiceEnableStrategy::AlreadyEnabledInSystem => Ok(false),
                ServiceEnableStrategy::Ambiguous { candidates } => Err(
                    ExecutorError::NixosError(format!(
                        "multiple service configuration files found: {}",
                        format_paths(&candidates),
                    )),
                ),
                ServiceEnableStrategy::Unsupported => install_service_in_dedicated_module(
                    context.flake_root(),
                    &config,
                    &plan.target.name,
                    "no safe service configuration target was found",
                ),
            }
        }

        Action::DisableService => {
            if !provenance_paths.is_empty() {
                if provenance.contains_local_boolean(context.flake_root(), false) {
                    return Ok(false);
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

                nixos::writer::disable_service(path, &plan.target.name)
                    .map_err(ExecutorError::NixosError)?;
                return Ok(true);
            }

            match disable_service_strategy(&config, &system, &plan.target.name) {
                ServiceDisableStrategy::Declared { paths } => {
                    for path in paths {
                        nixos::writer::disable_service(&path, &plan.target.name)
                            .map_err(ExecutorError::NixosError)?;
                    }
                    Ok(true)
                }
                ServiceDisableStrategy::ExistingFile { path } => {
                    nixos::writer::disable_service(&path, &plan.target.name)
                        .map_err(ExecutorError::NixosError)?;
                    Ok(true)
                }
                ServiceDisableStrategy::NotDeclared => Ok(false),
                ServiceDisableStrategy::Ambiguous { candidates } => Err(
                    ExecutorError::NixosError(format!(
                        "multiple service configuration files found: {}",
                        format_paths(&candidates),
                    )),
                ),
            }
        }

        _ => Ok(false),
    }
}

fn install_service_in_dedicated_module(
    flake_root: &std::path::Path,
    config: &nixos::config::ConfigState,
    service: &str,
    reason: &str,
) -> Result<bool, ExecutorError> {
    println!("Falling back to the NXC dedicated service module: {}", reason);

    nixos::dedicated::enable_service(flake_root, config, service)
        .map_err(ExecutorError::NixosError)?;

    Ok(true)
}


fn format_paths(paths: &[std::path::PathBuf]) -> String {
    paths
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}
