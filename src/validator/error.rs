use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    EmptyTarget,
    WhitespaceOnlyTarget,
    LeadingOrTrailingWhitespace,
    TargetTooLong,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::EmptyTarget => "target name is required",
            Self::WhitespaceOnlyTarget => "target name cannot be only whitespace",
            Self::LeadingOrTrailingWhitespace => {
                "target name must not have leading or trailing whitespace"
            }
            Self::TargetTooLong => "target name is too long (maximum: 64 bytes)",
        };

        f.write_str(message)
    }
}

impl std::error::Error for ValidationError {}
