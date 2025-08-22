//! Tree-sitter ABI-15 feature exploration and documentation
//!
//! This test file explores available ABI-15 features that could enhance
//! our LanguageBehavior trait implementation in Stage 2 of the refactoring.
//!
//! Run with: cargo test abi15_exploration --nocapture
//!
//! Key findings will be used to inform the design of language-specific
//! behavior abstractions.

#[cfg(test)]
mod abi15_tests {
    use tree_sitter::Language;

    #[test]
    fn explore_typescript_interface_extends_structure() {
        let language: Language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into();
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&language).unwrap();

        println!("\n=== TypeScript Interface Extends Structure Exploration ===");

        let code = r#"
interface Serializable {
    serialize(): string;
}

interface AdvancedSerializable extends Serializable {
    deserialize(data: string): void;
}

class User extends BaseEntity implements Serializable {
    name: string;
}
"#;

        let tree = parser.parse(code, None).unwrap();
        let root = tree.root_node();

        println!("Analyzing interface and class inheritance structures:\n");

        fn analyze_node(node: tree_sitter::Node, code: &str, depth: usize) {
            let indent = "  ".repeat(depth);

            if node.kind() == "interface_declaration" || node.kind() == "class_declaration" {
                println!("{}Found {} at depth {}:", indent, node.kind(), depth);
                println!(
                    "{}  Full text: '{}'",
                    indent,
                    &code[node.byte_range()].lines().next().unwrap_or("")
                );

                // Show all children with field names
                let mut cursor = node.walk();
                for (i, child) in node.children(&mut cursor).enumerate() {
                    let field_name = node.field_name_for_child(i as u32);
                    println!(
                        "{}  Child {}: [{}] field={:?}",
                        indent,
                        i,
                        child.kind(),
                        field_name
                    );

                    // Dive deeper into extends/implements related nodes
                    if child.kind().contains("extends")
                        || child.kind().contains("implements")
                        || child.kind() == "class_heritage"
                    {
                        println!("{}    -> Exploring {}:", indent, child.kind());
                        let mut sub_cursor = child.walk();
                        for (j, subchild) in child.children(&mut sub_cursor).enumerate() {
                            let sub_field = child.field_name_for_child(j as u32);
                            println!(
                                "{}      Sub {}: [{}] field={:?} text='{}'",
                                indent,
                                j,
                                subchild.kind(),
                                sub_field,
                                &code[subchild.byte_range()]
                            );
                        }
                    }
                }
                println!();
            }

            // Recurse
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                analyze_node(child, code, depth + 1);
            }
        }

        analyze_node(root, code, 0);

        println!("\n=== KEY FINDINGS ===");
        println!("1. Check if 'extends' is a field or a child node for interfaces");
        println!(
            "2. Identify the exact node kind for interface extends (extends_clause vs extends_type_clause)"
        );
        println!("3. Compare with class extends structure");
    }

    #[test]
    fn explore_typescript_generic_constructor_nodes() {
        let language: Language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into();
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&language).unwrap();

        println!("\n=== TypeScript Generic Constructor Node Exploration ===");

        // Test case: new Map<string, Session>()
        let code = r#"
