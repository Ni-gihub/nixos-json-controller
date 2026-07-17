use std::fs;
use std::path::Path;

const MODULES_DIR: &str = "modules";
const GENERATED_MODULE: &str = "modules/generated.nix";



pub fn ensure_modules() -> Result<(), String> {

    if !Path::new(
        MODULES_DIR
    )
    .exists()
    {

        fs::create_dir_all(
            MODULES_DIR
        )
        .map_err(
            |e| e.to_string()
        )?;

    }

    Ok(())

}



pub fn ensure_flake() -> Result<(), String> {

    if Path::new(
        "flake.nix"
    )
    .exists()
    {

        return Ok(());

    }


    let flake =
r#"
{
  outputs = { self, nixpkgs }:
  {
    nixosConfigurations.default =
      nixpkgs.lib.nixosSystem {

        system = "x86_64-linux";

        modules = [
          ./modules/generated.nix
        ];
      };
  };
}
"#;


    fs::write(
        "flake.nix",
        flake
    )
    .map_err(
        |e| e.to_string()
    )?;

    Ok(())

}



pub fn write_module(
    content: &str
) -> Result<(), String> {

    ensure_modules()?;

    fs::write(
        GENERATED_MODULE,
        content
    )
    .map_err(
        |e| e.to_string()
    )?;

    Ok(())

}