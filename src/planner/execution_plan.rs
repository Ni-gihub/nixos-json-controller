use crate::command::Action;
use crate::resolver::ResolvedTarget;


#[derive(Debug)]
pub struct ExecutionPlan {

    pub action: Action,

    pub target: ResolvedTarget,

}