interface Session {
    id: string;
}
const sessions = new Map<string, Session>();
const simple = new Map();
const nested = new Array<Map<string, User>>();
const func = useState<Session>(null);
const typed: Map<string, Session> = new Map();
"#;

        let tree = parser.parse(code, None).unwrap();
        let root = tree.root_node();

        fn print_node_tree(node: tree_sitter::Node, code: &str, indent: usize) {
            let node_text = &code[node.byte_range()];
            let truncated = if node_text.len() > 40 {
                format!("{}...", &node_text[..40].replace('\n', "\\n"))
            } else {
                node_text.replace('\n', "\\n")
            };

            println!(
                "{:indent$}[{}] '{}' (id: {}, has_field: {})",
                "",
                node.kind(),
                truncated,
                node.kind_id(),
                node.child_count() > 0,
                indent = indent
            );

            // Print field names if available
            let mut cursor = node.walk();
            for (i, child) in node.children(&mut cursor).enumerate() {
                if let Some(field_name) = node.field_name_for_child(i as u32) {
                    println!(
                        "{:indent$}  └─ field: '{}'",
                        "",
                        field_name,
                        indent = indent + 2
                    );
                }
                print_node_tree(child, code, indent + 4);
            }
        }

        println!("\nFull tree structure:");
        print_node_tree(root, code, 0);

        // Now specifically look for patterns
        println!("\n=== Analyzing 'new Map<string, Session>()' pattern ===");

        fn find_new_expressions(node: tree_sitter::Node, code: &str, depth: usize) {
            if node.kind() == "new_expression" {
                println!("\nFound new_expression at depth {depth}:");
                println!("  Full text: '{}'", &code[node.byte_range()]);

                let mut cursor = node.walk();
                for (i, child) in node.children(&mut cursor).enumerate() {
                    let field_name = node.field_name_for_child(i as u32);
                    println!(
                        "  Child {}: [{}] field={:?} text='{}'",
                        i,
                        child.kind(),
                        field_name,
                        &code[child.byte_range()]
                    );

                    // If this is type_arguments, explore deeper
                    if child.kind() == "type_arguments" {
                        println!("    Found type_arguments!");
                        let mut type_cursor = child.walk();
                        for (j, type_child) in child.children(&mut type_cursor).enumerate() {
                            println!(
                                "      Type arg {}: [{}] '{}'",
                                j,
                                type_child.kind(),
                                &code[type_child.byte_range()]
                            );
                        }
                    }
                }
            }

            // Recurse
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                find_new_expressions(child, code, depth + 1);
            }
        }

        find_new_expressions(root, code, 0);
    }

    #[test]
    fn explore_rust_abi15_features() {
        let language: Language = tree_sitter_rust::LANGUAGE.into();

        println!("=== Rust Language ABI-15 Metadata ===");
        println!("  ABI Version: {}", language.abi_version());
        println!("  Field count: {}", language.field_count());
        println!("  Node kind count: {}", language.node_kind_count());

        // Explore node types that could inform LanguageBehavior
        println!("\n  Key Node Types for Symbol Extraction:");
        for node_kind in &[
            "function_item",
            "impl_item",
            "struct_item",
            "trait_item",
            "mod_item",
            "enum_item",
            "type_alias",
            "type_item",
            "const_item",
            "static_item",
            "macro_definition",
            "macro_rules",
        ] {
            let id = language.id_for_node_kind(node_kind, true);
            if id != 0 {
                println!("    {node_kind} -> ID: {id}");
            }
        }

        // Check field names (useful for extracting specific parts)
        println!("\n  Available Fields: {}", language.field_count());
        for i in 0..5.min(language.field_count()) {
            if let Some(name) = language.field_name_for_id(i as u16) {
                println!("    Field {i}: {name}");
            }
        }

        // TODO: Explore supertype information when API is clearer
        // TODO: Check for reserved word functionality
    }

    #[test]
    fn explore_python_abi15_features() {
        let language: Language = tree_sitter_python::LANGUAGE.into();

        println!("\n=== Python Language ABI-15 Metadata ===");
        println!("  ABI Version: {}", language.abi_version());
        println!("  Field count: {}", language.field_count());
        println!("  Node kind count: {}", language.node_kind_count());

        // Explore node types for symbol extraction
        println!("\n  Key Node Types for Symbol Extraction:");
        for node_kind in &[
            "function_definition",
            "class_definition",
            "assignment",
            "expression_statement",
            "annotated_assignment",
            "type_alias_statement",
            "decorator",
            "decorated_definition",
            "global_statement",
            "identifier",
            "module",
        ] {
            let id = language.id_for_node_kind(node_kind, true);
            if id != 0 {
                println!("    {node_kind} -> ID: {id}");
            }
        }
    }

    #[test]
    fn explore_python_abi15_comprehensive() {
        let language: Language = tree_sitter_python::LANGUAGE.into();

        println!("\n=== Python Language ABI-15 COMPREHENSIVE NODE MAPPING ===");
        println!("  ABI Version: {}", language.abi_version());
        println!("  Node kind count: {}", language.node_kind_count());

        println!("\n=== FUNCTION-RELATED NODES ===");
        for node_kind in &[
            "function_definition",
            "lambda",
            "async_function_definition",
            "decorator",
            "decorated_definition",
            "parameters",
            "default_parameter",
            "typed_parameter",
            "typed_default_parameter",
        ] {
            let id = language.id_for_node_kind(node_kind, true);
            if id != 0 {
                println!("  ✓ {node_kind} -> ID: {id}");
            } else {
                println!("  ✗ {node_kind} NOT FOUND");
            }
        }

        println!("\n=== CLASS-RELATED NODES ===");
        for node_kind in &[
            "class_definition",
            "class_body",
            "argument_list",
            "inheritance",
            "base_list",
            "metaclass",
        ] {
            let id = language.id_for_node_kind(node_kind, true);
            if id != 0 {
                println!("  ✓ {node_kind} -> ID: {id}");
            } else {
                println!("  ✗ {node_kind} NOT FOUND");
            }
        }

        println!("\n=== VARIABLE/ASSIGNMENT NODES ===");
        for node_kind in &[
            "assignment",
            "augmented_assignment",
            "annotated_assignment",
            "expression_statement",
            "global_statement",
            "nonlocal_statement",
            "identifier",
            "attribute",
            "subscript",
        ] {
            let id = language.id_for_node_kind(node_kind, true);
            if id != 0 {
                println!("  ✓ {node_kind} -> ID: {id}");
            } else {
                println!("  ✗ {node_kind} NOT FOUND");
            }
        }

        println!("\n=== TYPE-RELATED NODES ===");
        for node_kind in &[
            "type",
            "type_alias_statement",
            "generic_type",
            "union_type",
            "type_parameter",
            "type_comment",
            "type_hint",
            "type_annotation",
        ] {
            let id = language.id_for_node_kind(node_kind, true);
            if id != 0 {
                println!("  ✓ {node_kind} -> ID: {id}");
            } else {
                println!("  ✗ {node_kind} NOT FOUND");
            }
        }

        println!("\n=== IMPORT-RELATED NODES ===");
        for node_kind in &[
            "import_statement",
            "import_from_statement",
            "aliased_import",
            "dotted_name",
            "relative_import",
            "wildcard_import",
        ] {
            let id = language.id_for_node_kind(node_kind, true);
            if id != 0 {
                println!("  ✓ {node_kind} -> ID: {id}");
            } else {
                println!("  ✗ {node_kind} NOT FOUND");
            }
        }

        println!("\n=== CONSTANT/LITERAL NODES ===");
        for node_kind in &[
            "integer",
            "float",
            "string",
            "true",
            "false",
            "none",
            "list",
            "dictionary",
            "set",
            "tuple",
        ] {
            let id = language.id_for_node_kind(node_kind, true);
            if id != 0 {
                println!("  ✓ {node_kind} -> ID: {id}");
            } else {
                println!("  ✗ {node_kind} NOT FOUND");
            }
        }

        println!("\n=== ASYNC/GENERATOR NODES ===");
        for node_kind in &[
            "async_function_definition",
            "async_with_statement",
            "async_for_statement",
            "await_expression",
            "yield_expression",
            "generator_expression",
        ] {
            let id = language.id_for_node_kind(node_kind, true);
            if id != 0 {
                println!("  ✓ {node_kind} -> ID: {id}");
            } else {
                println!("  ✗ {node_kind} NOT FOUND");
            }
        }

        println!("\n=== DOCUMENTATION NODES ===");
        for node_kind in &["comment", "string", "expression_statement", "docstring"] {
            let id = language.id_for_node_kind(node_kind, true);
            if id != 0 {
                println!("  ✓ {node_kind} -> ID: {id}");
            } else {
                println!("  ✗ {node_kind} NOT FOUND");
            }
        }

        println!("\n=== Summary ===");
        println!("Total node kinds available: {}", language.node_kind_count());
        println!("Use these node names in the Python parser to extract symbols!");
    }

    #[test]
    fn explore_typescript_abi15_comprehensive() {
        let language: Language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into();

        println!("=== TypeScript Language ABI-15 COMPREHENSIVE NODE MAPPING ===");
        println!("  ABI Version: {}", language.abi_version());
        println!("  Node kind count: {}", language.node_kind_count());

        println!("\n=== CLASS-RELATED NODES ===");
        for node_kind in &[
            "class",
            "class_declaration",
            "abstract_class_declaration",
            "class_body",
            "class_heritage",
            "extends_clause",
            "implements_clause",
            "method_definition",
            "public_field_definition",
            "private_field_definition",
            "property_declaration",
            "constructor",
            "abstract",
            "abstract_method_signature",
        ] {
            let id = language.id_for_node_kind(node_kind, true);
            if id != 0 {
                println!("    {node_kind:30} -> ID: {id}");
            }
        }

        println!("\n=== INTERFACE-RELATED NODES ===");
        for node_kind in &[
            "interface",
            "interface_declaration",
            "interface_body",
            "property_signature",
            "method_signature",
            "index_signature",
            "extends_type_clause",
            "extends_clause",
        ] {
            let id = language.id_for_node_kind(node_kind, true);
            if id != 0 {
                println!("    {node_kind:30} -> ID: {id}");
            }
        }

        println!("\n=== TYPE-RELATED NODES ===");
        for node_kind in &[
            "type_alias_declaration",
            "type_annotation",
            "type_identifier",
            "type_parameter",
            "type_parameters",
            "type_arguments",
            "generic_type",
            "union_type",
            "intersection_type",
            "conditional_type",
            "literal_type",
            "template_literal_type",
            "nested_type_identifier",
        ] {
            let id = language.id_for_node_kind(node_kind, true);
            if id != 0 {
                println!("    {node_kind:30} -> ID: {id}");
            }
        }

        println!("\n=== ENUM-RELATED NODES ===");
        for node_kind in &[
            "enum",
            "enum_declaration",
            "enum_body",
            "enum_assignment",
            "enum_member",
        ] {
            let id = language.id_for_node_kind(node_kind, true);
            if id != 0 {
                println!("    {node_kind:30} -> ID: {id}");
            }
        }

        println!("\n=== FUNCTION-RELATED NODES ===");
        for node_kind in &[
            "function",
            "function_declaration",
            "function_expression",
            "arrow_function",
            "generator_function",
            "generator_function_declaration",
            "async_function",
            "async_arrow_function",
            "method_definition",
            "formal_parameters",
            "required_parameter",
            "optional_parameter",
            "rest_parameter",
        ] {
            let id = language.id_for_node_kind(node_kind, true);
            if id != 0 {
                println!("    {node_kind:30} -> ID: {id}");
            }
        }

        println!("\n=== VARIABLE/CONST NODES ===");
        for node_kind in &[
            "variable_declaration",
            "variable_declarator",
            "lexical_declaration",
            "const",
            "let",
            "var",
        ] {
            let id = language.id_for_node_kind(node_kind, true);
            if id != 0 {
                println!("    {node_kind:30} -> ID: {id}");
            }
        }

        println!("\n=== IMPORT/EXPORT NODES ===");
        for node_kind in &[
            "import_statement",
            "import_clause",
            "named_imports",
            "namespace_import",
            "import_specifier",
            "export_statement",
            "export_clause",
            "export_specifier",
            "export_default",
        ] {
            let id = language.id_for_node_kind(node_kind, true);
            if id != 0 {
                println!("    {node_kind:30} -> ID: {id}");
            }
        }

        println!("\n=== MODULE/NAMESPACE NODES ===");
        for node_kind in &[
            "module",
            "internal_module",
            "module_declaration",
            "namespace_declaration",
            "ambient_declaration",
        ] {
            let id = language.id_for_node_kind(node_kind, true);
            if id != 0 {
                println!("    {node_kind:30} -> ID: {id}");
            }
        }

        println!("\n=== DECORATOR NODES ===");
        for node_kind in &[
            "decorator",
            "decorator_member_expression",
            "decorator_call_expression",
        ] {
            let id = language.id_for_node_kind(node_kind, true);
            if id != 0 {
                println!("    {node_kind:30} -> ID: {id}");
            }
        }

        println!("\n=== CALL-RELATED NODES ===");
        for node_kind in &[
            "call_expression",
            "member_expression",
            "subscript_expression",
            "new_expression",
            "await_expression",
            "optional_chain",
            "arguments",
            "argument",
            "super",
            "this",
        ] {
            let id = language.id_for_node_kind(node_kind, true);
            if id != 0 {
                println!("    {node_kind:30} -> ID: {id}");
            }
        }

        // TYPE USAGE NODES (for find_uses implementation)
        println!("\n=== TYPE USAGE NODES (for find_uses) ===");
        for node_kind in &[
            "formal_parameters",
            "required_parameter",
            "optional_parameter",
            "rest_parameter",
            "type_annotation",
            "return_type",
            "implements_clause",
            "extends_clause",
            "constraint",
            "default_type",
            "variable_declarator",
            "lexical_declaration",
            "variable_declaration",
        ] {
            let id = language.id_for_node_kind(node_kind, true);
            if id != 0 {
                println!("    {node_kind:30} -> ID: {id}");
            }
        }

        println!("\n=== IMPORTANT: Use these exact node names in parser implementation!");
        println!("=== DO NOT GUESS node names - always verify with this test first!");
    }

    #[test]
    fn explore_php_defines_comprehensive() {
        let language: Language = tree_sitter_php::LANGUAGE_PHP.into();
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&language).unwrap();

        println!("\n=== PHP DEFINES EXPLORATION - DEBUGGING CODANNA ISSUE ===");

        let code = r#"<?php

