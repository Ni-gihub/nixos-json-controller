use crate::command::Action;
use crate::nixos;
use crate::planner::ExecutionPlan;

use super::error::ExecutorError;

pub struct Executor;

impl Executor {
    pub fn execute(plan: ExecutionPlan) -> Result<(), ExecutorError> {
        Self::execute_with_password(plan, None)
    }

    pub fn execute_with_password(
        plan: ExecutionPlan,
        password: Option<&str>,
    ) -> Result<(), ExecutorError> {
        if plan.dry_run {
            println!("========== Dry Run ==========");
            println!("Action : {:?}", plan.action);
            println!("Target : {}", plan.target.name);
            println!("=============================");
            return Ok(());
        }

        Self::execute_with_rebuild_password(plan, true, password)
    }

    pub fn execute_with_rebuild(plan: ExecutionPlan, rebuild: bool) -> Result<(), ExecutorError> {
        Self::execute_with_rebuild_password(plan, rebuild, None)
    }

    pub fn execute_with_rebuild_password(
        plan: ExecutionPlan,
        rebuild: bool,
        password: Option<&str>,
    ) -> Result<(), ExecutorError> {
        let package_change = match plan.action {
            Action::InstallPackage | Action::RemovePackage => {
                Some(super::package::execute_with_change(plan.clone())?)
            }
            Action::EnableService | Action::DisableService => None,
        };

        let service_change = match plan.action {
            Action::EnableService | Action::DisableService => {
                Some(super::service::execute(plan.clone())?)
            }
            Action::InstallPackage | Action::RemovePackage => None,
        };

        let changed = package_change
            .as_ref()
            .map(|change| change.is_changed())
            .or_else(|| service_change.as_ref().map(|change| change.is_changed()))
            .unwrap_or(false);

        if should_rebuild(&plan, changed, rebuild) {
            if let Err(error) = nixos::rebuild::switch_with_password(password) {
                return rollback_after_rebuild_failure(
                    error,
                    package_change.as_ref(),
                    service_change.as_ref(),
                );
            }

            let verification_result = match plan.action {
                Action::InstallPackage | Action::RemovePackage => {
                    verify_package_state(&plan.action, &plan.target.name)
                }
                Action::EnableService | Action::DisableService => {
                    verify_service_state(&plan.action, &plan.target.name)
                }
            };

            if let Err(error) = verification_result {
                return rollback_after_verification_failure(
                    error,
                    package_change.as_ref(),
                    service_change.as_ref(),
                    password,
                );
            }
        }

        Ok(())
    }
}

fn rollback_after_rebuild_failure(
    error: String,
    package_change: Option<&super::package::PackageChange>,
    service_change: Option<&super::service::ServiceChange>,
) -> Result<(), ExecutorError> {
    let rollback_result = rollback_change(package_change, service_change);
    if let Err(rollback_error) = rollback_result {
        return Err(ExecutorError::NixosError(format!(
            "nixos-rebuild failed: {}; rollback also failed: {}",
            error, rollback_error
        )));
    }

    Err(ExecutorError::NixosError(error))
}

fn rollback_after_verification_failure(
    error: String,
    package_change: Option<&super::package::PackageChange>,
    service_change: Option<&super::service::ServiceChange>,
    password: Option<&str>,
) -> Result<(), ExecutorError> {
    let rollback_result = rollback_change(package_change, service_change);
    if let Err(rollback_error) = rollback_result {
        return Err(ExecutorError::NixosError(format!(
            "{}; rollback also failed: {}",
            error, rollback_error
        )));
    }

    // The active system was already switched, so rebuild once more from the
    // restored configuration to return the machine to the previous state.
    if let Err(rebuild_error) = nixos::rebuild::switch_with_password(password) {
        return Err(ExecutorError::NixosError(format!(
            "{}; rollback succeeded but restoring the previous system failed: {}",
            error, rebuild_error
        )));
    }

    Err(ExecutorError::NixosError(error))
}

fn rollback_change(
    package_change: Option<&super::package::PackageChange>,
    service_change: Option<&super::service::ServiceChange>,
) -> Result<(), String> {
    if let Some(change) = package_change {
        change.rollback()
    } else if let Some(change) = service_change {
        change.rollback()
    } else {
        Ok(())
    }
}

fn verify_package_state(action: &Action, package: &str) -> Result<(), String> {
    let system = nixos::system::SystemState::discover()?;
    let present = system.has_package(package) || system.has_command(package);

    verify_package_presence(action, package, present)
}

fn verify_package_presence(action: &Action, package: &str, present: bool) -> Result<(), String> {
    match action {
        Action::InstallPackage if present => Ok(()),
        Action::RemovePackage if !present => Ok(()),
        Action::InstallPackage => Err(format!(
            "nixos-rebuild completed but package '{}' was not found in the active system",
            package
        )),
        Action::RemovePackage => Err(format!(
            "nixos-rebuild completed but package '{}' is still present in the active system",
            package
        )),
        _ => Ok(()),
    }
}

