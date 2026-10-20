use std::path::PathBuf;

use serde::Serialize;

use super::{
    config::ConfigState, discovery::DiscoveryContext, provenance::evaluate_service_provenance,
    system::SystemState,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageExplanation {
    pub name: String,
    pub system_present: bool,
    pub declared_in_config: bool,
    pub declaration_locations: Vec<PathBuf>,
    pub safe_declaration_locations: Vec<PathBuf>,
    pub unsafe_declaration_locations: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PackageState {
    pub name: String,
    pub configured: bool,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceExplanation {
    pub name: String,
    pub system_enabled: bool,
    pub declared_in_config: bool,
    pub enabled_in_config: bool,
    pub declaration_locations: Vec<PathBuf>,
    pub safe_declaration_locations: Vec<PathBuf>,
    pub unsafe_declaration_locations: Vec<PathBuf>,
}

pub fn package_states(packages: &[String]) -> Result<Vec<PackageState>, String> {
    let context = DiscoveryContext::load()?;
    let provenance = super::provenance::evaluate_option(
        context.flake_root(),
        context.configuration_name(),
        "environment.systemPackages",
    )?;
    let system = SystemState::discover()?;

    packages
        .iter()
        .map(|package| {
            let (active, configured) = if package.contains('.') {
                let output_name = super::provenance::evaluate_package_output_name(
                    context.flake_root(),
                    context.configuration_name(),
                    package,
                )?;
                (
                    system.has_package_output(&output_name),
                    provenance.contains_output_name(&output_name),
                )
            } else {
                (
                    system.has_package_reference(
                        context.flake_root(),
                        context.configuration_name(),
                        package,
                    )?,
                    provenance.contains_package(package),
                )
            };

            Ok(PackageState {
                name: package.clone(),
                configured,
                active,
            })
        })
        .collect()
}

pub fn package_state(package: &str) -> Result<PackageState, String> {
    package_states(&[package.to_string()])?
        .into_iter()
        .next()
        .ok_or_else(|| "package state was not returned".to_string())
}

pub fn explain_package(
    context: &DiscoveryContext,
    system: &SystemState,
    package: &str,
) -> Result<PackageExplanation, String> {
    // Unlike the normal executor fast path, explain intentionally performs
    // provenance evaluation even when the package is already installed.
    // Resolve a dotted path only once and reuse the output name for both checks.
    let output_name = if package.contains('.') {
        Some(super::provenance::evaluate_package_output_name(
            context.flake_root(),
            context.configuration_name(),
            package,
        )?)
    } else {
        None
    };
    let provenance = super::provenance::evaluate_package_provenance_with_output_name(
        context.flake_root(),
        context.configuration_name(),
        package,
        output_name.as_deref(),
    )?;

    let declaration_locations = provenance.local_files(context.flake_root());
    let config = ConfigState::discover(context.flake_root())?;

    let safe_declaration_locations = declaration_locations
        .iter()
        .filter(|path| config.is_safe_write_target(path))
        .cloned()
        .collect();

    let unsafe_declaration_locations = declaration_locations
        .iter()
        .filter(|path| !config.is_safe_write_target(path))
        .cloned()
        .collect();

    Ok(PackageExplanation {
        name: package.to_string(),
        system_present: match output_name.as_deref() {
            Some(output_name) => system.has_package_output(output_name),
            None => system.has_package_reference(
                context.flake_root(),
                context.configuration_name(),
                package,
            )?,
        },
        declared_in_config: !declaration_locations.is_empty(),
        declaration_locations,
        safe_declaration_locations,
        unsafe_declaration_locations,
    })
}

pub fn explain_service(
    context: &DiscoveryContext,
    system: &SystemState,
    service: &str,
) -> Result<ServiceExplanation, String> {
    let provenance =
        evaluate_service_provenance(context.flake_root(), context.configuration_name(), service)?;

    let declaration_locations = provenance.local_files(context.flake_root());
    let config = ConfigState::discover(context.flake_root())?;

    let safe_declaration_locations = declaration_locations
        .iter()
        .filter(|path| config.is_safe_write_target(path))
        .cloned()
        .collect();

    let unsafe_declaration_locations = declaration_locations
        .iter()
        .filter(|path| !config.is_safe_write_target(path))
        .cloned()
        .collect();

    Ok(ServiceExplanation {
        name: service.to_string(),
        system_enabled: system.is_service_enabled(service),
        declared_in_config: !declaration_locations.is_empty(),
        enabled_in_config: provenance.contains_local_boolean(context.flake_root(), true),
        declaration_locations,
        safe_declaration_locations,
        unsafe_declaration_locations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_explanation_tracks_system_and_config_state() {
        let explanation = PackageExplanation {
            name: "firefox".to_string(),
            system_present: true,
            declared_in_config: true,
            declaration_locations: vec![PathBuf::from("packages.nix")],
            safe_declaration_locations: vec![PathBuf::from("packages.nix")],
            unsafe_declaration_locations: Vec::new(),
        };

        assert!(explanation.system_present);
        assert!(explanation.declared_in_config);
        assert!(explanation.unsafe_declaration_locations.is_empty());
    }

    #[test]
    fn service_explanation_distinguishes_enabled_state() {
        let explanation = ServiceExplanation {
            name: "openssh".to_string(),
            system_enabled: true,
            declared_in_config: true,
            enabled_in_config: true,
            declaration_locations: vec![PathBuf::from("services.nix")],
            safe_declaration_locations: vec![PathBuf::from("services.nix")],
            unsafe_declaration_locations: Vec::new(),
        };

        assert!(explanation.system_enabled);
        assert!(explanation.enabled_in_config);
    }
}
