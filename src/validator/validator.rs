use crate::command::Command;

use super::error::ValidationError;


pub struct Validator;


impl Validator {


    pub fn validate(
        command: &Command
    ) -> Result<(), ValidationError> {


        let target =
            &command.target.raw;


        if target.is_empty() {

            return Err(
                ValidationError::EmptyTarget
            );

        }


        if target.trim().is_empty() {

            return Err(
                ValidationError::WhitespaceOnlyTarget
            );

        }


        if target.trim() != target {

            return Err(
                ValidationError::LeadingOrTrailingWhitespace
            );

        }


        if target.len() > 64 {

            return Err(
                ValidationError::TargetTooLong
            );

        }


        Ok(())

    }

}