fn verify_service_state(action: &Action, service: &str) -> Result<(), String> {
    let context = nixos::flake::discovery_context()?;
    let provenance = nixos::provenance::evaluate_service_provenance(
        context.flake_root(),
        context.configuration_name(),
        service,
    )?;

    let local_files = provenance.local_files(context.flake_root());
    let candidates = if local_files.is_empty() {
        // When there is no local definition, prefer the service-level option,
        // then fall back to a raw systemd unit option for custom services.
        vec![
            format!("services.{service}.enable"),
            format!("systemd.services.{service}.enable"),
        ]
    } else {
        vec![provenance.option]
    };

    let mut evaluation_errors = Vec::new();
    for option in candidates {
        match nixos::provenance::evaluate_boolean_option(
            context.flake_root(),
            context.configuration_name(),
            &option,
        ) {
            Ok(enabled) => return verify_service_option_value(action, service, &option, enabled),
            Err(error) => evaluation_errors.push(format!("{}: {}", option, error)),
        }
    }

    Err(format!(
        "could not evaluate the effective NixOS enable option for service '{}': {}",
        service,
        evaluation_errors.join("; ")
    ))
}

fn verify_service_option_value(
    action: &Action,
    service: &str,
    option: &str,
    enabled: bool,
) -> Result<(), String> {
    match action {
        Action::EnableService if enabled => Ok(()),
        Action::DisableService if !enabled => Ok(()),
        Action::EnableService => Err(format!(
            "nixos-rebuild completed but effective NixOS option '{}' for service '{}' is false; expected true",
            option, service
        )),
        Action::DisableService => Err(format!(
            "nixos-rebuild completed but effective NixOS option '{}' for service '{}' is true; expected false",
            option, service
        )),
        _ => Ok(()),
    }
}

fn should_rebuild(plan: &ExecutionPlan, changed: bool, rebuild: bool) -> bool {
    if !rebuild {
        return false;
    }

    changed
        || plan
            .details
            .as_ref()
            .is_some_and(|details| details.rebuild_required)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::PlanDetails;
    use std::path::PathBuf;

    fn plan_with_rebuild_required(rebuild_required: bool) -> ExecutionPlan {
        ExecutionPlan {
            action: Action::InstallPackage,
            target: crate::resolver::ResolvedTarget {
                name: "firefox".to_string(),
            },
            dry_run: false,
            details: Some(PlanDetails {
                strategy: "test".to_string(),
                affected_files: vec![PathBuf::from("packages.nix")],
                change_required: false,
                rebuild_required,
                reason: "test".to_string(),
            }),
        }
    }

    #[test]
    fn rebuilds_when_plan_requires_it_without_file_changes() {
        let plan = plan_with_rebuild_required(true);

        assert!(should_rebuild(&plan, false, true));
    }

    #[test]
    fn does_not_rebuild_when_plan_does_not_require_it_without_changes() {
        let plan = plan_with_rebuild_required(false);

        assert!(!should_rebuild(&plan, false, true));
    }

    #[test]
    fn does_not_rebuild_when_rebuild_is_disabled() {
        let plan = plan_with_rebuild_required(true);

        assert!(!should_rebuild(&plan, false, false));
    }

    #[test]
    fn accepts_package_when_install_verification_finds_it() {
        assert!(verify_package_presence(&Action::InstallPackage, "firefox", true).is_ok());
    }

    #[test]
    fn rejects_install_verification_when_package_is_missing() {
        let error = verify_package_presence(&Action::InstallPackage, "firefox", false).unwrap_err();

        assert!(error.contains("package 'firefox' was not found in the active system"));
    }

    #[test]
    fn accepts_package_removal_when_package_is_absent() {
        assert!(verify_package_presence(&Action::RemovePackage, "firefox", false).is_ok());
    }

    #[test]
    fn rejects_removal_verification_when_package_is_still_present() {
        let error = verify_package_presence(&Action::RemovePackage, "firefox", true).unwrap_err();

        assert!(error.contains("package 'firefox' is still present in the active system"));
    }
    #[test]
    fn accepts_service_enable_when_effective_option_is_true() {
        assert!(
            verify_service_option_value(
                &Action::EnableService,
                "openssh",
                "services.openssh.enable",
                true
            )
            .is_ok()
        );
    }

    #[test]
    fn accepts_service_disable_when_effective_option_is_false() {
        assert!(
            verify_service_option_value(
                &Action::DisableService,
                "openssh",
                "services.openssh.enable",
                false
            )
            .is_ok()
        );
    }

    #[test]
    fn rejects_service_enable_when_effective_option_remains_false() {
        let error = verify_service_option_value(
            &Action::EnableService,
            "openssh",
            "services.openssh.enable",
            false,
        )
        .unwrap_err();

        assert!(error.contains("effective NixOS option 'services.openssh.enable'"));
        assert!(error.contains("expected true"));
    }

    #[test]
    fn rejects_service_disable_when_effective_option_remains_true() {
        let error = verify_service_option_value(
            &Action::DisableService,
            "openssh",
            "services.openssh.enable",
            true,
        )
        .unwrap_err();

        assert!(error.contains("effective NixOS option 'services.openssh.enable'"));
        assert!(error.contains("expected false"));
    }
}