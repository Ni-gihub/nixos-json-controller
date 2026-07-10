#[derive(Debug, PartialEq)]
pub enum ValidationError {

    EmptyTarget,

    WhitespaceOnlyTarget,

    LeadingOrTrailingWhitespace,

    TargetTooLong,

}