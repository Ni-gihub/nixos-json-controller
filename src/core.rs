use crate::{
    command::{Action, Command, Target},
    executor::{Executor, ExecutorError},
    nixos::{
        discovery::{
            discover_candidates, inspect_candidates, save_discovery, select_candidate,
            DiscoveryReport, DiscoveryStatus, SearchOptions,
        },
        status::{explain_package, explain_service, PackageExplanation, ServiceExplanation},
        system::SystemState,
        discovery::DiscoveryContext,
    },
    planner::{ExecutionPlan, Planner},
    resolver::{ResolvedTarget, Resolver},
    validator::Validator,
};

pub struct NxcCore;

impl NxcCore {
    pub fn resolve(action: Action, target: impl Into<String>) -> Result<ResolvedTarget, String> {
        let command = Command {
            action,
            target: Target { raw: target.into() },
        };

        Validator::validate(&command).map_err(|e| format!("{e:?}"))?;
        Resolver::resolve(command.action, command.target)
    }

    pub fn plan(action: Action, target: impl Into<String>) -> Result<ExecutionPlan, String> {
        let resolved = Self::resolve(action.clone(), target)?;
        Planner::prepare(Planner::create(action, resolved))
    }

    pub fn execute(plan: ExecutionPlan) -> Result<(), ExecutorError> {
        Executor::execute(plan)
    }

    pub fn plan_and_execute(
        action: Action,
        target: impl Into<String>,
    ) -> Result<(), String> {
        let plan = Self::plan(action, target)?;
        Self::execute(plan).map_err(|e| format!("{e:?}"))
    }

    pub fn status() -> Result<SystemState, String> {
        SystemState::discover()
    }

    pub fn explain_package(target: impl Into<String>) -> Result<PackageExplanation, String> {
        let context = DiscoveryContext::load()?;
        let system = SystemState::discover()?;
        let resolved = Self::resolve(Action::InstallPackage, target)?;
        explain_package(&context, &system, &resolved.name)
    }

    pub fn explain_service(target: impl Into<String>) -> Result<ServiceExplanation, String> {
        let context = DiscoveryContext::load()?;
        let system = SystemState::discover()?;
        let resolved = Self::resolve(Action::EnableService, target)?;
        explain_service(&context, &system, &resolved.name)
    }

    pub fn discover() -> Result<DiscoveryReport, String> {
        let home = std::env::var_os("HOME")
            .ok_or_else(|| "HOME environment variable is not set".to_string())?;

        let options = SearchOptions {
            roots: vec![home.into()],
            max_depth: 4,
        };

        let mut candidates = discover_candidates(&options);
        inspect_candidates(&mut candidates);
        let report = select_candidate(&candidates);

        if matches!(report.status, DiscoveryStatus::Success) {
            if let Some(selected) = &report.selected {
                save_discovery(selected)?;
            } else {
                return Err("discovery succeeded without a selected result".to_string());
            }
        }

        Ok(report)
    }
}
