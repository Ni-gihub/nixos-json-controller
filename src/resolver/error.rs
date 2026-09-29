use std::fmt;

#[derive(Debug)]
pub enum ResolverError {
    UnknownTarget,
}

impl fmt::Display for ResolverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownTarget => f.write_str("target is not known to the NXC dictionary"),
        }
    }
}

impl std::error::Error for ResolverError {}
