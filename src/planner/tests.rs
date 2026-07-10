use crate::command::Action;
use crate::resolver::ResolvedTarget;

use super::Planner;


fn create_target(name: &str) -> ResolvedTarget {
    ResolvedTarget {
        name: name.to_string(),
    }
}


#[test]
fn create_install_package_plan() {

    let action = Action::InstallPackage;

    let target = create_target("firefox");


    let plan =
        Planner::create(
            action,
            target,
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



#[test]
fn create_enable_service_plan() {

    let action = Action::EnableService;

    let target = create_target("openssh");


    let plan =
        Planner::create(
            action,
            target,
        );


    assert!(
        matches!(
            plan.action,
            Action::EnableService
        )
    );


    assert_eq!(
        plan.target.name,
        "openssh"
    );
}