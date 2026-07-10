use super::generator::NixModule;


pub fn add_package(
    package: &str
) -> Result<String, String> {


    let module =
        NixModule {
            packages: vec![
                package.to_string()
            ],
        };


    Ok(
        module.generate()
    )

}



pub fn remove_package(
    package: &str
) -> Result<String, String> {


    Ok(
        format!(
            "# remove package: {}",
            package
        )
    )

}



pub fn enable_service(
    service: &str
) -> Result<String, String> {


    Ok(
        format!(
            "systemd.services.{}.enable = true;",
            service
        )
    )

}



pub fn disable_service(
    service: &str
) -> Result<String, String> {


    Ok(
        format!(
            "systemd.services.{}.enable = false;",
            service
        )
    )

}