use std::fmt;

#[derive(Debug)]
pub enum ExecutorError {
    PackageError(String),
    ServiceError(String),
    NixosError(String),
}

impl fmt::Display for ExecutorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PackageError(message) => write!(f, "package operation failed: {message}"),
            Self::ServiceError(message) => write!(f, "service operation failed: {message}"),
            Self::NixosError(message) => write!(f, "NixOS operation failed: {message}"),
        }
    }
}

impl std::error::Error for ExecutorError {}
