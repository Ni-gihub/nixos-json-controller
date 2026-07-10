pub mod error;
pub mod package;
pub mod resolver;
pub mod service;


pub use resolver::{
    Resolver,
    ResolvedTarget,
};


#[cfg(test)]
mod tests;