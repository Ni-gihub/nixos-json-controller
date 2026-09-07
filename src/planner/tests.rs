use crate::command::{
    Action,
    Target,
};

use crate::resolver::Resolver;

use crate::resolver::ResolvedTarget;

use super::Planner;

fn create_resolved_target(name: &str) -> ResolvedTarget {
    ResolvedTarget {
        name: name.to_string(),
    }
}

#[test]
fn create_install_package_plan() {
    let action = Action::InstallPackage;

    let target = create_resolved_target("firefox");

    let plan = Planner::create(action, target);

    assert!(matches!(plan.action, Action::InstallPackage));

    assert_eq!(plan.target.name, "firefox");
}

#[test]
fn create_enable_service_plan() {
    let action = Action::EnableService;

    let target = create_resolved_target("openssh");

    let plan = Planner::create(action, target);

    assert!(matches!(plan.action, Action::EnableService));

    assert_eq!(plan.target.name, "openssh");
}

#[test]
fn create_install_package_plan_from_dictionary_alias() {
    let target = Target {
        raw: "ファイヤーフォックス".to_string(),
    };

    let resolved =
        Resolver::resolve(
            Action::InstallPackage,
            target,
        )
        .unwrap();

    let plan =
        Planner::create(
            Action::InstallPackage,
            resolved,
        );

    assert!(
        matches!(
            plan.action,
            Action::InstallPackage
        )
    );

    assert_eq!(
        plan.target.name,
        "firefox"
    );
}