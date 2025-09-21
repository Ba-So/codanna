//! Nix behavior implementation placeholder
//!
//! This file is created as part of the module structure.
//! The actual implementation will be done in Task 3.

//! Nix language behavior implementation
//!
//! Provides Nix-specific language behavior for Codanna's code intelligence system.
//! Handles Nix language conventions including functional style, immutable bindings,
//! and attribute-based scoping.

use crate::Visibility;
use crate::parsing::LanguageBehavior;
use std::path::Path;
use tree_sitter::Language;

/// Nix language behavior implementation
///
/// Implements language-specific behavior for Nix including:
/// - Attribute-based module path formatting using '.' separator
/// - Visibility rules for functional language (all symbols public within scope)
/// - Symbol signature formatting appropriate for Nix expressions
/// - Indexing filters for functions, variables, and attribute sets
#[derive(Clone)]
pub struct NixBehavior;

impl NixBehavior {
    /// Create a new Nix behavior instance
    pub fn new() -> Self {
        Self
    }
}

impl Default for NixBehavior {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageBehavior for NixBehavior {
    /// Format module path using Nix attribute access patterns
    ///
    /// Nix uses '.' for attribute access, so module paths follow this convention.
    /// In Nix, symbols are organized in attribute sets, making '.' the natural separator.
    ///
    /// # Examples
    /// - `format_module_path("lib.utils", "helper")` → `"lib.utils.helper"`
    /// - `format_module_path("", "main")` → `"main"`
    fn format_module_path(&self, base_path: &str, symbol_name: &str) -> String {
        if base_path.is_empty() {
            symbol_name.to_string()
        } else {
            format!("{base_path}.{symbol_name}")
        }
    }

    /// Parse visibility from Nix symbol signature
    ///
    /// In Nix's functional programming model, all bindings within a scope are
    /// effectively public to that scope. Nix doesn't have explicit visibility
    /// modifiers like other languages.
    ///
    /// All symbols are treated as public since Nix is a functional language
    /// without traditional visibility concepts.
    fn parse_visibility(&self, _signature: &str) -> Visibility {
        // Nix doesn't have explicit visibility modifiers
        // All bindings are accessible within their scope
        Visibility::Public
    }

    /// Get the module separator for Nix
    ///
    /// Nix uses '.' for attribute access patterns, making it the natural
    /// choice for module path separation in the code intelligence system.
    fn module_separator(&self) -> &'static str {
        "."
    }

    /// Get the tree-sitter Language for Nix
    ///
    /// Uses the tree-sitter-nix language constant for AST metadata access.
    fn get_language(&self) -> Language {
        tree_sitter_nix::LANGUAGE.into()
    }

    /// Convert file path to Nix module path
    ///
    /// Nix files typically represent configuration or build expressions.
    /// The module path is derived from the file path relative to the project root.
    ///
    /// # Examples
    /// - `"lib/utils.nix"` → `"lib.utils"`
    /// - `"default.nix"` → `"default"`
    /// - `"pkgs/development/tools/build.nix"` → `"pkgs.development.tools.build"`
    fn module_path_from_file(&self, file_path: &Path, project_root: &Path) -> Option<String> {
        // Get relative path from project root
        let relative_path = file_path
            .strip_prefix(project_root)
            .ok()
            .or_else(|| file_path.strip_prefix("./").ok())
            .unwrap_or(file_path);

        let path_str = relative_path.to_str()?;

        // Remove .nix extension and convert path separators to dots
        let module_path = path_str
            .trim_start_matches("./")
            .trim_end_matches(".nix")
            .replace(['/', '\\'], ".");

        // Handle special Nix file names
        if module_path.is_empty() {
            Some("default".to_string())
        } else {
            Some(module_path)
        }
    }

    /// Nix doesn't have traits or interfaces
    ///
    /// Nix is a purely functional language without object-oriented concepts
    /// like traits or interfaces.
    fn supports_traits(&self) -> bool {
        false
    }

    /// Nix doesn't have inherent methods
    ///
    /// Nix doesn't have methods in the traditional sense. Functions are
    /// first-class values and there's no concept of methods on types.
    fn supports_inherent_methods(&self) -> bool {
        false
    }

