use crate::command::{
    Action,
    Target,
};

use super::Resolver;

fn create_target(name: &str) -> Target {
    Target {
        raw: name.to_string(),
    }
}

#[test]
fn resolve_package_name() {
    let target = create_target("firefox");

    let result =
        Resolver::resolve(
            Action::InstallPackage,
            target
        )
        .unwrap();

    assert_eq!(
        result.name,
        "firefox"
    );
}

#[test]
fn resolve_service_name() {
    let target = create_target("openssh");

    let result =
        Resolver::resolve(
            Action::EnableService,
            target
        )
        .unwrap();

    assert_eq!(
        result.name,
        "openssh"
    );
}

#[test]
fn reject_service_as_package() {
    let target =
        create_target("ssh");

    let result =
        Resolver::resolve(
            Action::InstallPackage,
            target
        );

    assert!(
        result.is_err()
    );
}

#[test]
fn reject_package_as_service() {
    let target =
        create_target("firefox");

    let result =
        Resolver::resolve(
            Action::EnableService,
            target
        );

    assert!(
        result.is_err()
    );
}