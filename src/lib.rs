pub mod command;
pub mod core;

#[cfg(test)]
mod core_tests;
pub mod dictionary;
pub mod executor;
pub mod nixos;
pub mod planner;
pub mod resolver;
pub mod validator;

use command::{Action, Command, Target};
use executor::Executor;
use planner::Planner;
use resolver::Resolver;
use validator::Validator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallOutcome {
    pub package: String,
    /// True only when a rebuild and active-system verification completed.
    pub active_state_verified: bool,
}

pub fn install_package(package: &str) -> Result<(), String> {
    install_package_with_password(package, None)
}

pub fn install_package_with_password(package: &str, password: Option<&str>) -> Result<(), String> {
    install_package_with_password_outcome(package, password).map(|_| ())
}

/// Install a package and report whether the executor verified its active state.
///
/// A no-op/already-declared path may succeed without a rebuild, in which case
/// callers must not infer that the package is active from success alone.
pub fn install_package_with_password_outcome(
    package: &str,
    password: Option<&str>,
) -> Result<InstallOutcome, String> {
    let command = Command {
        action: Action::InstallPackage,
        target: Target {
            raw: package.to_string(),
        },
    };

    Validator::validate(&command).map_err(|e| e.to_string())?;

    let target = Resolver::resolve(command.action.clone(), command.target)?;
    let package_name = target.name.clone();

    let plan = Planner::create(command.action, target);
    let plan =
        Planner::prepare(plan).map_err(|e| format!("failed to create execution plan: {e}"))?;

    let outcome =
        Executor::execute_with_password_outcome(plan, password).map_err(|e| e.to_string())?;

    Ok(InstallOutcome {
        package: package_name,
        active_state_verified: outcome.state_verified,
    })
}
