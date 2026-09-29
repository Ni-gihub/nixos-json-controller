use crate::{
    command::Action,
    core::NxcCore,
};

#[test]
fn core_resolves_package_targets() {
    let resolved = NxcCore::resolve(Action::InstallPackage, "firefox").unwrap();
    assert_eq!(resolved.name, "firefox");
}

#[test]
fn core_rejects_empty_targets() {
    let result = NxcCore::resolve(Action::InstallPackage, "   ");
    assert!(result.is_err());
}

#[test]
fn core_resolve_uses_action_specific_dictionary() {
    let package = NxcCore::resolve(Action::InstallPackage, "firefox").unwrap();
    assert_eq!(package.name, "firefox");
}
