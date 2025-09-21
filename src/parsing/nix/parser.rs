//! Nix parser implementation using tree-sitter-nix
//!
//! This module provides the core Nix language parsing functionality,
//! extracting symbols from Nix expressions including functions, variables,
//! and attribute sets.

use super::resolution::NixResolutionContext;
use crate::parsing::{
    LanguageParser, MethodCall, ParserContext, ScopeLevel, resolution::ResolutionScope,
};
use crate::types::SymbolCounter;
use crate::{FileId, Range, Symbol, SymbolKind, Visibility};
use std::any::Any;
use tree_sitter::{Node, Parser, Tree};

/// Nix language parser using tree-sitter-nix
pub struct NixParser {
    parser: Parser,
    context: ParserContext,
    /// Nix-specific resolution context for advanced scoping
    resolution_context: Option<NixResolutionContext>,
}

impl NixParser {
    /// Create a new Nix parser
    pub fn new() -> Result<Self, String> {
        let mut parser = Parser::new();
        let lang = tree_sitter_nix::LANGUAGE;
        parser
            .set_language(&lang.into())
            .map_err(|e| format!("Failed to set Nix language: {e}"))?;

        Ok(Self {
            parser,
            context: ParserContext::new(),
            resolution_context: None,
        })
    }

    /// Helper to create a symbol with basic fields
    fn create_symbol(
        &self,
        id: crate::types::SymbolId,
        name: String,
        kind: SymbolKind,
        file_id: FileId,
        range: Range,
        signature: Option<String>,
        doc_comment: Option<String>,
    ) -> Symbol {
        let mut symbol = Symbol::new(id, name, kind, file_id, range);

        if let Some(sig) = signature {
            symbol = symbol.with_signature(sig);
        }
        if let Some(doc) = doc_comment {
            symbol = symbol.with_doc(doc);
        }

        // Nix symbols are generally publicly visible within their scope
        symbol = symbol.with_visibility(Visibility::Public);

        // Set scope context
        symbol.scope_context = Some(self.context.current_scope_context());

        symbol
    }

