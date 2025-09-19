pub mod behavior_state;
pub mod context;
pub mod factory;
pub mod go;
pub mod import;
pub mod language;
pub mod language_behavior;
pub mod method_call;
pub mod nix;
pub mod parser;
pub mod php;
pub mod python;
pub mod registry;
pub mod resolution;
pub mod rust;
pub mod typescript;

pub use context::{ParserContext, ScopeType};
pub use factory::{ParserFactory, ParserWithBehavior};
pub use go::{GoBehavior, GoParser};
pub use import::Import;
pub use language::Language;
pub use language_behavior::{LanguageBehavior, LanguageMetadata};
pub use method_call::MethodCall;
pub use nix::{NixBehavior, NixParser};
pub use parser::LanguageParser;
pub use php::{PhpBehavior, PhpParser};
pub use python::{PythonBehavior, PythonParser};
pub use registry::{LanguageDefinition, LanguageId, LanguageRegistry, RegistryError, get_registry};
pub use resolution::{
    GenericInheritanceResolver, GenericResolutionContext, InheritanceResolver, ResolutionScope,
    ScopeLevel,
};
pub use rust::{RustBehavior, RustParser};
pub use typescript::{TypeScriptBehavior, TypeScriptParser};
