use crate::command::Action;
use crate::planner::ExecutionPlan;

use super::{
    package,
    service,
};

use super::error::ExecutorError;



pub struct Executor;



impl Executor {


    pub fn execute(
        plan: ExecutionPlan
    ) -> Result<(), ExecutorError> {


        match plan.action {


            Action::InstallPackage => {

                package::install(
                    &plan.target.name
                )

            }



            Action::RemovePackage => {

                package::remove(
                    &plan.target.name
                )

            }



            Action::EnableService => {

                service::enable(
                    &plan.target.name
                )

            }



            Action::DisableService => {

                service::disable(
                    &plan.target.name
                )

            }

        }

    }

}