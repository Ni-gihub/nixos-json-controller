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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogEntry {
    pub name: String,
    pub aliases: Vec<String>,
    pub system_present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppStoreCatalog {
    pub packages: Vec<CatalogEntry>,
    pub services: Vec<CatalogEntry>,
}

pub struct NxcCore;

impl NxcCore {
    pub fn resolve(action: Action, target: impl Into<String>) -> Result<ResolvedTarget, String> {
        let command = Command {
            action,
            target: Target { raw: target.into() },
        };

        Validator::validate(&command).map_err(|e| e.to_string())?;
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
        Self::execute(plan).map_err(|e| e.to_string())
    }

    pub fn status() -> Result<SystemState, String> {
        SystemState::discover()
    }

    pub fn catalog() -> Result<AppStoreCatalog, String> {
        let system = SystemState::discover()?;
        Self::catalog_with_system(&system)
    }

    pub(crate) fn catalog_with_system(system: &SystemState) -> Result<AppStoreCatalog, String> {
        let dictionary = crate::dictionary::Dictionary::load()?;

        let mut packages = dictionary
            .packages()
            .map(|(name, aliases)| CatalogEntry {
                name: name.to_string(),
                aliases: aliases.to_vec(),
                system_present: system.has_command(name) || system.has_package(name),
            })
            .collect::<Vec<_>>();

        let mut services = dictionary
            .services()
            .map(|(name, aliases)| CatalogEntry {
                name: name.to_string(),
                aliases: aliases.to_vec(),
                system_present: system.is_service_enabled(name),
            })
            .collect::<Vec<_>>();

        packages.sort_by(|a, b| a.name.cmp(&b.name));
        services.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(AppStoreCatalog { packages, services })
    }

    pub fn search_catalog(query: &str) -> Result<AppStoreCatalog, String> {
        let system = SystemState::discover()?;
        Self::search_catalog_with_system(query, &system)
    }

    pub(crate) fn search_catalog_with_system(
        query: &str,
        system: &SystemState,
    ) -> Result<AppStoreCatalog, String> {
        let query = query.trim();
        let catalog = Self::catalog_with_system(system)?;
        if query.is_empty() {
            return Ok(catalog);
        }

        let matches = |entry: &CatalogEntry| {
            entry.name == query
                || entry.name.contains(query)
                || entry
                    .aliases
                    .iter()
                    .any(|alias| alias == query || alias.contains(query))
        };

        Ok(AppStoreCatalog {
            packages: catalog.packages.into_iter().filter(&matches).collect(),
            services: catalog.services.into_iter().filter(matches).collect(),
        })
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
