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

pub fn install_package(package: &str) -> Result<(), String> {
    install_package_with_password(package, None)
}

pub fn install_package_with_password(
    package: &str,
    password: Option<&str>,
) -> Result<(), String> {
    let command = Command {
        action: Action::InstallPackage,
        target: Target {
            raw: package.to_string(),
        },
    };

    Validator::validate(&command).map_err(|e| e.to_string())?;

    let target = Resolver::resolve(command.action.clone(), command.target)?;

    let plan = Planner::create(command.action, target);
    let plan =
        Planner::prepare(plan).map_err(|e| format!("failed to create execution plan: {e}"))?;

    Executor::execute_with_password(plan, password).map_err(|e| e.to_string())?;

    Ok(())
}
