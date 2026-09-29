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

        if rebuild
            && changed
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
