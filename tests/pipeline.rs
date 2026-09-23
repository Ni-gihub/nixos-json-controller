use nixos_json_controller::{
    command::parser,
    executor::Executor,
    planner::Planner,
    resolver::Resolver,
    validator::Validator,
};

#[test]
fn install_package_pipeline() {
    let json = r#"
    {
        "action": "install_package",
        "target": "firefox"
    }
    "#;

    // JSON
    let command = parser::parse(json).unwrap();

    // validation
    Validator::validate(&command).unwrap();

    // actionを後でも使うためclone
    let action = command.action.clone();

    // resolve
    let resolved =
        Resolver::resolve(action, command.target)
            .expect("failed to resolve target");

    // plan
    let plan = Planner::create(command.action, resolved);

    // execute
    let result = Executor::execute(plan);

    assert!(result.is_ok());
}

#[test]
fn enable_service_pipeline() {
    let json = r#"
    {
        "action": "enable_service",
        "target": "openssh"
    }
    "#;

    // JSON
    let command = parser::parse(json).unwrap();

    // validation
    Validator::validate(&command).unwrap();

    // actionを後でも使うためclone
    let action = command.action.clone();

    // resolve
    let resolved =
        Resolver::resolve(action, command.target)
            .expect("failed to resolve target");

    // plan
    let plan = Planner::create(command.action, resolved);

    // execute
    let result = Executor::execute(plan);

    assert!(result.is_ok());
}

#[test]
fn remove_package_pipeline() {
    let json = r#"
    {
        "action": "remove_package",
        "target": "firefox"
    }
    "#;

    // JSON
    let command = parser::parse(json).unwrap();

    // validation
    Validator::validate(&command).unwrap();

    // actionを後でも使うためclone
    let action = command.action.clone();

    // resolve
    let resolved =
        Resolver::resolve(action, command.target)
            .expect("failed to resolve target");

    // plan
    let plan = Planner::create(command.action, resolved);

    // execute
    let result = Executor::execute(plan);

    assert!(result.is_ok());
}

#[test]
fn disable_service_pipeline() {
    let json = r#"
    {
        "action": "disable_service",
        "target": "openssh"
    }
    "#;

    // JSON
    let command = parser::parse(json).unwrap();

    // validation
    Validator::validate(&command).unwrap();

    // actionを後でも使うためclone
    let action = command.action.clone();

    // resolve
    let resolved =
        Resolver::resolve(action, command.target)
            .expect("failed to resolve target");

    // plan
    let plan = Planner::create(command.action, resolved);

    // execute
    let result = Executor::execute(plan);

    assert!(result.is_ok());
}