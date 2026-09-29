use std::path::PathBuf;

use crate::command::Action;
use crate::resolver::ResolvedTarget;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanDetails {
    pub strategy: String,
    pub affected_files: Vec<PathBuf>,
    pub change_required: bool,
    pub rebuild_required: bool,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    pub action: Action,
    pub target: ResolvedTarget,
    pub dry_run: bool,
    pub details: Option<PlanDetails>,
}
