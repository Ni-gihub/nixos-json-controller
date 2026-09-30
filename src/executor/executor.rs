use crate::command::Action;
use crate::nixos;
use crate::planner::ExecutionPlan;

use super::error::ExecutorError;

pub struct Executor;

impl Executor {
    pub fn execute(plan: ExecutionPlan) -> Result<(), ExecutorError> {
        if plan.dry_run {
            println!("========== Dry Run ==========");
            println!("Action : {:?}", plan.action);
            println!("Target : {}", plan.target.name);
            println!("=============================");
            return Ok(());
        }

        Self::execute_with_rebuild(plan, true)
    }

    pub fn execute_with_rebuild(plan: ExecutionPlan, rebuild: bool) -> Result<(), ExecutorError> {
        let package_change = match plan.action {
            Action::InstallPackage | Action::RemovePackage => {
                Some(super::package::execute_with_change(plan.clone())?)
            }
            Action::EnableService | Action::DisableService => None,
        };

        let service_change = match plan.action {
            Action::EnableService | Action::DisableService => {
                Some(super::service::execute(plan.clone())?)
            }
            Action::InstallPackage | Action::RemovePackage => None,
        };

        let changed = package_change
            .as_ref()
            .map(|change| change.is_changed())
            .or_else(|| service_change.as_ref().map(|change| change.is_changed()))
            .unwrap_or(false);

        if should_rebuild(&plan, changed, rebuild) {
            if let Err(error) = nixos::rebuild::switch() {
                return rollback_after_rebuild_failure(
                    error,
                    package_change.as_ref(),
                    service_change.as_ref(),
                );
            }

            if let Action::InstallPackage | Action::RemovePackage = plan.action {
                if let Err(error) = verify_package_state(&plan.action, &plan.target.name) {
                    return rollback_after_verification_failure(
                        error,
                        package_change.as_ref(),
                        service_change.as_ref(),
                    );
                }
            }
        }

        Ok(())
    }
}

fn rollback_after_rebuild_failure(
    error: String,
    package_change: Option<&super::package::PackageChange>,
    service_change: Option<&super::service::ServiceChange>,
) -> Result<(), ExecutorError> {
    let rollback_result = rollback_change(package_change, service_change);
    if let Err(rollback_error) = rollback_result {
        return Err(ExecutorError::NixosError(format!(
            "nixos-rebuild failed: {}; rollback also failed: {}",
            error, rollback_error
        )));
    }

    Err(ExecutorError::NixosError(error))
}

fn rollback_after_verification_failure(
    error: String,
    package_change: Option<&super::package::PackageChange>,
    service_change: Option<&super::service::ServiceChange>,
) -> Result<(), ExecutorError> {
    let rollback_result = rollback_change(package_change, service_change);
    if let Err(rollback_error) = rollback_result {
        return Err(ExecutorError::NixosError(format!(
            "{}; rollback also failed: {}",
            error, rollback_error
        )));
    }

    // The active system was already switched, so rebuild once more from the
    // restored configuration to return the machine to the previous state.
    if let Err(rebuild_error) = nixos::rebuild::switch() {
        return Err(ExecutorError::NixosError(format!(
            "{}; rollback succeeded but restoring the previous system failed: {}",
            error, rebuild_error
        )));
    }

    Err(ExecutorError::NixosError(error))
}

fn rollback_change(
    package_change: Option<&super::package::PackageChange>,
    service_change: Option<&super::service::ServiceChange>,
) -> Result<(), String> {
    if let Some(change) = package_change {
        change.rollback()
    } else if let Some(change) = service_change {
        change.rollback()
    } else {
        Ok(())
    }
}

fn verify_package_state(action: &Action, package: &str) -> Result<(), String> {
    let system = nixos::system::SystemState::discover()?;
    let present = system.has_package(package) || system.has_command(package);

    match action {
        Action::InstallPackage if present => Ok(()),
        Action::RemovePackage if !present => Ok(()),
        Action::InstallPackage => Err(format!(
            "nixos-rebuild completed but package '{}' was not found in the active system",
            package
        )),
        Action::RemovePackage => Err(format!(
            "nixos-rebuild completed but package '{}' is still present in the active system",
            package
        )),
        _ => Ok(()),
    }
}

fn should_rebuild(plan: &ExecutionPlan, changed: bool, rebuild: bool) -> bool {
    if !rebuild {
        return false;
    }

    changed
        || plan
            .details
            .as_ref()
            .is_some_and(|details| details.rebuild_required)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::PlanDetails;
    use std::path::PathBuf;

    fn plan_with_rebuild_required(rebuild_required: bool) -> ExecutionPlan {
        ExecutionPlan {
            action: Action::InstallPackage,
            target: crate::resolver::ResolvedTarget {
                name: "firefox".to_string(),
            },
            dry_run: false,
            details: Some(PlanDetails {
                strategy: "test".to_string(),
                affected_files: vec![PathBuf::from("packages.nix")],
                change_required: false,
                rebuild_required,
                reason: "test".to_string(),
            }),
        }
    }

    #[test]
    fn rebuilds_when_plan_requires_it_without_file_changes() {
        let plan = plan_with_rebuild_required(true);

        assert!(should_rebuild(&plan, false, true));
    }

    #[test]
    fn does_not_rebuild_when_plan_does_not_require_it_without_changes() {
        let plan = plan_with_rebuild_required(false);

        assert!(!should_rebuild(&plan, false, true));
    }

    #[test]
    fn does_not_rebuild_when_rebuild_is_disabled() {
        let plan = plan_with_rebuild_required(true);

        assert!(!should_rebuild(&plan, false, false));
    }
}