    /// Check if a Nix symbol should be included in the index
    ///
    /// For Nix, we include functions, variables, and attribute sets which
    /// represent the primary constructs in Nix expressions.
    fn is_resolvable_symbol(&self, symbol: &crate::Symbol) -> bool {
        use crate::SymbolKind;
        use crate::symbol::ScopeContext;

        // Check scope_context first if available
        if let Some(ref scope_context) = symbol.scope_context {
            match scope_context {
                ScopeContext::Module | ScopeContext::Global | ScopeContext::Package => true,
                ScopeContext::Local { .. } => {
                    // In Nix, local let-bindings are still resolvable within their scope
                    matches!(symbol.kind, SymbolKind::Function | SymbolKind::Variable)
                }
                ScopeContext::Parameter => false, // Function parameters are not globally resolvable
                ScopeContext::ClassMember => false, // Nix doesn't have classes
            }
        } else {
            // Fallback to symbol kind for Nix-specific symbols
            matches!(
                symbol.kind,
                SymbolKind::Function | SymbolKind::Variable | SymbolKind::Struct
            )
        }
    }

    /// Configure a Nix symbol with language-specific rules
    ///
    /// Applies Nix-specific formatting to symbols including module path
    /// construction and visibility rules.
    fn configure_symbol(&self, symbol: &mut crate::Symbol, module_path: Option<&str>) {
        // Apply Nix module path formatting using '.' separator
        if let Some(path) = module_path {
            let full_path = self.format_module_path(path, &symbol.name);
            symbol.module_path = Some(full_path.into());
        }

        // Apply Nix visibility - all symbols are public within their scope
        symbol.visibility = Visibility::Public;

        // Set default module path for symbols without one
        if symbol.module_path.is_none() {
            symbol.module_path = Some(symbol.name.to_string().into());
        }
    }

    /// Format method call for Nix (attribute access)
    ///
    /// In Nix, function calls and attribute access use different syntax,
    /// but for code intelligence purposes we represent attribute access
    /// using the dot notation.
    fn format_method_call(&self, receiver: &str, method: &str) -> String {
        format!("{receiver}.{method}")
    }

