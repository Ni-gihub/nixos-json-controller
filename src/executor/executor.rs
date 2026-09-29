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

    pub fn execute_with_rebuild(
        plan: ExecutionPlan,
        rebuild: bool,
    ) -> Result<(), ExecutorError> {
        let package_change = match plan.action {
            Action::InstallPackage | Action::RemovePackage => {
                Some(super::package::execute_with_change(plan.clone())?)
            }
            Action::EnableService | Action::DisableService => {
                super::service::execute(plan)?
                ;
                None
            }
        };

        let changed = package_change
            .as_ref()
            .map(|change| change.is_changed())
            .unwrap_or(true);

        if rebuild && changed {
            if let Err(error) = nixos::rebuild::switch() {
                if let Some(change) = package_change {
                    if let Err(rollback_error) = change.rollback() {
                        return Err(ExecutorError::NixosError(format!(
                            "nixos-rebuild failed: {}; rollback also failed: {}",
                            error, rollback_error
                        )));
                    }
                }

                return Err(ExecutorError::NixosError(error));
            }
        }

        Ok(())
    }
}
