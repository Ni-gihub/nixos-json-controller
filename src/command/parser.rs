use serde::Deserialize;

use super::{
    action::Action,
    command::Command,
    target::Target,
};


#[derive(Debug, Deserialize)]
struct RawCommand {
    action: Action,
    target: String,
}


pub fn parse(json: &str) -> Result<Command, serde_json::Error> {

    let raw: RawCommand = serde_json::from_str(json)?;

    Ok(Command {
        action: raw.action,
        target: Target {
            raw: raw.target,
        },
    })
}