    /// Extract symbols from a Nix AST node recursively
    fn extract_symbols_from_node(
        &mut self,
        node: Node,
        code: &str,
        file_id: FileId,
        counter: &mut SymbolCounter,
        symbols: &mut Vec<Symbol>,
    ) {
        match node.kind() {
            // Handle let-in expressions: let a = 1; b = 2; in expression
            "let_expression" => {
                self.process_let_expression_advanced(node, code, file_id, counter, symbols);
            }
            // Handle attribute sets: { name = value; }
            "attrset" => {
                self.process_attribute_set(node, code, file_id, counter, symbols);
            }
            // Handle recursive attribute sets: rec { a = 1; b = a + 1; }
            "rec_attrset" => {
                self.process_recursive_attribute_set_advanced(
                    node, code, file_id, counter, symbols,
                );
            }
            // Handle function definitions: arg: body or { arg1, arg2 }: body
            "function" | "function_expression" => {
                self.process_lambda_function(node, code, file_id, counter, symbols);
            }
            // Handle bindings (assignments): name = value
            "binding" => {
                self.process_binding(node, code, file_id, counter, symbols);
            }
            // Handle with expressions: with attr-set; expression
            "with_expression" => {
                self.process_with_expression(node, code, file_id, counter, symbols);
            }
            // Handle string interpolation: "text ${expr} more text"
            "indented_string_expression" | "string_expression" => {
                self.process_string_interpolation(node, code, file_id, counter, symbols);
            }
            // Handle path literals: ./path/to/file
            "path_expression" => {
                self.process_path_literal(node, code, file_id, counter, symbols);
            }
            _ => {
                // Recursively process child nodes for other node types
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.extract_symbols_from_node(child, code, file_id, counter, symbols);
                }
            }
        }
    }

    #[allow(dead_code)]
    /// Process let-in expression and extract variable bindings
    fn process_let_expression(
        &mut self,
        node: Node,
        code: &str,
        file_id: FileId,
        counter: &mut SymbolCounter,
        symbols: &mut Vec<Symbol>,
    ) {
        // Find bindings within the let expression
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "binding" {
                self.process_binding(child, code, file_id, counter, symbols);
            }
        }

        // Continue processing other child nodes
        for child in node.children(&mut cursor) {
            if child.kind() != "binding" {
                self.extract_symbols_from_node(child, code, file_id, counter, symbols);
            }
        }
    }

    /// Process binding (name = value)
    fn process_binding(
        &mut self,
        node: Node,
        code: &str,
        file_id: FileId,
        counter: &mut SymbolCounter,
        symbols: &mut Vec<Symbol>,
    ) {
        // Get the identifier (left side of =) - need to navigate through attrpath
        let identifier_node = if let Some(attrpath) = node.child_by_field_name("attrpath") {
            // First child of attrpath should be the identifier
            attrpath.child(0)
        } else {
            // Fallback: try direct name field
            node.child_by_field_name("name")
        };

        if let Some(identifier_node) = identifier_node {
            let name = code[identifier_node.byte_range()].to_string();
            let ts_range = identifier_node.range();
            let range = Range::new(
                ts_range.start_point.row as u32,
                ts_range.start_point.column as u16,
                ts_range.end_point.row as u32,
                ts_range.end_point.column as u16,
            );

            // Determine if this is a function binding by looking at the value
            let symbol_kind = if let Some(value_node) = node.child_by_field_name("expression") {
                if value_node.kind() == "function_expression" || value_node.kind() == "function" {
                    SymbolKind::Function
                } else {
                    SymbolKind::Variable
                }
            } else {
                SymbolKind::Variable
            };

            // Create signature for functions
            let signature = if symbol_kind == SymbolKind::Function {
                Some(format!("{name} = <function>"))
            } else {
                Some(format!("{name} = <value>"))
            };

            // Look for documentation comment (preceding comment)
            let doc_comment = self.extract_doc_comment(&node, code);

            let symbol = self.create_symbol(
                counter.next_id(),
                name,
                symbol_kind,
                file_id,
                range,
                signature,
                doc_comment,
            );

            symbols.push(symbol);
        }

        // Recursively process the value expression
        if let Some(value_node) = node.child_by_field_name("expression") {
            self.extract_symbols_from_node(value_node, code, file_id, counter, symbols);
        }
    }

    /// Process attribute set: { name = value; }
    fn process_attribute_set(
        &mut self,
        node: Node,
        code: &str,
        file_id: FileId,
        counter: &mut SymbolCounter,
        symbols: &mut Vec<Symbol>,
    ) {
        // Process each binding within the attribute set
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "binding" {
                self.process_binding(child, code, file_id, counter, symbols);
            } else {
                self.extract_symbols_from_node(child, code, file_id, counter, symbols);
            }
        }
    }

    #[allow(dead_code)]
    /// Process recursive attribute set: rec { a = 1; b = a + 1; }
    fn process_recursive_attribute_set(
        &mut self,
        node: Node,
        code: &str,
        file_id: FileId,
        counter: &mut SymbolCounter,
        symbols: &mut Vec<Symbol>,
    ) {
        // Similar to regular attribute set but allows self-references
        self.process_attribute_set(node, code, file_id, counter, symbols);
    }

    #[allow(dead_code)]
    /// Process function definition
    fn process_function(
        &mut self,
        node: Node,
        code: &str,
        file_id: FileId,
        counter: &mut SymbolCounter,
        symbols: &mut Vec<Symbol>,
    ) {
        // For anonymous functions, we don't create a symbol entry
        // but we still need to process the function body
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_symbols_from_node(child, code, file_id, counter, symbols);
        }
    }

    /// Walk the entire tree and extract all symbols
    fn walk_tree(
        &mut self,
        tree: Tree,
        code: &str,
        file_id: FileId,
        counter: &mut SymbolCounter,
    ) -> Vec<Symbol> {
        let mut symbols = Vec::new();
        let root_node = tree.root_node();
        self.extract_symbols_from_node(root_node, code, file_id, counter, &mut symbols);
        symbols
    }

    /// Process with expression: with attr-set; expression
    /// Brings attributes from attr-set into scope for the expression
    fn process_with_expression(
        &mut self,
        node: Node,
        code: &str,
        file_id: FileId,
        counter: &mut SymbolCounter,
        symbols: &mut Vec<Symbol>,
    ) {
        // Enter with scope in resolution context
        if let Some(ref mut ctx) = self.resolution_context {
            ctx.enter_with_scope(std::collections::HashMap::new());
        }

        // Process the with expression - typically has 'expression' field for the body
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_symbols_from_node(child, code, file_id, counter, symbols);
        }

        // Exit with scope
        if let Some(ref mut ctx) = self.resolution_context {
            ctx.exit_with_scope();
        }
    }

    /// Process string interpolation: "text ${expr} more text"
    /// Extract symbols from interpolated expressions
    fn process_string_interpolation(
        &mut self,
        node: Node,
        code: &str,
        file_id: FileId,
        counter: &mut SymbolCounter,
        symbols: &mut Vec<Symbol>,
    ) {
        // Find interpolation expressions within the string
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "interpolation" {
                // Process the expression inside ${}
                self.extract_symbols_from_node(child, code, file_id, counter, symbols);
            }
        }
    }

    /// Process lambda function: param: body or { param1, param2 }: body
    /// Extract function parameters and process body with proper scoping
    fn process_lambda_function(
        &mut self,
        node: Node,
        code: &str,
        file_id: FileId,
        counter: &mut SymbolCounter,
        symbols: &mut Vec<Symbol>,
    ) {
        // Extract parameters
        let mut parameters = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            match child.kind() {
                "identifier" => {
                    // Simple parameter: param: body
                    let param_name = code[child.byte_range()].to_string();
                    let param_id = counter.next_id();
                    parameters.push((param_name, param_id));
                }
                "formals" => {
                    // Pattern parameters: { param1, param2 }: body
                    self.extract_formals_parameters(child, code, counter, &mut parameters);
                }
                _ => {}
            }
        }

        // Enter function scope with parameters
        if let Some(ref mut ctx) = self.resolution_context {
            ctx.enter_function_scope(parameters);
        }

        // Process function body
        for child in node.children(&mut cursor) {
            if child.kind() != "identifier" && child.kind() != "formals" {
                self.extract_symbols_from_node(child, code, file_id, counter, symbols);
            }
        }

        // Exit function scope
        if let Some(ref mut ctx) = self.resolution_context {
            ctx.exit_function_scope();
        }
    }

    /// Extract parameters from function formals: { param1, param2, ... }
    fn extract_formals_parameters(
        &self,
        node: Node,
        code: &str,
        counter: &mut SymbolCounter,
        parameters: &mut Vec<(String, crate::types::SymbolId)>,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "formal" {
                // Each formal parameter
                if let Some(identifier) = child.child_by_field_name("name") {
                    let param_name = code[identifier.byte_range()].to_string();
                    let param_id = counter.next_id();
                    parameters.push((param_name, param_id));
                }
            }
        }
    }

    /// Process path literal: ./path/to/file or /absolute/path
    /// These are Nix-specific constructs for file references
    fn process_path_literal(
        &mut self,
        node: Node,
        code: &str,
        file_id: FileId,
        counter: &mut SymbolCounter,
        symbols: &mut Vec<Symbol>,
    ) {
        let path_str = code[node.byte_range()].to_string();
        let ts_range = node.range();
        let range = Range::new(
            ts_range.start_point.row as u32,
            ts_range.start_point.column as u16,
            ts_range.end_point.row as u32,
            ts_range.end_point.column as u16,
        );

        // Create a constant symbol for the path literal
        let symbol = self.create_symbol(
            counter.next_id(),
            format!("path_{}", symbols.len()), // Generate unique name for path
            SymbolKind::Constant,
            file_id,
            range,
            Some(format!("path = {path_str}")),
            None,
        );

        symbols.push(symbol);
    }

    /// Enhanced recursive attribute set processing with forward references
    fn process_recursive_attribute_set_advanced(
        &mut self,
        node: Node,
        code: &str,
        file_id: FileId,
        counter: &mut SymbolCounter,
        symbols: &mut Vec<Symbol>,
    ) {
        // Enter recursive scope
        if let Some(ref mut ctx) = self.resolution_context {
            ctx.enter_attrset_scope(true);
        }

        // First pass: collect all attribute names for forward references
        let mut attr_symbols = Vec::new();
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            if child.kind() == "binding" {
                if let Some(attrpath) = child.child_by_field_name("attrpath") {
                    if let Some(identifier_node) = attrpath.child(0) {
                        let name = code[identifier_node.byte_range()].to_string();
                        let symbol_id = counter.next_id();

                        // Add to resolution context for forward references
                        if let Some(ref mut ctx) = self.resolution_context {
                            ctx.add_recursive_symbol(name.clone(), symbol_id);
                        }

                        attr_symbols.push((name, symbol_id, child));
                    }
                }
            }
        }

        // Second pass: process all bindings with forward references available
        for (name, symbol_id, binding_node) in attr_symbols {
            let ts_range = binding_node.range();
            let range = Range::new(
                ts_range.start_point.row as u32,
                ts_range.start_point.column as u16,
                ts_range.end_point.row as u32,
                ts_range.end_point.column as u16,
            );

            // Determine symbol kind by checking the value
            let symbol_kind = if let Some(value_node) =
                binding_node.child_by_field_name("expression")
            {
                if value_node.kind() == "function_expression" || value_node.kind() == "function" {
                    SymbolKind::Function
                } else {
                    SymbolKind::Variable
                }
            } else {
                SymbolKind::Variable
            };

            let signature = if symbol_kind == SymbolKind::Function {
                Some(format!("{name} = <function>"))
            } else {
                Some(format!("{name} = <value>"))
            };

            let symbol = self.create_symbol(
                symbol_id,
                name,
                symbol_kind,
                file_id,
                range,
                signature,
                None,
            );

            symbols.push(symbol);

            // Process the value expression
            if let Some(value_node) = binding_node.child_by_field_name("expression") {
                self.extract_symbols_from_node(value_node, code, file_id, counter, symbols);
            }
        }

        // Exit recursive scope
        if let Some(ref mut ctx) = self.resolution_context {
            ctx.exit_attrset_scope();
        }
    }

    /// Enhanced let-in expression processing with proper scoping
    fn process_let_expression_advanced(
        &mut self,
        node: Node,
        code: &str,
        file_id: FileId,
        counter: &mut SymbolCounter,
        symbols: &mut Vec<Symbol>,
    ) {
        // Enter let scope
        if let Some(ref mut ctx) = self.resolution_context {
            ctx.enter_let_scope();
        }

        // Process let bindings first
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "binding" {
                self.process_binding(child, code, file_id, counter, symbols);

                // Add binding to let context for the 'in' expression
                if let Some(attrpath) = child.child_by_field_name("attrpath") {
                    if let Some(identifier_node) = attrpath.child(0) {
                        let name = code[identifier_node.byte_range()].to_string();
                        if let Some(symbol) = symbols.last() {
                            if let Some(ref mut ctx) = self.resolution_context {
                                ctx.add_symbol(name, symbol.id, ScopeLevel::Local);
                            }
                        }
                    }
                }
            }
        }

        // Process the 'in' expression with bindings available
        for child in node.children(&mut cursor) {
            if child.kind() != "binding" && child.kind() != "let" {
                self.extract_symbols_from_node(child, code, file_id, counter, symbols);
            }
        }

        // Exit let scope
        if let Some(ref mut ctx) = self.resolution_context {
            ctx.exit_let_scope();
        }
    }
}

