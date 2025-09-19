//! Nix language definition and registration
//!
//! This module defines the Nix language support for Codanna, providing
//! Tree-sitter-based parsing and symbol extraction for Nix expressions.
//!
//! ## AST Node Types and Symbol Mappings
//!
//! The Nix parser uses Tree-sitter-nix v0.6.1 and handles the following
//! primary node types and their corresponding symbol classifications:
//!
//! ### Function Definitions
//! - **Named functions** (`binding` with function) → `SymbolKind::Function`
//! - **Lambda expressions** (`function`) → `SymbolKind::Function`
//!
//! ### Variable Bindings
//! - **Let bindings** (`let_binding`) → `SymbolKind::Variable`
//! - **Attribute bindings** (`attr_binding`) → `SymbolKind::Variable`
//!
//! ### Attribute Sets
//! - **Attribute sets** (`attrset`) → `SymbolKind::Object`
//! - **Recursive attribute sets** (`rec_attrset`) → `SymbolKind::Object`
//!
//! ### Lists and Other Constructs
//! - **Lists** (`list`) → `SymbolKind::Array`
//! - **String interpolation** and path literals handled for completeness
//!
//! ## Nix-Specific Language Features
//!
//! The Nix parser handles unique Nix constructs including:
//! - Lazy evaluation semantics
//! - Immutable bindings and functional composition
//! - Attribute inheritance and `with` expressions
//! - String interpolation and path literals
//! - Let-in expression scoping

use crate::parsing::{
    LanguageBehavior, LanguageDefinition, LanguageId, LanguageParser, LanguageRegistry,
};
use crate::{IndexError, IndexResult, Settings};
use std::sync::Arc;

use super::{NixBehavior, NixParser};

/// Nix language definition
///
/// Provides factory methods for creating Nix parsers and behaviors,
/// and defines language metadata like file extensions and identification.
pub struct NixLanguage;

impl LanguageDefinition for NixLanguage {
    fn id(&self) -> LanguageId {
        LanguageId::new("nix")
    }

    fn name(&self) -> &'static str {
        "Nix"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["nix"]
    }

    fn create_parser(&self, _settings: &Settings) -> IndexResult<Box<dyn LanguageParser>> {
        NixParser::new()
            .map(|parser| Box::new(parser) as Box<dyn LanguageParser>)
            .map_err(|e| IndexError::General(format!("Failed to create NixParser: {e}")))
    }

    fn create_behavior(&self) -> Box<dyn LanguageBehavior> {
        Box::new(NixBehavior::new())
    }

    fn default_enabled(&self) -> bool {
        true // Enable Nix by default
    }

    fn is_enabled(&self, settings: &Settings) -> bool {
        settings
            .languages
            .get("Nix")
            .map(|config| config.enabled)
            .unwrap_or(self.default_enabled())
    }
}

/// Register Nix language with the registry
pub(crate) fn register(registry: &mut LanguageRegistry) {
    registry.register(Arc::new(NixLanguage));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nix_language_id() {
        let lang = NixLanguage;
        assert_eq!(lang.id().as_str(), "nix");
    }

    #[test]
    fn test_nix_language_name() {
        let lang = NixLanguage;
        assert_eq!(lang.name(), "Nix");
    }

    #[test]
    fn test_nix_extensions() {
        let lang = NixLanguage;
        assert_eq!(lang.extensions(), &["nix"]);
    }
}
