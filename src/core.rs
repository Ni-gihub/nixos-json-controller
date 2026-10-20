use crate::{
    command::{Action, Command, Target},
    executor::{Executor, ExecutorError},
    nixos::{
        discovery::DiscoveryContext,
        discovery::{
            DiscoveryReport, DiscoveryStatus, SearchOptions, discover_candidates,
            inspect_candidates, save_discovery, select_candidate,
        },
        status::{PackageExplanation, ServiceExplanation, explain_package, explain_service},
        system::SystemState,
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

    pub fn plan_and_execute(action: Action, target: impl Into<String>) -> Result<(), String> {
        let plan = Self::plan(action, target)?;
        Self::execute(plan).map_err(|e| e.to_string())
    }

    pub fn status() -> Result<SystemState, String> {
        SystemState::discover()
    }

    pub fn catalog() -> Result<AppStoreCatalog, String> {
        let system = SystemState::discover()?;
        let context = DiscoveryContext::load()?;
        Self::catalog_with_system_and_context(&system, &context)
    }

    fn catalog_with_system_and_context(
        system: &SystemState,
        context: &DiscoveryContext,
    ) -> Result<AppStoreCatalog, String> {
        let dictionary = crate::dictionary::Dictionary::load()?;
        let package_entries: Vec<(String, Vec<String>)> = dictionary
            .packages()
            .map(|(name, aliases)| (name.to_string(), aliases.to_vec()))
            .collect();
        let package_names: Vec<String> = package_entries
            .iter()
            .map(|(name, _)| name.clone())
            .collect();
        let output_names = crate::nixos::provenance::evaluate_package_output_names(
            context.flake_root(),
            context.configuration_name(),
            &package_names,
        )?;

        let mut packages = package_catalog_entries(package_entries, output_names, system)?;

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

    // This test helper feeds synthetic output names through the same pure mapping
    // function used by the production catalog after Nix resolves real output names.
    #[cfg(test)]
    pub(crate) fn catalog_with_system(system: &SystemState) -> Result<AppStoreCatalog, String> {
        let dictionary = crate::dictionary::Dictionary::load()?;
        let package_entries: Vec<(String, Vec<String>)> = dictionary
            .packages()
            .map(|(name, aliases)| (name.to_string(), aliases.to_vec()))
            .collect();
        let output_names = package_entries
            .iter()
            .map(|(name, _)| Some(vec![name.clone()]))
            .collect();
        let mut packages = package_catalog_entries(package_entries, output_names, system)?;

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
        let catalog = Self::catalog()?;
        Ok(filter_catalog(catalog, query))
    }

    #[cfg(test)]
    pub(crate) fn search_catalog_with_system(
        query: &str,
        system: &SystemState,
    ) -> Result<AppStoreCatalog, String> {
        Ok(filter_catalog(Self::catalog_with_system(system)?, query))
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
/// Check one resolved output name without applying a package-name prefix match.
pub(crate) fn package_output_present(output_name: Option<&str>, system: &SystemState) -> bool {
    output_name.is_some_and(|output_name| system.has_package_output(output_name))
}

fn package_outputs_present(output_names: Option<&[String]>, system: &SystemState) -> bool {
    output_names.is_some_and(|names| {
        names
            .iter()
            .any(|output_name| package_output_present(Some(output_name), system))
    })
}

fn package_catalog_entries(
    entries: Vec<(String, Vec<String>)>,
    output_names: Vec<Option<Vec<String>>>,
    system: &SystemState,
) -> Result<Vec<CatalogEntry>, String> {
    if entries.len() != output_names.len() {
        return Err(format!(
            "catalog contains {} package entries but Nix returned {} output-name entries",
            entries.len(),
            output_names.len()
        ));
    }

    Ok(entries
        .into_iter()
        .zip(output_names)
        .map(|((name, aliases), output_names)| CatalogEntry {
            name,
            aliases,
            system_present: package_outputs_present(output_names.as_deref(), system),
        })
        .collect())
}

}

fn filter_catalog(catalog: AppStoreCatalog, query: &str) -> AppStoreCatalog {
    let query = query.trim();
    if query.is_empty() {
        return catalog;
    }

    let matches = |entry: &CatalogEntry| {
        entry.name == query
            || entry.name.contains(query)
            || entry
                .aliases
                .iter()
                .any(|alias| alias == query || alias.contains(query))
    };

    AppStoreCatalog {
        packages: catalog.packages.into_iter().filter(&matches).collect(),
        services: catalog.services.into_iter().filter(matches).collect(),
    }
}