impl LanguageParser for NixParser {
    /// Parse Nix source code and extract symbols
    fn parse(
        &mut self,
        code: &str,
        file_id: FileId,
        symbol_counter: &mut SymbolCounter,
    ) -> Vec<Symbol> {
        // Reset context for each file
        self.context = ParserContext::new();
        // Initialize resolution context for advanced scoping
        self.resolution_context = Some(NixResolutionContext::new(file_id));

        match self.parser.parse(code, None) {
            Some(tree) => {
                if tree.root_node().has_error() {
                    // Log parsing errors but continue with partial results
                    eprintln!("Nix parsing errors detected in file {}", file_id.0);
                }
                self.walk_tree(tree, code, file_id, symbol_counter)
            }
            None => {
                eprintln!("Failed to parse Nix file {}", file_id.0);
                Vec::new()
            }
        }
    }

    /// Enable downcasting to NixParser
    fn as_any(&self) -> &dyn Any {
        self
    }

    /// Extract documentation comment for Nix (typically # comments)
    fn extract_doc_comment(&self, node: &Node, code: &str) -> Option<String> {
        // Look for preceding comment lines that start with #
        let start_line = node.start_position().row;

        if start_line == 0 {
            return None;
        }

        let lines: Vec<&str> = code.lines().collect();
        let mut doc_lines = Vec::new();

        // Look backwards for consecutive comment lines
        for i in (0..start_line).rev() {
            let line = lines.get(i)?.trim();
            if line.starts_with('#') {
                // Remove # and trim whitespace
                let comment_text = line.trim_start_matches('#').trim();
                doc_lines.insert(0, comment_text.to_string());
            } else if line.is_empty() {
                // Empty lines are okay, continue looking
                continue;
            } else {
                // Non-comment, non-empty line - stop looking
                break;
            }
        }

        if doc_lines.is_empty() {
            None
        } else {
            Some(doc_lines.join(" "))
        }
    }

