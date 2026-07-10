use nixos_json_controller::{
    command::parser,
    validator::Validator,
    resolver::Resolver,
    planner::Planner,
    executor::Executor,
};


#[test]
fn install_package_pipeline(){

    let json = r#"
    {
        "action":"install_package",
        "target":"firefox"
    }
    "#;


    // JSON
    let command =
        parser::parse(json)
        .unwrap();


    // validation
    Validator::validate(
        &command
    )
    .unwrap();


    // resolve
    let resolved =
        Resolver::resolve(
            command.target
        );


    // plan
    let plan =
        Planner::create(
            command.action,
            resolved
        );


    // execute
    let result =
        Executor::execute(
            plan
        );


    assert!(
        result.is_ok()
    );
}

#[test]
fn enable_service_pipeline(){

    let json = r#"
    {
        "action":"enable_service",
        "target":"openssh"
    }
    "#;


    let command =
        parser::parse(json)
        .unwrap();


    Validator::validate(
        &command
    )
    .unwrap();


    let resolved =
        Resolver::resolve(
            command.target
        );


    let plan =
        Planner::create(
            command.action,
            resolved
        );


    let result =
        Executor::execute(
            plan
        );


    assert!(
        result.is_ok()
    );
}