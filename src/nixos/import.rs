use std::fs;

use super::flake;

pub fn ensure_generated_module() -> Result<(), String> {
    let flake_path =
        flake::repository_path()?;

    let flake_file =
        flake_path.join("flake.nix");

    let mut flake =
        fs::read_to_string(
            &flake_file
        )
        .map_err(
            |e| e.to_string()
        )?;

    if flake.contains(
        "./modules/generated.nix"
    ) {
        return Ok(());
    }

    let modules =
        "modules = [";

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
        flake_file,
        flake
    )
    .map_err(
        |e| e.to_string()
    )?;

    Ok(())
}