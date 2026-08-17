use std::fs;

const NIX_CONFIG_REPOSITORY: &str = "../nix-config";
const PKGS_FILE: &str = "../nix-config/modules/pkgs.nix";
const CORE_FILE: &str = "../nix-config/modules/core.nix";



pub fn write_pkgs(
    content: &str,
) -> Result<(), String> {

    fs::write(
        PKGS_FILE,
        content,
    )
    .map_err(
        |e| e.to_string(),
    )?;

    Ok(())
}



pub fn write_core(
    content: &str,
) -> Result<(), String> {

    fs::write(
        CORE_FILE,
        content,
    )
    .map_err(
        |e| e.to_string(),
    )?;

    Ok(())
}



pub fn read_pkgs() -> Result<String, String> {
    std::fs::read_to_string(
        pkgs_path(),
    )
    .map_err(|e| e.to_string())
}




pub fn read_core() -> Result<String, String> {

    fs::read_to_string(
        CORE_FILE,
    )
    .map_err(
        |e| e.to_string(),
    )
}



pub fn repository_path() -> &'static str {

    NIX_CONFIG_REPOSITORY

}



pub fn pkgs_path() -> &'static str {

    PKGS_FILE

}