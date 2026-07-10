use std::io::{
    self,
    Read,
};


use nixos_json_controller::{
    command,
    validator::Validator,
    resolver::Resolver,
    planner::Planner,
    executor::Executor,
};



pub fn run() -> Result<(), String> {


    let mut input =
        String::new();


    io::stdin()
        .read_to_string(
            &mut input
        )
        .map_err(
            |e| e.to_string()
        )?;



    let command =
        command::parse(
            &input
        )
        .map_err(
            |e| e.to_string()
        )?;



    Validator::validate(
        &command
    )
    .map_err(
        |e| format!("{:?}", e)
    )?;



    let target =
        Resolver::resolve(
            command.target
        );



    let plan =
        Planner::create(
            command.action,
            target
        );



    Executor::execute(
        plan
    )
    .map_err(
        |e| format!("{:?}", e)
    )?;


    Ok(())

}