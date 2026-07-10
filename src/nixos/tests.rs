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
fn write_generated_module() {


    let path =
        "/tmp/test-module.nix";


    let content =
        "environment.systemPackages = with pkgs; [ firefox ];";


    flake::write_module(
        path
    )

    .unwrap();


    let result =
        std::fs::read_to_string(
            path
        )
        .unwrap();


    assert!(
        result.contains(
            "firefox"
        )
    );


}



#[test]
fn create_rebuild_command() {


    let command =
        rebuild::build_command();


    let program =
        command
            .get_program()
            .to_str()
            .unwrap();


    assert_eq!(
        program,
        "nixos-rebuild"
    );


    let args =
        command
            .get_args()
            .collect::<Vec<_>>();


    assert_eq!(
        args.len(),
        1
    );


    assert_eq!(
        args[0],
        "switch"
    );

}


#[test]
fn write_configuration_file() {

    let content =
        "environment.systemPackages = [ pkgs.firefox ];";


    let result =
        flake::write_module(
            content
        );


    assert!(
        result.is_ok()
    );


    let file =
        std::fs::read_to_string(
            "configuration.nix"
        )
        .unwrap();


    assert!(
        file.contains(
            "firefox"
        )
    );

}