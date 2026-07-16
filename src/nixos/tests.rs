use super::{
    flake,
    module,
    rebuild,
    generator::NixModule,
};

#[test]
fn module_add_package() {


    let result =
        module::add_package(
            "firefox"
        )
        .unwrap();


    assert!(
        result.contains(
            "firefox"
        )
    );


}



#[test]
fn remove_package() {

    let result =
        module::remove_package(
            "firefox"
        );


    assert!(
        result.is_ok()
    );

}



#[test]
fn module_enable_service() {


    let result =
        module::enable_service(
            "openssh"
        )
        .unwrap();


    assert!(
        result.contains(
            "openssh"
        )
    );


    assert!(
        result.contains(
            "true"
        )
    );

}



#[test]
fn disable_service() {

    let result =
        module::disable_service(
            "openssh"
        );


    assert!(
        result.is_ok()
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


    assert!(
        result.contains("firefox")
    );


    assert!(
        result.contains(
            "environment.systemPackages"
        )
    );

}



#[test]
fn generate_multiple_package_module() {

    let module =
        NixModule {
            packages: vec![
                "firefox".to_string(),
                "vim".to_string(),
            ],
        };


    let result =
        module.generate();


    assert!(
        result.contains("firefox")
    );


    assert!(
        result.contains("vim")
    );

}



#[test]
fn generate_empty_package_module() {

    let module =
        NixModule {
            packages: vec![],
        };


    let result =
        module.generate();


    assert!(
        result.contains(
            "environment.systemPackages"
        )
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
        "nixos-rebuild"
    );

    let args: Vec<_> =
        command
            .get_args()
            .map(|a| {
                a.to_str().unwrap()
            })
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
fn create_flake() {

    flake::ensure_flake()
        .unwrap();


    assert!(
        std::path::Path::new(
            "flake.nix"
        )
        .exists()
    );


    assert!(
        std::path::Path::new(
            "modules/generated.nix"
        )
        .parent()
        .unwrap()
        .exists()
    );

}

#[test]
fn ensure_modules_directory() {

    flake::ensure_modules()
        .unwrap();

    assert!(
        std::path::Path::new(
            "modules"
        )
        .exists()
    );

}


#[test]
fn write_generated_module() {

    let content =
        module::add_package(
            "firefox"
        )
        .unwrap();

    flake::write_module(
        &content
    )
    .unwrap();

    let result =
        std::fs::read_to_string(
            "modules/generated.nix"
        )
        .unwrap();

    assert!(
        result.contains(
            "firefox"
        )
    );

}


#[test]
fn ensure_generated_module_import() {

    let sample = r#"
{
  outputs = { self, nixpkgs }: {

    nixosConfigurations.default =
      nixpkgs.lib.nixosSystem {

        system = "x86_64-linux";

        modules = [
        ];

      };

  };
}
"#;

    std::fs::write(
        "flake.nix",
        sample,
    )
    .unwrap();

    super::import::ensure_generated_module()
        .unwrap();

    let result =
        std::fs::read_to_string("flake.nix")
            .unwrap();

    assert!(
        result.contains("./modules/generated.nix")
    );

}