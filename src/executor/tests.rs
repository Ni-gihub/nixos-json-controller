use crate::command::Action;
use crate::planner::ExecutionPlan;
use crate::resolver::ResolvedTarget;

use super::Executor;


fn create_plan(
    action: Action,
    target: &str,
) -> ExecutionPlan {

    ExecutionPlan {
        action,

        target: ResolvedTarget {
            name: target.to_string(),
        },
    }

}



#[test]
fn execute_install_package() {

    let plan =
        create_plan(
            Action::InstallPackage,
            "firefox",
        );


    let result =
        Executor::execute(plan);


    assert!(
        result.is_ok()
    );

}



#[test]
fn execute_remove_package() {

    let plan =
        create_plan(
            Action::RemovePackage,
            "firefox",
        );


    let result =
        Executor::execute(plan);


    assert!(
        result.is_ok()
    );

}



#[test]
fn execute_enable_service() {

    let plan =
        create_plan(
            Action::EnableService,
            "openssh",
        );


    let result =
        Executor::execute(plan);


    assert!(
        result.is_ok()
    );

}



#[test]
fn execute_disable_service() {

    let plan =
        create_plan(
            Action::DisableService,
            "openssh",
        );


    let result =
        Executor::execute(plan);


    assert!(
        result.is_ok()
    );

}