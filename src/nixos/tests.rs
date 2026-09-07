use super::flake;
use super::generator::NixModule;
use super::module;
use super::rebuild;

use std::path::PathBuf;


// ============================================================
// Generator
// ============================================================

#[test]
fn generate_empty_package_module() {
    let module =
        NixModule {
            packages: vec![],
        };

    let result =
        module.generate();

    assert_eq!(
        result,
        "environment.systemPackages = with pkgs; [\n];\n"
    );
}


#[test]
fn generate_single_package_module() {
    let module =
        NixModule {
            packages: vec![
                "firefox".to_string(),
            ],
        };

    let result =
        module.generate();

    assert_eq!(
        result,
        "environment.systemPackages = with pkgs; [\n  firefox\n];\n"
    );
}


#[test]
fn generate_multiple_package_module() {
    let module =
        NixModule {
            packages: vec![
                "firefox".to_string(),
                "git".to_string(),
                "curl".to_string(),
            ],
        };

    let result =
        module.generate();

    assert_eq!(
        result,
        "environment.systemPackages = with pkgs; [\n  firefox\n  git\n  curl\n];\n"
    );
}


// ============================================================
// Module
// ============================================================

#[test]
fn module_add_package() {
    let content =
r#"{ config, pkgs, ... }:

{
environment.systemPackages = with pkgs; [
  git
  curl
];
}
"#;

    let result =
        module::add_package_to_content(
            content,
            "firefox",
        )
        .unwrap();

    assert!(
        result.contains(
            "  firefox"
        )
    );

    assert!(
        result.contains(
            "  git"
        )
    );

    assert!(
        result.contains(
            "  curl"
        )
    );
}


#[test]
fn module_add_package_keeps_existing_content() {
    let content =
r#"{ config, pkgs, ... }:

{
environment.systemPackages = with pkgs; [
  git
];

nixpkgs.config.allowUnfree = true;
}
"#;

    let result =
        module::add_package_to_content(
            content,
            "firefox",
        )
        .unwrap();

    assert!(
        result.contains(
            "  git"
        )
    );

    assert!(
        result.contains(
            "  firefox"
        )
    );

    assert!(
        result.contains(
            "nixpkgs.config.allowUnfree = true;"
        )
    );
}


#[test]
fn module_add_package_requires_package_section() {
    let content =
r#"{ config, pkgs, ... }:

{
}
"#;

    let result =
        module::add_package_to_content(
            content,
            "firefox",
        );

    assert!(
        result.is_err()
    );
}


// ============================================================
// Flake discovery
// ============================================================

#[test]
fn repository_paths() {
    let repository =
        flake::repository_path()
            .unwrap();

    // 実際に発見されたリポジトリが
    // NixOS設定flakeであることを確認する。
    assert!(
        repository.join("flake.nix").is_file()
    );

    assert!(
        repository
            .join("flake.nix")
            .to_str()
            .unwrap()
            .contains("nix-config")
    );

    assert_eq!(
        flake::pkgs_path()
            .unwrap(),
        repository
            .join("modules")
            .join("pkgs.nix")
    );
}


#[test]
fn repository_path_is_not_controller_repository() {
    let repository =
        flake::repository_path()
            .unwrap();

    let controller_repository =
        PathBuf::from(
            env!("CARGO_MANIFEST_DIR")
        );

    assert_ne!(
        repository,
        controller_repository
    );
}


#[test]
fn repository_contains_nixos_configuration() {
    let repository =
        flake::repository_path()
            .unwrap();

    let flake_content =
        std::fs::read_to_string(
            repository.join("flake.nix")
        )
        .unwrap();

    assert!(
        flake_content.contains(
            "nixosConfigurations"
        )
    );
}


// ============================================================
// Rebuild
// ============================================================

#[test]
fn create_rebuild_command() {
    let command =
        rebuild::build_command()
            .unwrap();

    assert_eq!(
        command
            .get_program()
            .to_str()
            .unwrap(),
        "sudo"
    );

    let args: Vec<_> =
        command
            .get_args()
            .map(
                |arg| {
                    arg.to_str()
                        .unwrap()
                }
            )
            .collect();

    let repository =
        flake::repository_path()
            .unwrap();

    let expected_flake =
        format!(
            "{}#laptop",
            repository.display()
        );

    assert_eq!(
        args,
        vec![
            "nixos-rebuild",
            "switch",
            "--flake",
            expected_flake.as_str(),
        ]
    );
}


// ============================================================
// Generator integration
// ============================================================

#[test]
fn generate_module_contains_package() {
    let module =
        NixModule {
            packages: vec![
                "firefox".to_string(),
            ],
        };

    let result =
        module.generate();

    assert!(
        result.contains(
            "firefox"
        )
    );

    assert!(
        result.contains(
            "environment.systemPackages"
        )
    );
}
