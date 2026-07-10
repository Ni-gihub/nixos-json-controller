use crate::nixos;


use super::error::ExecutorError;



pub fn enable(
    service: &str
) -> Result<(), ExecutorError> {


    let module =
        nixos::module::enable_service(
            service
        )
        .map_err(
            ExecutorError::ServiceError
        )?;


    nixos::flake::write_module(
        &module
    )
    .map_err(
        ExecutorError::ServiceError
    )?;


    Ok(())

}



pub fn disable(
    service: &str
) -> Result<(), ExecutorError> {


    let module =
        nixos::module::disable_service(
            service
        )
        .map_err(
            ExecutorError::ServiceError
        )?;


    nixos::flake::write_module(
        &module
    )
    .map_err(
        ExecutorError::ServiceError
    )?;


    Ok(())

}