interface Logger {
    public function log(string $message): void;
    public function warn(string $message): void;
}

class DatabaseLogger implements Logger {
    private string $connection;

    public function __construct(string $connection) {
        $this->connection = $connection;
    }

    public function log(string $message): void {
        echo "[DB] " . $message . "\n";
    }

    public function warn(string $message): void {
        echo "[DB WARNING] " . $message . "\n";
    }

    public function connect(): bool {
        return strlen($this->connection) > 0;
    }
}
"#;

        if let Some(tree) = parser.parse(code, None) {
            let root = tree.root_node();

            println!("=== FULL TREE STRUCTURE ===");
            print_php_tree(root, code, 0);

            println!("\n=== SEARCHING FOR DEFINES RELATIONSHIPS ===");
            find_php_defines(root, code, 0);
        }

        fn print_php_tree(node: tree_sitter::Node, code: &str, depth: usize) {
            let indent = "  ".repeat(depth);
            let node_text = &code[node.byte_range()];
            let first_line = node_text.lines().next().unwrap_or("").trim();

            println!(
                "{}[{}] '{}'",
                indent,
                node.kind(),
                if first_line.len() > 50 {
                    format!("{}...", &first_line[..50])
                } else {
                    first_line.to_string()
                }
            );

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                print_php_tree(child, code, depth + 1);
            }
        }

        fn find_php_defines(node: tree_sitter::Node, code: &str, depth: usize) {
            let indent = "  ".repeat(depth);

            match node.kind() {
                "interface_declaration" => {
                    println!("{indent}🎯 FOUND INTERFACE_DECLARATION!");
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let interface_name = &code[name_node.byte_range()];
                        println!("{indent}  Interface name: '{interface_name}'");

                        // Look for methods within interface
                        let mut cursor = node.walk();
                        for child in node.children(&mut cursor) {
                            println!(
                                "{}    Child: [{}] '{}'",
                                indent,
                                child.kind(),
                                &code[child.byte_range()].lines().next().unwrap_or("")
                            );

                            if child.kind() == "method_declaration" {
                                println!("{indent}      🎯 FOUND METHOD_DECLARATION in interface!");
                                if let Some(method_name_node) = child.child_by_field_name("name") {
                                    let method_name = &code[method_name_node.byte_range()];
                                    println!(
                                        "{indent}        DEFINES: {interface_name} -> {method_name}"
                                    );
                                }
                            }
                        }
                    }
                }
                "class_declaration" => {
                    println!("{indent}🎯 FOUND CLASS_DECLARATION!");
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let class_name = &code[name_node.byte_range()];
                        println!("{indent}  Class name: '{class_name}'");

                        // Look for methods within class
                        let mut cursor = node.walk();
                        for child in node.children(&mut cursor) {
                            println!(
                                "{}    Child: [{}] '{}'",
                                indent,
                                child.kind(),
                                &code[child.byte_range()].lines().next().unwrap_or("")
                            );

                            if child.kind() == "method_declaration" {
                                println!("{indent}      🎯 FOUND METHOD_DECLARATION in class!");
                                if let Some(method_name_node) = child.child_by_field_name("name") {
                                    let method_name = &code[method_name_node.byte_range()];
                                    println!(
                                        "{indent}        DEFINES: {class_name} -> {method_name}"
                                    );
                                }
                            }
                        }
                    }
                }
                _ => {}
            }

            // Recurse
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                find_php_defines(child, code, depth + 1);
            }
        }

        println!("\n=== SUMMARY ===");
        println!("This test shows exactly what PHP tree-sitter produces.");
        println!("If we see interface_declaration and class_declaration nodes");
        println!("with method_declaration children, then the PHP parser");
        println!("extract_defines_from_node method should work.");
        println!("If not, we need to fix the implementation.");
    }

    #[test]
    fn explore_php_abi15_features() {
        let language: Language = tree_sitter_php::LANGUAGE_PHP.into();

        println!("\n=== PHP Language ABI-15 Metadata ===");
        println!("  ABI Version: {}", language.abi_version());
        println!("  Node kind count: {}", language.node_kind_count());

        println!("\n  Key Node Types:");
        for node_kind in &[
            "class_declaration",
            "function_definition",
            "method_declaration",
            "interface_declaration",
            "const_declaration",
            "const_element",
            "function_call_expression",
            "assignment_expression",
            "expression_statement",
            "namespace_definition",
            "enum_declaration",
            "global_declaration",
            "simple_parameter",
        ] {
            let id = language.id_for_node_kind(node_kind, true);
            if id != 0 {
                println!("    {node_kind} -> ID: {id}");
            }
        }
    }

    #[test]
    fn explore_typescript_abi15_features() {
        let language: Language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into();

        println!("\n=== TypeScript Language ABI-15 Metadata ===");
        println!("  ABI Version: {}", language.abi_version());
        println!("  Field count: {}", language.field_count());
        println!("  Node kind count: {}", language.node_kind_count());

        // Explore TypeScript-specific node types for symbol extraction
        println!("\n  Key Node Types for Symbol Extraction:");
        for node_kind in &[
            // Function-related
            "function_declaration",
            "function_expression",
            "arrow_function",
            "generator_function_declaration",
            "method_definition",
            "function_signature",
            // Class-related
            "class_declaration",
            "class_expression",
            "constructor",
            "property_declaration",
            "method_signature",
            "public_field_definition",
            "private_field_definition",
            // Interface & Type
            "interface_declaration",
            "type_alias_declaration",
            "enum_declaration",
            "type_parameter",
            "type_annotation",
            // Variables
            "variable_declaration",
            "lexical_declaration",
            "const_declaration",
            "let_declaration",
            "variable_declarator",
            // Module/Namespace
            "module_declaration",
            "namespace_declaration",
            "export_statement",
            "import_statement",
            "ambient_declaration",
            // Decorators
            "decorator",
            "decorator_expression",
            // JSX/TSX
            "jsx_element",
            "jsx_self_closing_element",
            "jsx_opening_element",
            "jsx_attribute",
        ] {
            let id = language.id_for_node_kind(node_kind, true);
            if id != 0 {
                println!("    {node_kind} -> ID: {id}");
            }
        }

        // Check field names for TypeScript-specific constructs
        println!("\n  Available Fields: {}", language.field_count());
        for i in 0..10.min(language.field_count()) {
            if let Some(name) = language.field_name_for_id(i as u16) {
                println!("    Field {i}: {name}");
            }
        }

        // Test TypeScript vs JavaScript differences
        let js_language: Language = tree_sitter_javascript::LANGUAGE.into();
        println!("\n  TypeScript vs JavaScript Comparison:");
        println!("    TypeScript nodes: {}", language.node_kind_count());
        println!("    JavaScript nodes: {}", js_language.node_kind_count());
        println!(
            "    Difference: {} additional nodes",
            language.node_kind_count() as i32 - js_language.node_kind_count() as i32
        );
    }

    #[test]
    fn explore_language_behavior_candidates() {
        println!("\n=== Potential LanguageBehavior Enhancements ===");

        // Compare capabilities across languages
        let rust_lang: Language = tree_sitter_rust::LANGUAGE.into();
        let python_lang: Language = tree_sitter_python::LANGUAGE.into();
        let php_lang: Language = tree_sitter_php::LANGUAGE_PHP.into();

        println!("\n  Cross-Language Comparison:");
        println!("  Language    | ABI | Node Kinds | Fields");
        println!("  ------------|-----|------------|-------");
        println!(
            "  Rust        | {:3} | {:10} | {:6}",
            rust_lang.abi_version(),
            rust_lang.node_kind_count(),
            rust_lang.field_count()
        );
        println!(
            "  Python      | {:3} | {:10} | {:6}",
            python_lang.abi_version(),
            python_lang.node_kind_count(),
            python_lang.field_count()
        );
        println!(
            "  PHP         | {:3} | {:10} | {:6}",
            php_lang.abi_version(),
            php_lang.node_kind_count(),
            php_lang.field_count()
        );

        // Test common node type mapping
        println!("\n  Common Symbol Types Across Languages:");
        let common_concepts = vec![
            (
                "Function",
                vec![
                    "function_item",
                    "function_definition",
                    "function_definition",
                ],
            ),
            (
                "Class",
                vec!["struct_item", "class_definition", "class_declaration"],
            ),
            (
                "Method",
                vec!["function_item", "function_definition", "method_declaration"],
            ),
        ];

        for (concept, node_kinds) in common_concepts {
            println!("    {concept}:");
            println!(
                "      Rust:   {} (ID: {})",
                node_kinds[0],
                rust_lang.id_for_node_kind(node_kinds[0], true)
            );
            println!(
                "      Python: {} (ID: {})",
                node_kinds[1],
                python_lang.id_for_node_kind(node_kinds[1], true)
            );
            println!(
                "      PHP:    {} (ID: {})",
                node_kinds[2],
                php_lang.id_for_node_kind(node_kinds[2], true)
            );
        }

        println!("\n  Implications for LanguageBehavior:");
        println!("  - Each language has different node naming conventions");
        println!("  - ABI-15 provides consistent metadata access");
        println!("  - Can validate node types at behavior construction");
        println!("  - Field information could enhance symbol extraction");
    }

    #[test]
    fn explore_typescript_import_structure() {
        use tree_sitter::{Node, Parser};

        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
            .unwrap();

        println!("\n=== TypeScript Import Statement Structure ===");
        println!("CRITICAL LESSON: import_clause is NOT a field, it's a child node!");
        println!("This means node.child_by_field_name(\"import_clause\") returns None!");
        println!("Must use: node.children(&mut cursor).find(|c| c.kind() == \"import_clause\")\n");

        let test_cases = vec![
            ("import React from 'react';", "Default import"),
            ("import { Component } from 'react';", "Named import"),
            ("import React, { Component } from 'react';", "Mixed import"),
            ("import * as utils from './utils';", "Namespace import"),
            ("import type { Props } from './types';", "Type-only import"),
            ("import './styles.css';", "Side-effect import"),
        ];

        for (code, description) in test_cases {
            println!("--- {description} ---");
            println!("Code: {code}");

            if let Some(tree) = parser.parse(code, None) {
                let root = tree.root_node();
                let mut cursor = root.walk();

                for child in root.children(&mut cursor) {
                    if child.kind() == "import_statement" {
                        analyze_import_node(child, code);
                    }
                }
            }
            println!();
        }

        println!("=== KEY FINDINGS FOR IMPLEMENTATION ===");
        println!("1. import_clause is a CHILD not a FIELD");
        println!("2. source IS a field (use child_by_field_name(\"source\"))");
        println!("3. Within import_clause:");
        println!("   - First 'identifier' child = default import name");
        println!("   - 'named_imports' child = {{ Component, useState }}");
        println!("   - 'namespace_import' child = * as name");
        println!("4. For namespace imports, the identifier is nested inside namespace_import");
        println!("5. Type-only imports have a 'type' keyword as first child");

        fn analyze_import_node(node: Node, code: &str) {
            println!("  import_statement structure:");

            // Show all children with field names
            let mut cursor = node.walk();
            for (i, child) in node.children(&mut cursor).enumerate() {
                let field_name = node.field_name_for_child(i as u32);
                println!(
                    "    [{}] kind='{}', field={:?}, text='{}'",
                    i,
                    child.kind(),
                    field_name,
                    &code[child.byte_range()]
                );

                // Dive into import_clause
                if child.kind() == "import_clause" {
                    let mut clause_cursor = child.walk();
                    for (j, grandchild) in child.children(&mut clause_cursor).enumerate() {
                        println!(
                            "      clause[{}]: kind='{}', text='{}'",
                            j,
                            grandchild.kind(),
                            &code[grandchild.byte_range()]
                        );

                        // Show namespace_import contents
                        if grandchild.kind() == "namespace_import" {
                            let mut ns_cursor = grandchild.walk();
                            for (k, ggc) in grandchild.children(&mut ns_cursor).enumerate() {
                                println!(
                                    "        ns[{}]: kind='{}', text='{}'",
                                    k,
                                    ggc.kind(),
                                    &code[ggc.byte_range()]
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}