    /// Find function/method calls in Nix code
    fn find_calls<'a>(&mut self, _code: &'a str) -> Vec<(&'a str, &'a str, Range)> {
        // TODO: Implement call detection for Nix
        // This is a basic implementation - Nix function calls are more complex
        Vec::new()
    }

    /// Find method calls with receiver information
    fn find_method_calls(&mut self, _code: &str) -> Vec<MethodCall> {
        // Nix doesn't have traditional method calls like OOP languages
        // Function application is the primary mechanism
        Vec::new()
    }

    /// Find trait/interface implementations (not applicable to Nix)
    fn find_implementations<'a>(&mut self, _code: &'a str) -> Vec<(&'a str, &'a str, Range)> {
        // Nix doesn't have traits or interfaces
        Vec::new()
    }

    /// Find type usage (not applicable to Nix)
    fn find_uses<'a>(&mut self, _code: &'a str) -> Vec<(&'a str, &'a str, Range)> {
        // Nix is dynamically typed - no explicit type usage
        Vec::new()
    }

    /// Find method definitions (not applicable to Nix)
    fn find_defines<'a>(&mut self, _code: &'a str) -> Vec<(&'a str, &'a str, Range)> {
        // Nix doesn't have traditional method definitions
        Vec::new()
    }

    /// Find import statements in Nix code
    fn find_imports(&mut self, _code: &str, _file_id: FileId) -> Vec<crate::parsing::Import> {
        // TODO: Implement import detection for Nix (import statements, with expressions)
        Vec::new()
    }

    /// Get the language this parser handles
    fn language(&self) -> crate::parsing::Language {
        crate::parsing::Language::Nix
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FileId;

    #[test]
    fn test_nix_parser_creation() {
        let parser = NixParser::new();
        assert!(
            parser.is_ok(),
            "Failed to create NixParser: {:?}",
            parser.err()
        );
    }

    #[test]
    fn test_basic_nix_parsing() {
        let mut parser = NixParser::new().expect("Failed to create NixParser");
        let mut counter = SymbolCounter::new();
        let file_id = FileId(1);

        let code = r#"
# Variable binding
let x = 42; in x
"#;

        let symbols = parser.parse(code, file_id, &mut counter);

        // Should extract the variable binding 'x'
        assert!(!symbols.is_empty(), "Should extract at least one symbol");

        // Check if we found the variable x
        let x_symbol = symbols.iter().find(|s| s.name.as_ref() == "x");
        assert!(x_symbol.is_some(), "Should find variable 'x'");

        let x_symbol = x_symbol.unwrap();
        assert_eq!(
            x_symbol.kind,
            SymbolKind::Variable,
            "x should be a variable"
        );
    }

    #[test]
    fn test_function_binding_parsing() {
        let mut parser = NixParser::new().expect("Failed to create NixParser");
        let mut counter = SymbolCounter::new();
        let file_id = FileId(1);

        let code = r#"
let double = n: n * 2; in double 5
"#;

        let symbols = parser.parse(code, file_id, &mut counter);

        // Should extract the function binding 'double'
        let double_symbol = symbols.iter().find(|s| s.name.as_ref() == "double");
        assert!(double_symbol.is_some(), "Should find function 'double'");

        let double_symbol = double_symbol.unwrap();
        assert_eq!(
            double_symbol.kind,
            SymbolKind::Function,
            "double should be a function"
        );
    }

    #[test]
    fn test_attribute_set_parsing() {
        let mut parser = NixParser::new().expect("Failed to create NixParser");
        let mut counter = SymbolCounter::new();
        let file_id = FileId(1);

        let code = r#"
{
  name = "test";
  value = 42;
}
"#;

        let symbols = parser.parse(code, file_id, &mut counter);

        // Should extract the attribute bindings
        let name_symbol = symbols.iter().find(|s| s.name.as_ref() == "name");
        let value_symbol = symbols.iter().find(|s| s.name.as_ref() == "value");

        assert!(name_symbol.is_some(), "Should find attribute 'name'");
        assert!(value_symbol.is_some(), "Should find attribute 'value'");

        assert_eq!(
            name_symbol.unwrap().kind,
            SymbolKind::Variable,
            "name should be a variable"
        );
        assert_eq!(
            value_symbol.unwrap().kind,
            SymbolKind::Variable,
            "value should be a variable"
        );
    }

    #[test]
    fn test_recursive_attribute_set_parsing() {
        let mut parser = NixParser::new().expect("Failed to create NixParser");
        let mut counter = SymbolCounter::new();
        let file_id = FileId(1);

        let code = r#"
rec {
  a = 1;
  b = a + 2;
  c = b * 3;
}
"#;

        let symbols = parser.parse(code, file_id, &mut counter);

        // Should extract all recursive bindings
        let a_symbol = symbols.iter().find(|s| s.name.as_ref() == "a");
        let b_symbol = symbols.iter().find(|s| s.name.as_ref() == "b");
        let c_symbol = symbols.iter().find(|s| s.name.as_ref() == "c");

        assert!(a_symbol.is_some(), "Should find attribute 'a'");
        assert!(b_symbol.is_some(), "Should find attribute 'b'");
        assert!(c_symbol.is_some(), "Should find attribute 'c'");

        assert_eq!(
            a_symbol.unwrap().kind,
            SymbolKind::Variable,
            "a should be a variable"
        );
        assert_eq!(
            b_symbol.unwrap().kind,
            SymbolKind::Variable,
            "b should be a variable"
        );
        assert_eq!(
            c_symbol.unwrap().kind,
            SymbolKind::Variable,
            "c should be a variable"
        );
    }

    #[test]
    fn test_with_expression_parsing() {
        let mut parser = NixParser::new().expect("Failed to create NixParser");
        let mut counter = SymbolCounter::new();
        let file_id = FileId(1);

        let code = r#"
let pkgs = { a = 1; b = 2; };
in with pkgs; a + b
"#;

        let symbols = parser.parse(code, file_id, &mut counter);

        // Should extract the pkgs binding
        let pkgs_symbol = symbols.iter().find(|s| s.name.as_ref() == "pkgs");
        assert!(pkgs_symbol.is_some(), "Should find variable 'pkgs'");
        assert_eq!(
            pkgs_symbol.unwrap().kind,
            SymbolKind::Variable,
            "pkgs should be a variable"
        );
    }

    #[test]
    fn test_complex_function_parsing() {
        let mut parser = NixParser::new().expect("Failed to create NixParser");
        let mut counter = SymbolCounter::new();
        let file_id = FileId(1);

        let code = r#"
let
  # Simple function
  add = a: b: a + b;
  
  # Pattern matching function
  processConfig = { name, version ? "1.0", ... }: {
    inherit name version;
  };
  
  # Nested let-in with function
  buildPackage = name: let
    version = "2.0";
  in { inherit name version; };
in {
  inherit add processConfig buildPackage;
}
"#;

        let symbols = parser.parse(code, file_id, &mut counter);

        // Should extract function bindings
        let add_symbol = symbols.iter().find(|s| s.name.as_ref() == "add");
        let process_config_symbol = symbols.iter().find(|s| s.name.as_ref() == "processConfig");
        let build_package_symbol = symbols.iter().find(|s| s.name.as_ref() == "buildPackage");

        assert!(add_symbol.is_some(), "Should find function 'add'");
        assert!(
            process_config_symbol.is_some(),
            "Should find function 'processConfig'"
        );
        assert!(
            build_package_symbol.is_some(),
            "Should find function 'buildPackage'"
        );

        assert_eq!(
            add_symbol.unwrap().kind,
            SymbolKind::Function,
            "add should be a function"
        );
        assert_eq!(
            process_config_symbol.unwrap().kind,
            SymbolKind::Function,
            "processConfig should be a function"
        );
        assert_eq!(
            build_package_symbol.unwrap().kind,
            SymbolKind::Function,
            "buildPackage should be a function"
        );
    }

    #[test]
    fn test_string_interpolation_parsing() {
        let mut parser = NixParser::new().expect("Failed to create NixParser");
        let mut counter = SymbolCounter::new();
        let file_id = FileId(1);

        let code = r#"
let
  name = "world";
  greeting = "Hello ${name}!";
  complex = "The value is ${toString (42 + 8)}";
in { inherit name greeting complex; }
"#;

        let symbols = parser.parse(code, file_id, &mut counter);

        // Should extract variable bindings
        let name_symbol = symbols.iter().find(|s| s.name.as_ref() == "name");
        let greeting_symbol = symbols.iter().find(|s| s.name.as_ref() == "greeting");
        let complex_symbol = symbols.iter().find(|s| s.name.as_ref() == "complex");

        assert!(name_symbol.is_some(), "Should find variable 'name'");
        assert!(greeting_symbol.is_some(), "Should find variable 'greeting'");
        assert!(complex_symbol.is_some(), "Should find variable 'complex'");

        assert_eq!(
            name_symbol.unwrap().kind,
            SymbolKind::Variable,
            "name should be a variable"
        );
        assert_eq!(
            greeting_symbol.unwrap().kind,
            SymbolKind::Variable,
            "greeting should be a variable"
        );
        assert_eq!(
            complex_symbol.unwrap().kind,
            SymbolKind::Variable,
            "complex should be a variable"
        );
    }

    #[test]
    fn test_path_literal_parsing() {
        let mut parser = NixParser::new().expect("Failed to create NixParser");
        let mut counter = SymbolCounter::new();
        let file_id = FileId(1);

        let code = r#"
let
  relativePath = ./config/default.nix;
  absolutePath = /etc/nixos/configuration.nix;
in { inherit relativePath absolutePath; }
"#;

        let symbols = parser.parse(code, file_id, &mut counter);

        // Should extract path variable bindings and path literal constants
        let relative_symbol = symbols.iter().find(|s| s.name.as_ref() == "relativePath");
        let absolute_symbol = symbols.iter().find(|s| s.name.as_ref() == "absolutePath");

        assert!(
            relative_symbol.is_some(),
            "Should find variable 'relativePath'"
        );
        assert!(
            absolute_symbol.is_some(),
            "Should find variable 'absolutePath'"
        );

        assert_eq!(
            relative_symbol.unwrap().kind,
            SymbolKind::Variable,
            "relativePath should be a variable"
        );
        assert_eq!(
            absolute_symbol.unwrap().kind,
            SymbolKind::Variable,
            "absolutePath should be a variable"
        );

        // Should also extract path literal constants
        let path_constants: Vec<_> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Constant && s.name.starts_with("path_"))
            .collect();
        assert!(
            !path_constants.is_empty(),
            "Should extract path literal constants"
        );
    }

    #[test]
    fn test_nested_scoping() {
        let mut parser = NixParser::new().expect("Failed to create NixParser");
        let mut counter = SymbolCounter::new();
        let file_id = FileId(1);

        let code = r#"
let
  outer = "outer";
  func = arg: let
    inner = "inner";
    nested = arg + inner + outer;
  in nested;
in func "test"
"#;

        let symbols = parser.parse(code, file_id, &mut counter);

        // Should extract all bindings at their appropriate scopes
        let outer_symbol = symbols.iter().find(|s| s.name.as_ref() == "outer");
        let func_symbol = symbols.iter().find(|s| s.name.as_ref() == "func");
        let inner_symbol = symbols.iter().find(|s| s.name.as_ref() == "inner");
        let nested_symbol = symbols.iter().find(|s| s.name.as_ref() == "nested");

        assert!(outer_symbol.is_some(), "Should find variable 'outer'");
        assert!(func_symbol.is_some(), "Should find function 'func'");
        assert!(inner_symbol.is_some(), "Should find variable 'inner'");
        assert!(nested_symbol.is_some(), "Should find variable 'nested'");

        assert_eq!(
            outer_symbol.unwrap().kind,
            SymbolKind::Variable,
            "outer should be a variable"
        );
        assert_eq!(
            func_symbol.unwrap().kind,
            SymbolKind::Function,
            "func should be a function"
        );
        assert_eq!(
            inner_symbol.unwrap().kind,
            SymbolKind::Variable,
            "inner should be a variable"
        );
        assert_eq!(
            nested_symbol.unwrap().kind,
            SymbolKind::Variable,
            "nested should be a variable"
        );
    }

    #[test]
    fn test_doc_comment_extraction() {
        let mut parser = NixParser::new().expect("Failed to create NixParser");
        let mut counter = SymbolCounter::new();
        let file_id = FileId(1);

        let code = r#"
let
  # This is a documented variable
  # It has multiple lines of documentation
  documented = "value";
  
  # This function adds two numbers
  add = a: b: a + b;
in { inherit documented add; }
"#;

        let symbols = parser.parse(code, file_id, &mut counter);

        let documented_symbol = symbols.iter().find(|s| s.name.as_ref() == "documented");
        let add_symbol = symbols.iter().find(|s| s.name.as_ref() == "add");

        assert!(
            documented_symbol.is_some(),
            "Should find documented variable"
        );
        assert!(add_symbol.is_some(), "Should find add function");

        // Check documentation was extracted
        let doc_symbol = documented_symbol.unwrap();
        assert!(
            doc_symbol.doc_comment.is_some(),
            "Should have documentation"
        );
        let doc_text = doc_symbol.doc_comment.as_ref().unwrap();
        assert!(
            doc_text.contains("documented variable"),
            "Should contain doc text"
        );

        let add_doc_symbol = add_symbol.unwrap();
        assert!(
            add_doc_symbol.doc_comment.is_some(),
            "Should have documentation for add"
        );
        let add_doc_text = add_doc_symbol.doc_comment.as_ref().unwrap();
        assert!(
            add_doc_text.contains("adds two numbers"),
            "Should contain function doc text"
        );
    }
}
