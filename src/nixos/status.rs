use std::path::PathBuf;

use super::{
    config::ConfigState,
    discovery::DiscoveryContext,
    provenance::{evaluate_package_provenance, evaluate_service_provenance},
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

pub fn explain_package(
    context: &DiscoveryContext,
    system: &SystemState,
    package: &str,
) -> Result<PackageExplanation, String> {
    // Unlike the normal executor fast path, explain intentionally performs
    // provenance evaluation even when the package is already installed.
    // explain is a diagnostic command, so source provenance is part of the
    // requested answer rather than an optional optimization.
    let provenance =
        evaluate_package_provenance(context.flake_root(), context.configuration_name(), package)?;

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
        system_present: system.has_command(package) || system.has_package(package),
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
