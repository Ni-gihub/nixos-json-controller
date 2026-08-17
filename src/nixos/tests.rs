use super::flake;
use super::generator::NixModule;
use super::module;
use super::rebuild;



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



#[test]
fn repository_paths() {

    assert_eq!(
        flake::repository_path(),
        "../nix-config"
    );


    assert_eq!(
        flake::pkgs_path(),
        "../nix-config/modules/pkgs.nix"
    );

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


    assert_eq!(
        args,
        vec![
            "nixos-rebuild",
            "switch",
            "--flake",
            "../nix-config#laptop",
        ]
    );
}



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