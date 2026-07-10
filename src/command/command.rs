use super::action::Action;
use super::target::Target;


#[derive(Debug)]
pub struct Command {
    pub action: Action,
    pub target: Target,
}