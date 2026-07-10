use crate::command::{
    Action,
    Command,
    Target,
};

use super::{
    Validator,
    ValidationError,
};


fn create_command(target: &str) -> Command {

    Command {
        action: Action::InstallPackage,

        target: Target {
            raw: target.to_string(),
        },
    }

}


#[test]
fn accept_valid_target(){

    let command =
        create_command("firefox");


    let result =
        Validator::validate(&command);


    assert!(
        result.is_ok()
    );
}



#[test]
fn reject_empty_target(){

    let command =
        create_command("");


    let result =
        Validator::validate(&command);


    assert!(
        matches!(
            result,
            Err(
                ValidationError::EmptyTarget
            )
        )
    );
}



#[test]
fn reject_whitespace_only_target(){

    let command =
        create_command("   ");


    let result =
        Validator::validate(&command);


    assert!(
        matches!(
            result,
            Err(
                ValidationError::WhitespaceOnlyTarget
            )
        )
    );
}



#[test]
fn reject_leading_trailing_whitespace(){

    let command =
        create_command(" firefox ");


    let result =
        Validator::validate(&command);

    assert!(
        matches!(
            result,
            Err(
                ValidationError::LeadingOrTrailingWhitespace
            )
        )
    );
}



#[test]
fn reject_too_long_target(){

    let long_target =
        "a".repeat(101);


    let command =
        create_command(&long_target);


    let result =
        Validator::validate(&command);


    assert!(
        matches!(
            result,
            Err(
                ValidationError::TargetTooLong
            )
        )
    );
}