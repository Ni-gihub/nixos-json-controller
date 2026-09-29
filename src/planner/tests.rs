use std::path::PathBuf;

use crate::command::{Action, Target};
use crate::resolver::{ResolvedTarget, Resolver};

use super::Planner;

fn create_resolved_target(name: &str) -> ResolvedTarget {
    ResolvedTarget { name: name.to_string() }
}

#[test]
fn create_install_package_plan() {
    let plan = Planner::create(Action::InstallPackage, create_resolved_target("firefox"));
    assert!(matches!(plan.action, Action::InstallPackage));
    assert_eq!(plan.target.name, "firefox");
    assert!(!plan.dry_run);
    assert!(plan.details.is_none());
}

#[test]
fn create_enable_service_plan() {
    let plan = Planner::create(Action::EnableService, create_resolved_target("openssh"));
    assert!(matches!(plan.action, Action::EnableService));
    assert_eq!(plan.target.name, "openssh");
}

#[test]
fn create_install_package_plan_from_dictionary_alias() {
    let target = Target { raw: "ファイヤーフォックス".to_string() };
    let resolved = Resolver::resolve(Action::InstallPackage, target).unwrap();
    let plan = Planner::create(Action::InstallPackage, resolved);
    assert!(matches!(plan.action, Action::InstallPackage));
    assert_eq!(plan.target.name, "firefox");
}

#[test]
fn plan_details_can_describe_a_noop() {
    let details = super::PlanDetails {
        strategy: "already-present".to_string(),
        affected_files: Vec::<PathBuf>::new(),
        change_required: false,
        rebuild_required: false,
        reason: "already installed".to_string(),
    };
    assert!(!details.change_required);
    assert!(!details.rebuild_required);
}
