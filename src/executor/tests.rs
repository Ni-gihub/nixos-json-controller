use crate::command::Action;
use crate::planner::ExecutionPlan;
use crate::resolver::ResolvedTarget;
use crate::nixos::rebuild;

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

        dry_run: true,
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
        Executor::execute_with_rebuild(
            plan,
            false,
        );

    assert!(result.is_ok());

}

#[test]
fn execute_remove_package() {

    let plan =
        create_plan(
            Action::RemovePackage,
            "firefox",
        );

    let result =
        Executor::execute_with_rebuild(
            plan,
            false,
        );

    assert!(result.is_ok());

}

#[test]
fn execute_enable_service() {

    let plan =
        create_plan(
            Action::EnableService,
            "openssh",
        );

    let result =
        Executor::execute_with_rebuild(
            plan,
            false,
        );

    assert!(result.is_ok());

}

#[test]
fn execute_disable_service() {

    let plan =
        create_plan(
            Action::DisableService,
            "openssh",
        );

    let result =
        Executor::execute_with_rebuild(
            plan,
            false,
        );

    assert!(result.is_ok());

}

#[test]
fn create_rebuild_command() {

    let command =
        rebuild::build_command();

    assert_eq!(
        command
            .get_program()
            .to_str()
            .unwrap(),
        "nixos-rebuild"
    );

    let args: Vec<_> =
        command
            .get_args()
            .map(|a| a.to_str().unwrap())
            .collect();

    assert_eq!(
        args,
        vec![
            "switch",
            "--flake",
            ".#default",
        ]
    );

}

#[test]
fn execute_dry_run() {

    let plan =
        ExecutionPlan {

            action: Action::InstallPackage,

            target: ResolvedTarget {
                name: "firefox".to_string(),
            },

            dry_run: true,
        };

    let result =
        Executor::execute_with_rebuild(
            plan,
            false,
        );

    assert!(result.is_ok());

}