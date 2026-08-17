use crate::command::Action;
use crate::nixos;
use crate::planner::ExecutionPlan;

use super::error::ExecutorError;



pub struct Executor;



impl Executor {

    pub fn execute(
        plan: ExecutionPlan,
    ) -> Result<(), ExecutorError> {

        if plan.dry_run {

            println!(
                "========== Dry Run =========="
            );

            println!(
                "Action : {:?}",
                plan.action
            );

            println!(
                "Target : {}",
                plan.target.name
            );

            println!(
                "============================="
            );

            return Ok(());

        }


        Self::execute_with_rebuild(
            plan,
            true,
        )

    }



    pub fn execute_with_rebuild(
    plan: ExecutionPlan,
    rebuild: bool,
) -> Result<(), ExecutorError> {

    match plan.action {

        Action::InstallPackage
        | Action::RemovePackage => {

            super::package::execute(
                plan
            )?;

        }



        Action::EnableService
        | Action::DisableService => {

            super::service::execute(
                plan
            )?;

        }

    }



    if rebuild {

        nixos::rebuild::switch()
            .map_err(
                ExecutorError::NixosError,
            )?;

    }



    Ok(())
}

}