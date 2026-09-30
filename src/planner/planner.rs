use crate::command::Action;
use crate::nixos::{
    application::ApplicationState,
    config::ConfigState,
    discovery::DiscoveryContext,
    provenance::evaluate_service_provenance,
    system::SystemState,
    write_strategy::{
        PackageInstallStrategy, ServiceDisableStrategy, ServiceEnableStrategy,
        disable_service_strategy, enable_service_strategy, install_package_strategy,
    },
};
use crate::resolver::ResolvedTarget;

use super::execution_plan::{ExecutionPlan, PlanDetails};

pub struct Planner;

impl Planner {
    pub fn create(action: Action, target: ResolvedTarget) -> ExecutionPlan {
        ExecutionPlan {
            action,
            target,
            dry_run: false,
            details: None,
        }
    }

    pub fn prepare(mut plan: ExecutionPlan) -> Result<ExecutionPlan, String> {
        let context = DiscoveryContext::load()?;
        let config = ConfigState::discover(context.flake_root())?;
        let system = SystemState::discover()?;

        plan.details = Some(match plan.action {
            Action::InstallPackage => {
                Self::plan_install_package(&context, &config, &system, &plan.target.name)?
            }
            Action::RemovePackage => {
                Self::plan_remove_package(&context, &system, &plan.target.name)?
            }
            Action::EnableService => {
                Self::plan_enable_service(&context, &config, &system, &plan.target.name)?
            }
            Action::DisableService => {
                Self::plan_disable_service(&context, &config, &system, &plan.target.name)?
            }
        });

        Ok(plan)
    }

    fn plan_install_package(
        context: &DiscoveryContext,
        config: &ConfigState,
        system: &SystemState,
        package: &str,
    ) -> Result<PlanDetails, String> {
        let state = ApplicationState::inspect(context, package, system)?;

        if state.system_present {
            return Ok(PlanDetails {
                strategy: "already-present".to_string(),
                affected_files: Vec::new(),
                change_required: false,
                rebuild_required: false,
                reason: "package is already present in the active system".to_string(),
            });
        }

        if state.declared_in_config {
            return Ok(PlanDetails {
                strategy: "declared-not-active".to_string(),
                affected_files: state.declaration_locations,
                change_required: false,
                rebuild_required: true,
                reason:
                    "package is declared in the NixOS configuration but is not present in the active system"
                        .to_string(),
            });
        }

        match install_package_strategy(config, system, package) {
            PackageInstallStrategy::ExistingFile { path } => Ok(Self::change(
                "existing-file",
                vec![path],
                "add package declaration to the existing safe configuration file",
            )),
            PackageInstallStrategy::AlreadyDeclared { paths } => Ok(Self::no_change(
                "already-declared",
                paths,
                "package is already declared in the configuration",
            )),
            PackageInstallStrategy::AlreadyPresentInSystem => Ok(Self::no_change(
                "already-present",
                Vec::new(),
                "package is already present in the active system",
            )),
            PackageInstallStrategy::Ambiguous { candidates } => Err(format!(
                "multiple package write targets found: {}",
                format_paths(&candidates)
            )),
            PackageInstallStrategy::Unsupported => Ok(Self::change(
                "dedicated-file",
                vec![context.flake_root().join("nxc/packages.nix")],
                "create or reuse the NXC dedicated package module",
            )),
        }
    }

    fn plan_remove_package(
        context: &DiscoveryContext,
        system: &SystemState,
        package: &str,
    ) -> Result<PlanDetails, String> {
        let provenance = crate::nixos::provenance::evaluate_package_provenance(
            context.flake_root(),
            context.configuration_name(),
            package,
        )?;
        let paths = provenance.local_files(context.flake_root());

        if !paths.is_empty() {
            return Ok(Self::change(
                "evaluated-provenance",
                paths,
                "remove the package from its evaluated local declaration",
            ));
        }

        if system.has_command(package) || system.has_package(package) {
            return Ok(PlanDetails {
                strategy: "system-only".to_string(),
                affected_files: Vec::new(),
                change_required: false,
                rebuild_required: true,
                reason: "package is present in the active system but has no local evaluated declaration; rebuild is required to remove it from the active system".to_string(),
            });
        }

        Ok(Self::no_change(
            "not-declared",
            Vec::new(),
            "package is neither present in the active system nor declared locally",
        ))
    }

