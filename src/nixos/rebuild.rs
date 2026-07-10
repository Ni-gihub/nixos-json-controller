use std::process::Command;


pub fn build_command() -> Command {

    let mut command =
        Command::new(
            "nixos-rebuild"
        );


    command
        .arg("switch");


    command

}



pub fn switch() -> Result<(), String> {


    let status =
        build_command()
            .status()
            .map_err(
                |e|
                e.to_string()
            )?;


    if status.success() {

        Ok(())

    } else {

        Err(
            "nixos-rebuild failed"
                .to_string()
        )

    }

}