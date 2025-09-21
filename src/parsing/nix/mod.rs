//! Nix language parser implementation
//!
//! This module provides comprehensive Nix language support for Codanna's code intelligence system,
//! enabling precise symbol extraction, relationship tracking, and semantic analysis of Nix expressions.

pub mod behavior;
pub mod definition;
pub mod parser;
pub mod resolution;

#[cfg(test)]
pub mod test_helpers;

// TODO: Add resolution tests when interface is stable
// #[cfg(test)]
// mod resolution_tests;

pub use behavior::NixBehavior;
pub use definition::NixLanguage;
pub use parser::NixParser;
pub use resolution::{NixInheritanceResolver, NixResolutionContext};

// Re-export for registry registration
pub(crate) use definition::register;
