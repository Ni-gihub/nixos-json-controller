use std::fs;

const FLAKE: &str = "flake.nix";



pub fn ensure_generated_module() -> Result<(), String> {

    let mut flake =
        fs::read_to_string(
            FLAKE
        )
        .map_err(
            |e| e.to_string()
        )?;


    if flake.contains(
        "./modules/generated.nix"
    ) {

        return Ok(());

    }


    let modules = "modules = [";


    let pos =
        flake
            .find(modules)
            .ok_or(
                "modules section not found"
            )?;


    let insert =
        pos + modules.len();


    flake.insert_str(
        insert,
        "\n          ./modules/generated.nix"
    );


    fs::write(
        FLAKE,
        flake
    )
    .map_err(
        |e| e.to_string()
    )?;


    Ok(())

}