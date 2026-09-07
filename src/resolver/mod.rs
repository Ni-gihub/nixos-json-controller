pub mod error;
pub mod package;
pub mod resolver;
pub mod service;

pub use resolver::{ResolvedTarget, Resolver};

#[cfg(test)]
mod tests;
