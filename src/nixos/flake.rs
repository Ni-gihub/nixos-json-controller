use std::fs;


pub fn write_module(
    content: &str
) -> Result<(), String> {


    fs::write(
        "configuration.nix",
        content
    )
    .map_err(
        |e| e.to_string()
    )?;


    Ok(())

}