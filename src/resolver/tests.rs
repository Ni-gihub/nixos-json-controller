use crate::command::Target;

use super::{
    Resolver,
};


fn create_target(name: &str) -> Target {

    Target {
        raw: name.to_string(),
    }

}


#[test]
fn resolve_package_name(){

    let target =
        create_target("firefox");


    let result =
        Resolver::resolve(target);


    assert_eq!(
        result.name,
        "firefox"
    );
}



#[test]
fn resolve_service_name(){

    let target =
        create_target("openssh");


    let result =
        Resolver::resolve(target);


    assert_eq!(
        result.name,
        "openssh"
    );
}