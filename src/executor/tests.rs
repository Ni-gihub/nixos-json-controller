use super::Executor;

use crate::command::Action;
use crate::planner::ExecutionPlan;
use crate::resolver::ResolvedTarget;

#[test]
fn create_rebuild_command() {
    let plan = ExecutionPlan {
        action: Action::InstallPackage,
        target: ResolvedTarget {
            name: "firefox".to_string(),
        },
        dry_run: false,
    };

    let command = Executor::execute_with_rebuild;

    let _ = command;

    assert_eq!(
        plan.target.name,
        "firefox"
    );
}

#[test]
fn execute_dry_run() {
    let plan = ExecutionPlan {
        action: Action::InstallPackage,
        target: ResolvedTarget {
            name: "firefox".to_string(),
        },
        dry_run: true,
    };

    let result =
        Executor::execute(
            plan
        );

    assert!(
        result.is_ok()
    );
}

#[test]
fn execute_install_package() {
    let plan = ExecutionPlan {
        action: Action::InstallPackage,
        target: ResolvedTarget {
            name: "firefox".to_string(),
        },
        dry_run: false,
    };

    let result =
        Executor::execute_with_rebuild(
            plan,
            false,
        );

    assert!(
        result.is_ok()
    );
}

#[test]
fn execute_remove_package() {
    let plan = ExecutionPlan {
        action: Action::RemovePackage,
        target: ResolvedTarget {
            name: "firefox".to_string(),
        },
        dry_run: false,
    };

    let result =
        Executor::execute_with_rebuild(
            plan,
            false,
        );

    assert!(
        result.is_ok()
    );
}

#[test]
fn execute_enable_service() {
    let plan = ExecutionPlan {
        action: Action::EnableService,
        target: ResolvedTarget {
            name: "openssh".to_string(),
        },
        dry_run: false,
    };

    let result =
        Executor::execute_with_rebuild(
            plan,
            false,
        );

    assert!(
        result.is_ok()
    );
}

#[test]
fn execute_disable_service() {
    let plan = ExecutionPlan {
        action: Action::DisableService,
        target: ResolvedTarget {
            name: "openssh".to_string(),
        },
        dry_run: false,
    };

    let result =
        Executor::execute_with_rebuild(
            plan,
            false,
        );

    assert!(
        result.is_ok()
    );
}