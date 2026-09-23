use std::fs;

use super::flake;

pub fn ensure_generated_module() -> Result<(), String> {
    let flake_path =
        flake::repository_path()?;

    let flake_file =
        flake_path.join("flake.nix");

    let mut flake =
        fs::read_to_string(&flake_file)
            .map_err(|e| {
                format!(
                    "failed to read {}: {}",
                    flake_file.display(),
                    e
                )
            })?;

    if flake.contains(
        "./modules/generated.nix"
    ) {
        return Ok(());
    }

    let modules = "modules = [";

    let pos =
        flake.find(modules)
            .ok_or_else(|| {
                format!(
                    "modules section not found in {}",
                    flake_file.display()
                )
            })?;

    let insert =
        pos + modules.len();

    flake.insert_str(
        insert,
        "\n          ./modules/generated.nix",
    );

    fs::write(
        &flake_file,
        flake,
    )
    .map_err(|e| {
        format!(
            "failed to write {}: {}",
            flake_file.display(),
            e
        )
    })?;

    Ok(())
}