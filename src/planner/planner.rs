use crate::command::Action;
use crate::resolver::ResolvedTarget;

use super::execution_plan::ExecutionPlan;


pub struct Planner;


impl Planner {

    pub fn create(
        action: Action,
        target: ResolvedTarget,
    ) -> ExecutionPlan {

        ExecutionPlan {

            action,

            target,

            dry_run: false,

        }

    }

}