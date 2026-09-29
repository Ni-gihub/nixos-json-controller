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

#[test]
fn catalog_entries_are_sorted_and_expose_state() {
    let catalog = NxcCore::catalog().unwrap();
    assert!(catalog.packages.windows(2).all(|w| w[0].name <= w[1].name));
    assert!(catalog.services.windows(2).all(|w| w[0].name <= w[1].name));
}

#[test]
fn catalog_search_matches_canonical_names() {
    let catalog = NxcCore::search_catalog("firefox").unwrap();
    assert!(catalog.packages.iter().any(|entry| entry.name == "firefox"));
}
