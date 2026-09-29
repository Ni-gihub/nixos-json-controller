use std::path::PathBuf;

use super::discovery::DiscoveryContext;
use super::provenance::evaluate_package_provenance;
use super::system::SystemState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationState {
    pub system_present: bool,
    pub declared_in_config: bool,
    pub declaration_locations: Vec<PathBuf>,
}

impl ApplicationState {
    pub fn inspect(
        context: &DiscoveryContext,
        package: &str,
        system: &SystemState,
    ) -> Result<Self, String> {
        let system_present = system.has_command(package) || system.has_package(package);

        // The active system is authoritative for the "already installed"
        // fast path. Do not make an otherwise successful no-op depend on
        // provenance evaluation of the source configuration.
        if system_present {
            return Ok(Self {
                system_present: true,
                declared_in_config: false,
                declaration_locations: Vec::new(),
            });
        }

        let provenance = evaluate_package_provenance(
            context.flake_root(),
            context.configuration_name(),
            package,
        )?;

        let declaration_locations = provenance.local_files(context.flake_root());

        Ok(Self {
            system_present: false,
            declared_in_config: !declaration_locations.is_empty(),
            declaration_locations,
        })
    }

    pub fn installed(&self) -> bool {
        self.system_present
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installed_state_is_based_on_system_presence() {
        let state = ApplicationState {
            system_present: true,
            declared_in_config: false,
            declaration_locations: Vec::new(),
        };

        assert!(state.installed());
    }

    #[test]
    fn declaration_state_is_independent_from_system_presence() {
        let state = ApplicationState {
            system_present: false,
            declared_in_config: true,
            declaration_locations: vec![PathBuf::from("packages.nix")],
        };

        assert!(!state.installed());
        assert!(state.declared_in_config);
    }
}