    /// Get inheritance relation name for Nix
    ///
    /// Nix doesn't have inheritance, but we use "references" for attribute
    /// access patterns and function composition.
    fn inheritance_relation_name(&self) -> &'static str {
        "references"
    }

    /// Map Nix-specific relationships to generic RelationKind
    ///
    /// Nix has specific relationship patterns based on functional programming
    /// and attribute sets.
    fn map_relationship(&self, language_specific: &str) -> crate::relationship::RelationKind {
        use crate::relationship::RelationKind;

        match language_specific {
            "references" => RelationKind::References,
            "calls" => RelationKind::Calls,
            "imports" => RelationKind::References, // Nix imports are references
            "with" => RelationKind::References,    // with expressions create references
            _ => RelationKind::References,
        }
    }

    /// Import matching for Nix
    ///
    /// Nix imports are typically relative file paths or attribute paths.
    /// This method handles matching import paths to symbols.
    fn import_matches_symbol(
        &self,
        import_path: &str,
        symbol_module_path: &str,
        importing_module: Option<&str>,
    ) -> bool {
        // Case 1: Exact match
        if import_path == symbol_module_path {
            return true;
        }

        // Case 2: Relative path resolution for Nix
        if let Some(importing_mod) = importing_module {
            // Handle relative imports like "./lib" from "pkgs.development"
            if import_path.starts_with("./") {
                let relative_path = import_path.trim_start_matches("./");
                let resolved = if importing_mod.is_empty() {
                    relative_path.replace('/', ".")
                } else {
                    format!("{}.{}", importing_mod, relative_path.replace('/', "."))
                };

                if resolved == symbol_module_path {
                    return true;
                }
            }
            // Handle parent directory imports like "../shared"
            else if import_path.starts_with("../") {
                let mut module_parts: Vec<String> =
                    importing_mod.split('.').map(|s| s.to_string()).collect();
                let mut path_remaining = import_path;

                // Navigate up for each '../'
                while path_remaining.starts_with("../") {
                    if !module_parts.is_empty() {
                        module_parts.pop();
                    }
                    path_remaining = &path_remaining[3..];
                }

                // Add remaining path
                if !path_remaining.is_empty() {
                    let remaining_path = path_remaining.replace('/', ".");
                    let parts: Vec<String> = remaining_path
                        .split('.')
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                        .collect();
                    module_parts.extend(parts);
                }

                let resolved = module_parts.join(".");
                if resolved == symbol_module_path {
                    return true;
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::registry::LanguageId;
    use crate::{FileId, Range, Symbol, SymbolId, SymbolKind, Visibility};
    use std::path::Path;

    #[test]
    fn test_module_separator() {
        let behavior = NixBehavior::new();
        assert_eq!(behavior.module_separator(), ".");
    }

    #[test]
    fn test_format_module_path() {
        let behavior = NixBehavior::new();

        // Test with base path
        assert_eq!(
            behavior.format_module_path("lib.utils", "helper"),
            "lib.utils.helper"
        );

        // Test with empty base path
        assert_eq!(behavior.format_module_path("", "main"), "main");

        // Test nested paths
        assert_eq!(
            behavior.format_module_path("pkgs.development.tools", "build"),
            "pkgs.development.tools.build"
        );
    }

    #[test]
    fn test_parse_visibility() {
        let behavior = NixBehavior::new();

        // All Nix symbols are public within their scope
        assert_eq!(behavior.parse_visibility("let x = 42;"), Visibility::Public);
        assert_eq!(
            behavior.parse_visibility("func = x: x + 1"),
            Visibility::Public
        );
        assert_eq!(
            behavior.parse_visibility("{ name = \"test\"; }"),
            Visibility::Public
        );
    }

    #[test]
    fn test_module_path_from_file() {
        let behavior = NixBehavior::new();
        let project_root = Path::new("/home/user/project");

        // Test basic Nix file
        let file_path = Path::new("/home/user/project/lib/utils.nix");
        assert_eq!(
            behavior.module_path_from_file(file_path, project_root),
            Some("lib.utils".to_string())
        );

        // Test root level file
        let file_path = Path::new("/home/user/project/default.nix");
        assert_eq!(
            behavior.module_path_from_file(file_path, project_root),
            Some("default".to_string())
        );

        // Test nested package
        let file_path = Path::new("/home/user/project/pkgs/development/tools/build.nix");
        assert_eq!(
            behavior.module_path_from_file(file_path, project_root),
            Some("pkgs.development.tools.build".to_string())
        );

        // Test file without .nix extension (edge case)
        let file_path = Path::new("/home/user/project/flake");
        assert_eq!(
            behavior.module_path_from_file(file_path, project_root),
            Some("flake".to_string())
        );
    }

    #[test]
    fn test_supports_traits() {
        let behavior = NixBehavior::new();
        assert!(!behavior.supports_traits()); // Nix doesn't have traits
    }

    #[test]
    fn test_supports_inherent_methods() {
        let behavior = NixBehavior::new();
        assert!(!behavior.supports_inherent_methods()); // Nix doesn't have methods
    }

    #[test]
    fn test_is_resolvable_symbol() {
        use crate::symbol::ScopeContext;

        let behavior = NixBehavior::new();

        // Test function symbol (should be resolvable)
        let mut function_symbol = create_test_symbol("testFunc", SymbolKind::Function);
        function_symbol.scope_context = Some(ScopeContext::Module);
        assert!(behavior.is_resolvable_symbol(&function_symbol));

        // Test variable symbol (should be resolvable)
        let mut variable_symbol = create_test_symbol("testVar", SymbolKind::Variable);
        variable_symbol.scope_context = Some(ScopeContext::Module);
        assert!(behavior.is_resolvable_symbol(&variable_symbol));

        // Test local variable (should be resolvable in Nix)
        let mut local_symbol = create_test_symbol("localVar", SymbolKind::Variable);
        local_symbol.scope_context = Some(ScopeContext::Local {
            hoisted: false,
            parent_name: None,
            parent_kind: None,
        });
        assert!(behavior.is_resolvable_symbol(&local_symbol));

        // Test parameter symbol (should not be resolvable)
        let mut param_symbol = create_test_symbol("param", SymbolKind::Parameter);
        param_symbol.scope_context = Some(ScopeContext::Parameter);
        assert!(!behavior.is_resolvable_symbol(&param_symbol));

        // Test fallback for symbols without scope_context
        let function_symbol_no_scope = create_test_symbol("func", SymbolKind::Function);
        assert!(behavior.is_resolvable_symbol(&function_symbol_no_scope));

        let variable_symbol_no_scope = create_test_symbol("var", SymbolKind::Variable);
        assert!(behavior.is_resolvable_symbol(&variable_symbol_no_scope));

        let method_symbol = create_test_symbol("method", SymbolKind::Method);
        assert!(!behavior.is_resolvable_symbol(&method_symbol)); // Not in fallback list
    }

    #[test]
    fn test_configure_symbol() {
        let behavior = NixBehavior::new();

        let mut symbol = create_test_symbol("testSymbol", SymbolKind::Function);
        symbol.signature = Some("testFunc = x: x + 1".into());
        symbol.visibility = Visibility::Private; // Will be updated by configure_symbol

        behavior.configure_symbol(&mut symbol, Some("lib.utils"));

        assert_eq!(
            symbol.module_path.as_ref().map(|s| s.as_ref()),
            Some("lib.utils.testSymbol")
        );
        assert_eq!(symbol.visibility, Visibility::Public); // Should be public in Nix

        // Test symbol without module path
        let mut symbol_no_path = create_test_symbol("standalone", SymbolKind::Variable);
        behavior.configure_symbol(&mut symbol_no_path, None);

        assert_eq!(
            symbol_no_path.module_path.as_ref().map(|s| s.as_ref()),
            Some("standalone")
        );
        assert_eq!(symbol_no_path.visibility, Visibility::Public);
    }

    #[test]
    fn test_format_method_call() {
        let behavior = NixBehavior::new();
        assert_eq!(
            behavior.format_method_call("config", "packages"),
            "config.packages"
        );
        assert_eq!(behavior.format_method_call("nixpkgs", "lib"), "nixpkgs.lib");
    }

    #[test]
    fn test_inheritance_relation_name() {
        let behavior = NixBehavior::new();
        assert_eq!(behavior.inheritance_relation_name(), "references");
    }

    #[test]
    fn test_map_relationship() {
        use crate::relationship::RelationKind;

        let behavior = NixBehavior::new();

        assert_eq!(
            behavior.map_relationship("references"),
            RelationKind::References
        );
        assert_eq!(behavior.map_relationship("calls"), RelationKind::Calls);
        assert_eq!(
            behavior.map_relationship("imports"),
            RelationKind::References
        );
        assert_eq!(behavior.map_relationship("with"), RelationKind::References);
        assert_eq!(
            behavior.map_relationship("unknown"),
            RelationKind::References
        ); // Default
    }

    #[test]
    fn test_import_matches_symbol() {
        let behavior = NixBehavior::new();

        // Test exact matches
        assert!(behavior.import_matches_symbol("lib.utils", "lib.utils", None));
        assert!(behavior.import_matches_symbol("nixpkgs", "nixpkgs", None));

        // Test relative imports
        assert!(behavior.import_matches_symbol("./utils", "lib.utils", Some("lib")));
        assert!(behavior.import_matches_symbol("./helpers", "pkgs.helpers", Some("pkgs")));

        // Test parent directory imports
        assert!(behavior.import_matches_symbol("../shared", "lib.shared", Some("lib.internal")));
        assert!(behavior.import_matches_symbol(
            "../common",
            "pkgs.common",
            Some("pkgs.development")
        ));

        // Test complex relative paths
        assert!(behavior.import_matches_symbol("./sub/module", "base.sub.module", Some("base")));

        // Test non-matches
        assert!(!behavior.import_matches_symbol("lib.utils", "lib.other", None));
        assert!(!behavior.import_matches_symbol("./utils", "lib.other", Some("lib")));
    }

    #[test]
    fn test_get_language() {
        let behavior = NixBehavior::new();
        let language = behavior.get_language();

        // Just verify we can get the language without panicking
        // The actual language object is from tree-sitter-nix
        assert!(language.node_kind_count() > 0);
    }

    // Helper function to create test symbols
    fn create_test_symbol(name: &str, kind: SymbolKind) -> Symbol {
        Symbol {
            id: SymbolId::new(1).unwrap(),
            name: name.into(),
            kind,
            signature: None,
            module_path: None,
            file_id: FileId::new(1).unwrap(),
            range: Range {
                start_line: 1,
                start_column: 1,
                end_line: 1,
                end_column: 10,
            },
            doc_comment: None,
            visibility: Visibility::Private,
            scope_context: None,
            language_id: Some(LanguageId::new("nix")),
        }
    }
}