    fn plan_enable_service(
        context: &DiscoveryContext,
        config: &ConfigState,
        system: &SystemState,
        service: &str,
    ) -> Result<PlanDetails, String> {
        let provenance = evaluate_service_provenance(
            context.flake_root(),
            context.configuration_name(),
            service,
        )?;
        let paths = provenance.local_files(context.flake_root());

        if !paths.is_empty() {
            if provenance.contains_local_boolean(context.flake_root(), true) {
                return Ok(Self::no_change(
                    "already-enabled",
                    paths,
                    "service is already enabled in the evaluated configuration",
                ));
            }

            return Ok(Self::change(
                "evaluated-provenance",
                paths,
                "update the existing evaluated service declaration to enabled",
            ));
        }

        match enable_service_strategy(config, system, service) {
            ServiceEnableStrategy::ExistingFile { path } => Ok(Self::change(
                "existing-file",
                vec![path],
                "add or update the service declaration in the existing safe configuration file",
            )),
            ServiceEnableStrategy::AlreadyDeclared { paths } => Ok(Self::no_change(
                "already-declared",
                paths,
                "service is already declared in the configuration",
            )),
            ServiceEnableStrategy::AlreadyEnabledInSystem => Ok(Self::no_change(
                "already-enabled",
                Vec::new(),
                "service is already enabled in the active system",
            )),
            ServiceEnableStrategy::Ambiguous { candidates } => Err(format!(
                "multiple service write targets found: {}",
                format_paths(&candidates)
            )),
            ServiceEnableStrategy::Unsupported => Ok(Self::change(
                "dedicated-file",
                vec![context.flake_root().join("nxc/services.nix")],
                "create or reuse the NXC dedicated service module",
            )),
        }
    }

    fn plan_disable_service(
        context: &DiscoveryContext,
        config: &ConfigState,
        system: &SystemState,
        service: &str,
    ) -> Result<PlanDetails, String> {
        let provenance = evaluate_service_provenance(
            context.flake_root(),
            context.configuration_name(),
            service,
        )?;
        let paths = provenance.local_files(context.flake_root());

        if !paths.is_empty() {
            if provenance.contains_local_boolean(context.flake_root(), false) {
                return Ok(Self::no_change(
                    "already-disabled",
                    paths,
                    "service is already disabled in the evaluated configuration",
                ));
            }

            return Ok(Self::change(
                "evaluated-provenance",
                paths,
                "update the existing evaluated service declaration to disabled",
            ));
        }

        match disable_service_strategy(config, system, service) {
            ServiceDisableStrategy::Declared { paths } => Ok(Self::change(
                "existing-declaration",
                paths,
                "disable the existing service declaration",
            )),
            ServiceDisableStrategy::ExistingFile { path } => Ok(Self::change(
                "existing-file",
                vec![path],
                "add a disabling declaration to the existing safe configuration file",
            )),
            ServiceDisableStrategy::NotDeclared => Ok(Self::no_change(
                "not-declared",
                Vec::new(),
                "service is not enabled by a local declaration",
            )),
            ServiceDisableStrategy::Ambiguous { candidates } => Err(format!(
                "multiple service write targets found: {}",
                format_paths(&candidates)
            )),
        }
    }

    fn change(strategy: &str, files: Vec<std::path::PathBuf>, reason: &str) -> PlanDetails {
        PlanDetails {
            strategy: strategy.to_string(),
            affected_files: files,
            change_required: true,
            rebuild_required: true,
            reason: reason.to_string(),
        }
    }

    fn no_change(strategy: &str, files: Vec<std::path::PathBuf>, reason: &str) -> PlanDetails {
        PlanDetails {
            strategy: strategy.to_string(),
            affected_files: files,
            change_required: false,
            rebuild_required: false,
            reason: reason.to_string(),
        }
    }
}

fn format_paths(paths: &[std::path::PathBuf]) -> String {
    paths
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}
