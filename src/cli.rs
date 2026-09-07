use std::env;

use nixos_json_controller::{
    command::{
        Action,
        Command,
        Target,
    },
    validator::Validator,
    resolver::Resolver,
    planner::Planner,
    executor::Executor,
};


pub fn run() -> Result<(), String> {

    let args: Vec<String> =
        env::args()
            .skip(1)
            .collect();


    if args.len() == 1 && (args[0] == "--help" || args[0] == "-h") {
        println!("usage: nxc [i|r|e|d] <target>");
        println!();
        println!("commands:");
        println!("  nxc <package>       install package");
        println!("  nxc i <package>     install package");
        println!("  nxc r <package>     remove package");
        println!("  nxc e <service>     enable service");
        println!("  nxc d <service>     disable service");

        return Ok(());
    }


    let (action, target) =
        match args.as_slice() {

            // nxc firefox
            [target] => (
                Action::InstallPackage,
                target.clone()
            ),

            // nxc i firefox
            [operation, target] => {

                let action =
                    match operation.as_str() {

                        "i" =>
                            Action::InstallPackage,

                        "r" =>
                            Action::RemovePackage,

                        "e" =>
                            Action::EnableService,

                        "d" =>
                            Action::DisableService,

                        _ =>
                            return Err(
                                "unknown operation. use i, r, e, or d"
                                    .to_string()
                            ),
                    };


                (
                    action,
                    target.clone()
                )
            },

            _ =>
                return Err(
                    "usage: nxc [i|r|e|d] <target>"
                        .to_string()
                ),
        };


    let command =
        Command {
            action,
            target: Target {
                raw: target,
            },
        };


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