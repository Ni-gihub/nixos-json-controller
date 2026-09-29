use crate::command::Action;
use crate::nixos;
use crate::nixos::write_strategy::{
    disable_service_strategy,
    enable_service_strategy,
    ServiceDisableStrategy,
    ServiceEnableStrategy,
};
use crate::planner::ExecutionPlan;

use super::error::ExecutorError;

pub fn execute(plan: ExecutionPlan) -> Result<bool, ExecutorError> {
    let context = nixos::discovery_context().map_err(ExecutorError::NixosError)?;
    let config = nixos::config::ConfigState::discover(context.flake_root())
        .map_err(ExecutorError::NixosError)?;
    let system = nixos::system::SystemState::discover()
        .map_err(ExecutorError::NixosError)?;

    match plan.action {
        Action::EnableService => match enable_service_strategy(&config, &system, &plan.target.name) {
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
            ServiceEnableStrategy::Unsupported => Err(
                ExecutorError::NixosError(
                    "no safe service configuration target was found".to_string(),
                ),
            ),
        },

        Action::DisableService => match disable_service_strategy(&config, &system, &plan.target.name) {
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
        },

        _ => Ok(false),
    }
}

fn format_paths(paths: &[std::path::PathBuf]) -> String {
    paths.iter().map(|path| path.display().to_string()).collect::<Vec<_>>().join(", ")
}
