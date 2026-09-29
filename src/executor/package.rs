use crate::command::Action;
use crate::nixos;
use crate::nixos::write_strategy::{
    install_package_strategy,
    remove_package_strategy,
    PackageInstallStrategy,
    PackageRemoveStrategy,
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
        Action::InstallPackage => match install_package_strategy(&config, &system, &plan.target.name) {
            PackageInstallStrategy::ExistingFile { path } => {
                nixos::writer::install_package(&path, &plan.target.name)
                    .map_err(ExecutorError::NixosError)?;
                Ok(true)
            }
            PackageInstallStrategy::AlreadyDeclared { .. }
            | PackageInstallStrategy::AlreadyPresentInSystem => Ok(false),
            PackageInstallStrategy::Ambiguous { candidates } => Err(
                ExecutorError::NixosError(format!(
                    "multiple package configuration files found: {}",
                    format_paths(&candidates),
                )),
            ),
            PackageInstallStrategy::Unsupported => Err(
                ExecutorError::NixosError(
                    "no safe package configuration target was found".to_string(),
                ),
            ),
        },

        Action::RemovePackage => match remove_package_strategy(&config, &system, &plan.target.name) {
            PackageRemoveStrategy::Declared { paths } => {
                for path in paths {
                    nixos::writer::remove_package(&path, &plan.target.name)
                        .map_err(ExecutorError::NixosError)?;
                }
                Ok(true)
            }
            PackageRemoveStrategy::NotDeclared | PackageRemoveStrategy::SystemOnly => Ok(false),
            PackageRemoveStrategy::Ambiguous { candidates } => Err(
                ExecutorError::NixosError(format!(
                    "multiple package configuration files found: {}",
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
