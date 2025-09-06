//! TypeScript parser audit module
//!
//! Tracks which AST nodes the parser actually handles vs what's available in the grammar.
//! This helps identify gaps in our symbol extraction.

use super::TypeScriptParser;
use crate::types::FileId;
use std::collections::{HashMap, HashSet};
use thiserror::Error;
use tree_sitter::{Node, Parser};

#[derive(Error, Debug)]
pub enum AuditError {
    #[error("Failed to read file: {0}")]
    FileRead(#[from] std::io::Error),

    #[error("Failed to set language: {0}")]
    LanguageSetup(String),

    #[error("Failed to parse code")]
    ParseFailure,

    #[error("Failed to create parser: {0}")]
    ParserCreation(String),
}

pub struct TypeScriptParserAudit {
    /// Nodes found in the grammar/file
    pub grammar_nodes: HashMap<String, u16>,
    /// Nodes our parser actually processes (from tracking parse calls)
    pub implemented_nodes: HashSet<String>,
    /// Symbols actually extracted
    pub extracted_symbol_kinds: HashSet<String>,
}

impl TypeScriptParserAudit {
    /// Run audit on a TypeScript source file
    pub fn audit_file(file_path: &str) -> Result<Self, AuditError> {
        let code = std::fs::read_to_string(file_path)?;
        Self::audit_code(&code)
    }

    /// Run audit on TypeScript source code  
    pub fn audit_code(code: &str) -> Result<Self, AuditError> {
        // First, discover all nodes in the file using tree-sitter directly
        let mut parser = Parser::new();
        let language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into();
        parser
            .set_language(&language)
            .map_err(|e| AuditError::LanguageSetup(e.to_string()))?;

        let tree = parser.parse(code, None).ok_or(AuditError::ParseFailure)?;

        let mut grammar_nodes = HashMap::new();
        discover_nodes(tree.root_node(), &mut grammar_nodes);

        // Now parse with our actual parser to see what symbols get extracted
        let mut ts_parser =
            TypeScriptParser::new().map_err(|e| AuditError::ParserCreation(e.to_string()))?;
        let file_id = FileId(1);
        let mut symbol_counter = crate::types::SymbolCounter::new();
        let symbols = ts_parser.parse(code, file_id, &mut symbol_counter);

        // Track which symbol kinds were produced
        let mut extracted_symbol_kinds = HashSet::new();
        for symbol in &symbols {
            extracted_symbol_kinds.insert(format!("{:?}", symbol.kind));
        }

        // Track which nodes our parser handles by analyzing the source
        let implemented_nodes = Self::find_implemented_nodes();

        Ok(Self {
            grammar_nodes,
            implemented_nodes,
            extracted_symbol_kinds,
        })
    }

    /// Find which nodes are handled in parser implementation
    /// This is a simple approach - in production we might instrument the parser
    fn find_implemented_nodes() -> HashSet<String> {
        // For now, we'll hardcode based on our knowledge
        // In a real implementation, we could:
        // 1. Parse the parser.rs file itself
        // 2. Or add instrumentation to track visited nodes
        // 3. Or use compile-time reflection

        let mut nodes = HashSet::new();

        // Main symbol-producing nodes
        nodes.insert("class_declaration".to_string());
        nodes.insert("interface_declaration".to_string());
        nodes.insert("enum_declaration".to_string());
        nodes.insert("type_alias_declaration".to_string());
        nodes.insert("function_declaration".to_string());
        nodes.insert("method_definition".to_string());
        nodes.insert("public_field_definition".to_string());
        nodes.insert("variable_declaration".to_string());
        nodes.insert("lexical_declaration".to_string());

        // Function variations
        nodes.insert("arrow_function".to_string());
        nodes.insert("function_expression".to_string());
        nodes.insert("generator_function_declaration".to_string());

        // Import/Export
        nodes.insert("import_statement".to_string());
        nodes.insert("export_statement".to_string());
        nodes.insert("namespace_import".to_string());
        nodes.insert("named_imports".to_string());

        // Parameters
        nodes.insert("required_parameter".to_string());
        nodes.insert("optional_parameter".to_string());
        nodes.insert("rest_parameter".to_string());

        nodes
    }

