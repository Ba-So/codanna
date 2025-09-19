//! Language-specific project resolution providers
//!
//! Each language implements the ProjectResolutionProvider trait to handle
//! project configuration files and path resolution rules.

pub mod typescript;

pub use typescript::TypeScriptProvider;
