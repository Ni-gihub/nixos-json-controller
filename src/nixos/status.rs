use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde::Serialize;

use super::{
    config::ConfigState, discovery::DiscoveryContext, provenance::evaluate_service_provenance,
    system::{PackageSystemSnapshot, SystemState},
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
    pub configured: Option<bool>,
    pub active: Option<bool>,
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

fn log_perf_duration(stage: &str, elapsed: Duration) {
    if matches!(std::env::var("NXC_PERF_DEBUG").as_deref(), Ok("1")) {
        eprintln!("[nxc-perf] {stage} elapsed_ms={}", elapsed.as_millis());
    }
}

pub fn package_states(packages: &[String]) -> Result<Vec<PackageState>, String> {
    if packages.is_empty() {
        return Ok(Vec::new());
    }

    let total_started = Instant::now();

    let stage_started = Instant::now();
    let context = DiscoveryContext::load()?;
    log_perf_duration("package_states.discovery_context", stage_started.elapsed());

    let stage_started = Instant::now();
    let provenance = super::provenance::evaluate_option(
        context.flake_root(),
        context.configuration_name(),
        "environment.systemPackages",
    )?;
    log_perf_duration(
        "package_states.environment_system_packages_eval",
        stage_started.elapsed(),
    );

    // Package status does not need binary discovery or systemd service enumeration.
    // This snapshot also reuses nix-store results while the active generation is unchanged.
    let stage_started = Instant::now();
    let system = PackageSystemSnapshot::discover()?;
    log_perf_duration(
        "package_states.active_system_snapshot",
        stage_started.elapsed(),
    );

    let stage_started = Instant::now();
    let output_names = super::provenance::evaluate_package_output_names(
        context.flake_root(),
        context.configuration_name(),
        packages,
    )?;
    log_perf_duration(
        "package_states.package_output_names_eval",
        stage_started.elapsed(),
    );

    let states =
        package_states_from_output_names(packages, output_names, &provenance, &system)?;
    log_perf_duration("package_states.total", total_started.elapsed());
    states
}

pub fn package_state(package: &str) -> Result<PackageState, String> {
    package_states(&[package.to_string()])?
        .into_iter()
        .next()
        .ok_or_else(|| "package state was not returned".to_string())
}

fn package_states_from_output_names(
    packages: &[String],
    output_names: Vec<Option<Vec<String>>>,
    provenance: &super::provenance::EvaluatedProvenance,
    system: &PackageSystemSnapshot,
) -> Result<Vec<PackageState>, String> {
    if packages.len() != output_names.len() {
        return Err(format!(
            "received {} package names but {} output-name results",
            packages.len(),
            output_names.len()
        ));
    }

    Ok(packages
        .iter()
        .zip(output_names)
        .map(|(package, output_names)| match output_names {
            Some(output_names) => PackageState {
                name: package.clone(),
                configured: Some(provenance.contains_output_names(&output_names)),
                active: Some(system.has_any_package_output(&output_names)),
            },
            None => PackageState {
                name: package.clone(),
                configured: None,
                active: None,
            },
        })
        .collect())
}

pub fn explain_package(
    context: &DiscoveryContext,
    system: &SystemState,
    package: &str,
) -> Result<PackageExplanation, String> {
    // Resolve the package exactly once and use its full derivation name for
    // both active-system and provenance checks.
    let output_names = super::provenance::evaluate_package_output_names_for_package(
        context.flake_root(),
        context.configuration_name(),
        package,
    )?;
    let provenance = super::provenance::evaluate_package_provenance_with_output_names(
        context.flake_root(),
        context.configuration_name(),
        package,
        Some(&output_names),
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
        system_present: system.has_any_package_output(&output_names),
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
    use std::collections::BTreeSet;

    #[test]
    fn unresolved_package_does_not_hide_other_batch_states() {
        let system = PackageSystemSnapshot {
            packages: BTreeSet::from(["valid-1.0".to_string()]),
        };
        let provenance = super::super::provenance::EvaluatedProvenance::default();
        let states = package_states_from_output_names(
            &["valid".to_string(), "missing".to_string()],
            vec![Some(vec!["valid-1.0".to_string()]), None],
            &provenance,
            &system,
        )
        .unwrap();

        assert_eq!(states.len(), 2);
        assert_eq!(states[0].active, Some(true));
        assert_eq!(states[0].configured, Some(false));
        assert_eq!(states[1].active, None);
        assert_eq!(states[1].configured, None);

        let unresolved_json = serde_json::to_value(&states[1]).unwrap();
        assert!(unresolved_json["active"].is_null());
        assert!(unresolved_json["configured"].is_null());
    }

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