    /// Generate coverage report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();

        report.push_str("# TypeScript Parser Coverage Report\n\n");

        // Summary
        report.push_str("## Summary\n");
        report.push_str(&format!("- Nodes in file: {}\n", self.grammar_nodes.len()));
        report.push_str(&format!(
            "- Nodes handled by parser: {}\n",
            self.implemented_nodes.len()
        ));
        report.push_str(&format!(
            "- Symbol kinds extracted: {}\n",
            self.extracted_symbol_kinds.len()
        ));

        // Coverage table
        report.push_str("\n## Coverage Table\n\n");
        report.push_str("| Node Type | ID | Status |\n");
        report.push_str("|-----------|-----|--------|\n");

        // Key nodes we care about for symbol extraction
        let key_nodes = vec![
            "class_declaration",
            "interface_declaration",
            "enum_declaration",
            "type_alias_declaration",
            "function_declaration",
            "method_definition",
            "public_field_definition",
            "private_field_definition",
            "variable_declaration",
            "lexical_declaration",
            "arrow_function",
            "function_expression",
            "generator_function_declaration",
            "import_statement",
            "export_statement",
            "namespace_import",
            "named_imports",
            "required_parameter",
            "optional_parameter",
            "rest_parameter",
            "type_parameter",
            "type_annotation",
            "predefined_type",
            "namespace_declaration",
            "module_declaration",
        ];

        let mut gaps = Vec::new();
        let mut missing = Vec::new();

        for node_name in key_nodes {
            let status = if let Some(id) = self.grammar_nodes.get(node_name) {
                if self.implemented_nodes.contains(node_name) {
                    format!("{id} | ✅ implemented")
                } else {
                    gaps.push(node_name);
                    format!("{id} | ⚠️ gap")
                }
            } else {
                missing.push(node_name);
                "- | ❌ not found".to_string()
            };
            report.push_str(&format!("| {node_name} | {status} |\n"));
        }

        // Add legend
        report.push_str("\n## Legend\n\n");
        report
            .push_str("- ✅ **implemented**: Node type is recognized and handled by the parser\n");
        report.push_str("- ⚠️ **gap**: Node type exists in the grammar but not handled by parser (needs implementation)\n");
        report.push_str("- ❌ **not found**: Node type not present in the example file (may need better examples)\n");

        // Add recommendations
        report.push_str("\n## Recommended Actions\n\n");

        if !gaps.is_empty() {
            report.push_str("### Priority 1: Implementation Gaps\n");
            report.push_str("These nodes exist in your code but aren't being captured:\n\n");
            for gap in &gaps {
                report.push_str(&format!("- `{gap}`: Add parsing logic in parser.rs\n"));
            }
            report.push('\n');
        }

        if !missing.is_empty() {
            report.push_str("### Priority 2: Missing Examples\n");
            report.push_str("These nodes aren't in the comprehensive example. Consider:\n\n");
            for node in &missing {
                report.push_str(&format!(
                    "- `{node}`: Add example to comprehensive.ts or verify node name\n"
                ));
            }
            report.push('\n');
        }

        if gaps.is_empty() && missing.is_empty() {
            report.push_str("✨ **Excellent coverage!** All key nodes are implemented.\n");
        }

        report
    }
}

fn discover_nodes(node: Node, registry: &mut HashMap<String, u16>) {
    registry.insert(node.kind().to_string(), node.kind_id());

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        discover_nodes(child, registry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_simple_typescript() {
        let code = r#"
interface Example {
    name: string;
    getValue(): number;
}

class MyClass implements Example {
    name: string = "test";
    
    getValue(): number {
        return 42;
    }
}
"#;

        let audit = TypeScriptParserAudit::audit_code(code).unwrap();

        // Should find these nodes in the code
        assert!(audit.grammar_nodes.contains_key("interface_declaration"));
        assert!(audit.grammar_nodes.contains_key("class_declaration"));
        assert!(audit.grammar_nodes.contains_key("method_definition"));

        // Should extract Interface and Class symbols
        assert!(audit.extracted_symbol_kinds.contains("Interface"));
        assert!(audit.extracted_symbol_kinds.contains("Class"));
    }
}
