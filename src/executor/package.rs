use crate::command::Action;
use crate::nixos;
use crate::planner::ExecutionPlan;

use super::error::ExecutorError;



pub fn execute(
    plan: ExecutionPlan,
) -> Result<(), ExecutorError> {

    match plan.action {

        Action::InstallPackage => {

            let content =
                nixos::flake::read_pkgs()
                    .map_err(
                        ExecutorError::NixosError,
                    )?;

            let updated =
                nixos::module::add_package_to_content(
                    &content,
                    &plan.target.name,
                )
                .map_err(
                    ExecutorError::NixosError,
                )?;

            nixos::flake::write_pkgs(
                &updated,
            )
            .map_err(
                ExecutorError::NixosError,
            )?;

        }



        Action::RemovePackage => {

    let content =
        nixos::flake::read_pkgs()
            .map_err(
                ExecutorError::NixosError,
            )?;

    let content =
        nixos::module::remove_package_from_content(
            &content,
            &plan.target.name,
        )
        .map_err(
            ExecutorError::NixosError,
        )?;

    nixos::flake::write_pkgs(
        &content,
    )
    .map_err(
        ExecutorError::NixosError,
    )?;

}



        _ => {}

    }


    Ok(())
}