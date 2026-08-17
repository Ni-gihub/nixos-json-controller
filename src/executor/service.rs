use crate::command::Action;
use crate::nixos;
use crate::planner::ExecutionPlan;

use super::error::ExecutorError;



pub fn execute(
    plan: ExecutionPlan,
) -> Result<(), ExecutorError> {

    match plan.action {

        Action::EnableService => {

            let content =
                nixos::flake::read_core()
                    .map_err(
                        ExecutorError::NixosError,
                    )?;

            let updated =
                nixos::module::add_service_to_content(
                    &content,
                    &plan.target.name,
                    true,
                )
                .map_err(
                    ExecutorError::NixosError,
                )?;

            nixos::flake::write_core(
                &updated,
            )
            .map_err(
                ExecutorError::NixosError,
            )?;

        }



        Action::DisableService => {

            let content =
                nixos::flake::read_core()
                    .map_err(
                        ExecutorError::NixosError,
                    )?;

            let updated =
                nixos::module::add_service_to_content(
                    &content,
                    &plan.target.name,
                    false,
                )
                .map_err(
                    ExecutorError::NixosError,
                )?;

            nixos::flake::write_core(
                &updated,
            )
            .map_err(
                ExecutorError::NixosError,
            )?;

        }



        _ => {}

    }


    Ok(())
}