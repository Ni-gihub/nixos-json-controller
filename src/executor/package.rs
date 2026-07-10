use crate::nixos;


use super::error::ExecutorError;



pub fn install(
    package: &str
) -> Result<(), ExecutorError> {


    let module =
        nixos::module::add_package(
            package
        )
        .map_err(
            ExecutorError::PackageError
        )?;


    nixos::flake::write_module(
        &module
    )
    .map_err(
        ExecutorError::PackageError
    )?;


    Ok(())

}




pub fn remove(
    package: &str
) -> Result<(), ExecutorError> {


    let module =
        nixos::module::remove_package(
            package
        )
        .map_err(
            ExecutorError::PackageError
        )?;


    nixos::flake::write_module(
        &module
    )
    .map_err(
            ExecutorError::PackageError
    )?;


    Ok(())

}