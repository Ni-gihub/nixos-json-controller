use crate::command::Action;
use crate::planner::ExecutionPlan;

use crate::nixos;

use super::error::ExecutorError;

pub struct Executor;

impl Executor {

    pub fn execute(
        plan: ExecutionPlan,
    ) -> Result<(), ExecutorError> {

        Self::execute_with_rebuild(
            plan,
            true,
        )

    }

    pub fn execute_with_rebuild(
        plan: ExecutionPlan,
        rebuild: bool,
    ) -> Result<(), ExecutorError> {

        if plan.dry_run {

            println!("========== Dry Run ==========");
            println!("Action : {:?}", plan.action);
            println!("Target : {}", plan.target.name);
            println!("=============================");

            return Ok(());

        }

        let content =
            match plan.action {

                Action::InstallPackage => {

                    nixos::module::add_package(
                        &plan.target.name,
                    )
                    .map_err(
                        ExecutorError::NixosError,
                    )?

                }

                Action::RemovePackage => {

                    nixos::module::remove_package(
                        &plan.target.name,
                    )
                    .map_err(
                        ExecutorError::NixosError,
                    )?

                }

                Action::EnableService => {

                    nixos::module::enable_service(
                        &plan.target.name,
                    )
                    .map_err(
                        ExecutorError::NixosError,
                    )?

                }

                Action::DisableService => {

                    nixos::module::disable_service(
                        &plan.target.name,
                    )
                    .map_err(
                        ExecutorError::NixosError,
                    )?

                }

            };

        nixos::flake::ensure_modules()
            .map_err(
                ExecutorError::NixosError,
            )?;

        nixos::flake::ensure_flake()
            .map_err(
                ExecutorError::NixosError,
            )?;

        nixos::flake::write_module(
            &content,
        )
        .map_err(
            ExecutorError::NixosError,
        )?;

        nixos::import::ensure_generated_module()
            .map_err(
                ExecutorError::NixosError,
            )?;

        if rebuild {

            nixos::rebuild::switch()
                .map_err(
                    ExecutorError::NixosError,
                )?;

        }

        Ok(())

    }

}