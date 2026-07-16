use crate::command::Action;
use crate::resolver::ResolvedTarget;


#[derive(Debug, Clone)]
pub struct ExecutionPlan {

    pub action: Action,

    pub target: ResolvedTarget,

    pub dry_run: bool,

}