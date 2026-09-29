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

        let plan_requires_rebuild = plan
            .details
            .as_ref()
            .is_some_and(|details| details.rebuild_required);

        if rebuild
            && (changed || plan_requires_rebuild)
            && let Err(error) = nixos::rebuild::switch()
        {
            let rollback_result = if let Some(change) = package_change {
                change.rollback()
            } else if let Some(change) = service_change {
                change.rollback()
            } else {
                Ok(())
            };

            if let Err(rollback_error) = rollback_result {
                return Err(ExecutorError::NixosError(format!(
                    "nixos-rebuild failed: {}; rollback also failed: {}",
                    error, rollback_error
                )));
            }

            return Err(ExecutorError::NixosError(error));
        }

        Ok(())
    }
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
    fn rebuild_is_required_when_plan_requires_it_without_file_changes() {
        let plan = plan_with_rebuild_required(true);

        assert!(plan
            .details
            .as_ref()
            .is_some_and(|details| details.rebuild_required));
    }

    #[test]
    fn rebuild_is_not_required_when_plan_does_not_require_it() {
        let plan = plan_with_rebuild_required(false);

        assert!(!plan
            .details
            .as_ref()
            .is_some_and(|details| details.rebuild_required));
    }
}
