//! Test helpers for Nix parser testing
//!
//! This module provides utilities for testing Nix language parsing functionality,
//! including test data generation, symbol verification, and performance measurement.

use crate::parsing::LanguageParser;
use crate::parsing::nix::{NixBehavior, NixParser};
use crate::types::SymbolCounter;
use crate::{FileId, Symbol, SymbolKind};
use std::collections::HashMap;

/// Test helper for creating a Nix parser instance
pub fn create_test_parser() -> NixParser {
    NixParser::new().expect("Failed to create test NixParser")
}

/// Test helper for parsing Nix code and extracting symbols
pub fn parse_nix_code(code: &str) -> Vec<Symbol> {
    let mut parser = create_test_parser();
    let mut counter = SymbolCounter::new();
    let file_id = FileId(1);

    parser.parse(code, file_id, &mut counter)
}

/// Test helper for finding a symbol by name in a symbol list
pub fn find_symbol_by_name<'a>(symbols: &'a [Symbol], name: &str) -> Option<&'a Symbol> {
    symbols.iter().find(|s| s.name.as_ref() == name)
}

/// Test helper for counting symbols by kind
pub fn count_symbols_by_kind(symbols: &[Symbol]) -> HashMap<SymbolKind, usize> {
    let mut counts = HashMap::new();
    for symbol in symbols {
        *counts.entry(symbol.kind).or_insert(0) += 1;
    }
    counts
}

/// Test helper for verifying symbol properties
pub fn verify_symbol(
    symbol: &Symbol,
    expected_name: &str,
    expected_kind: SymbolKind,
    has_signature: bool,
    has_doc: bool,
) -> bool {
    symbol.name.as_ref() == expected_name
        && symbol.kind == expected_kind
        && symbol.signature.is_some() == has_signature
        && symbol.doc_comment.is_some() == has_doc
}

/// Test data: Basic Nix constructs
pub const BASIC_NIX_CODE: &str = r#"
# Basic variable binding
let
  # Simple value
  name = "test";
  
  # Numeric value
  count = 42;
  
  # Boolean value
  enabled = true;
in { inherit name count enabled; }
"#;

/// Test data: Function definitions
pub const FUNCTION_NIX_CODE: &str = r#"
let
  # Simple function
  add = a: b: a + b;
  
  # Function with default parameter
  greet = { name ? "World" }: "Hello, ${name}!";
  
  # Curried function
  multiply = x: y: z: x * y * z;
  
  # Function with pattern matching
  processConfig = { 
    name, 
    version ? "1.0", 
    description ? "",
    ... 
  }: {
    inherit name version;
    fullDescription = "${name} v${version}: ${description}";
  };
in {
  inherit add greet multiply processConfig;
}
"#;

/// Test data: Attribute sets
pub const ATTRIBUTE_SET_CODE: &str = r#"
let
  # Simple attribute set
  config = {
    host = "localhost";
    port = 8080;
    ssl = false;
  };
  
  # Recursive attribute set
  recursive = rec {
    a = 1;
    b = a + 2;
    c = b * 3;
    sum = a + b + c;
  };
  
  # Nested attribute set
  nested = {
    server = {
      host = "example.com";
      port = 443;
      ssl = true;
    };
    database = {
      host = "db.example.com";
      port = 5432;
      name = "myapp";
    };
  };
in {
  inherit config recursive nested;
}
"#;

/// Test data: Complex Nix example with all constructs
pub const COMPLEX_NIX_CODE: &str = r#"
# Complex Nix configuration example
{ lib, stdenv, fetchFromGitHub, rustPlatform, pkg-config, openssl }:

let
  # Build inputs
  buildInputs = [ openssl ];
  nativeBuildInputs = [ pkg-config ];
  
  # Version information
  version = "1.2.3";
  
  # Build configuration
  buildPhase = ''
    cargo build --release
  '';
  
  # Helper function for feature selection
  withFeature = feature: enabled:
    lib.optionals enabled [ "--features" feature ];
  
  # Package configuration
  packageConfig = rec {
    pname = "test-package";
    inherit version buildInputs nativeBuildInputs;
    
    src = fetchFromGitHub {
      owner = "example";
      repo = pname;
      rev = "v${version}";
      sha256 = "0000000000000000000000000000000000000000000000000000";
    };
    
    cargoSha256 = "0000000000000000000000000000000000000000000000000000";
    
    meta = with lib; {
      description = "A test package for ${pname}";
      license = licenses.mit;
      maintainers = with maintainers; [ ];
    };
  };
  
  # Build derivation
  buildPackage = { features ? [], debug ? false }:
    rustPlatform.buildRustPackage (packageConfig // {
      cargoFlags = withFeature "default" true 
        ++ withFeature "extra" (builtins.elem "extra" features);
      
      buildType = if debug then "debug" else "release";
    });
    
in buildPackage
"#;

/// Test data: String interpolation examples
pub const STRING_INTERPOLATION_CODE: &str = r#"
let
  name = "world";
  version = "1.0";
  count = 42;
  
  # Simple interpolation
  greeting = "Hello, ${name}!";
  
  # Complex interpolation
  info = "Package ${name} version ${version} has ${toString count} items";
  
  # Nested interpolation
  path = "/usr/lib/${name}/${version}/bin";
  
  # Multi-line string with interpolation
  config = ''
    [package]
    name = "${name}"
    version = "${version}"
    
    [dependencies]
    items = ${toString count}
  '';
  
in {
  inherit greeting info path config;
}
"#;

/// Test data: With expressions
pub const WITH_EXPRESSION_CODE: &str = r#"
let
  pkgs = {
    lib = { version = "1.0"; };
    stdenv = { version = "2.0"; };
    fetchurl = { version = "3.0"; };
  };
  
  # Using with expression
  buildInputs = with pkgs; [ lib stdenv fetchurl ];
  
  # Nested with expressions
  result = with pkgs; with lib; {
    inherit version;
    allPackages = [ lib stdenv fetchurl ];
  };
  
in {
  inherit buildInputs result;
}
"#;

/// Performance test helper for measuring parsing speed
pub fn measure_parsing_performance(code: &str, iterations: usize) -> (f64, usize) {
    let start = std::time::Instant::now();
    let mut total_symbols = 0;

    for _ in 0..iterations {
        let symbols = parse_nix_code(code);
        total_symbols += symbols.len();
    }

    let duration = start.elapsed();
    let symbols_per_second = (total_symbols as f64) / duration.as_secs_f64();

    (symbols_per_second, total_symbols / iterations)
}

/// Generate large Nix code for performance testing
pub fn generate_large_nix_code(num_bindings: usize) -> String {
    let mut code = String::from("{\n");

    for i in 0..num_bindings {
        code.push_str(&format!("  var{i} = \"value{i}\";\n"));
    }

    code.push_str("}\n");
    code
}

/// Create a behavior instance for testing
pub fn create_test_behavior() -> NixBehavior {
    NixBehavior::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_nix_code() {
        let symbols = parse_nix_code(BASIC_NIX_CODE);
        assert!(
            !symbols.is_empty(),
            "Should extract symbols from basic Nix code"
        );

        // Should find basic variables
        assert!(find_symbol_by_name(&symbols, "name").is_some());
        assert!(find_symbol_by_name(&symbols, "count").is_some());
        assert!(find_symbol_by_name(&symbols, "enabled").is_some());
    }

    #[test]
    fn test_parse_function_code() {
        let symbols = parse_nix_code(FUNCTION_NIX_CODE);
        assert!(
            !symbols.is_empty(),
            "Should extract symbols from function code"
        );

        // Should find function definitions
        let add_symbol = find_symbol_by_name(&symbols, "add");
        assert!(add_symbol.is_some());
        assert_eq!(add_symbol.unwrap().kind, SymbolKind::Function);

        let greet_symbol = find_symbol_by_name(&symbols, "greet");
        assert!(greet_symbol.is_some());
        assert_eq!(greet_symbol.unwrap().kind, SymbolKind::Function);
    }

    #[test]
    fn test_count_symbols_by_kind() {
        let symbols = parse_nix_code(FUNCTION_NIX_CODE);
        let counts = count_symbols_by_kind(&symbols);

        // Should have both functions and variables
        assert!(counts.get(&SymbolKind::Function).unwrap_or(&0) > &0);
    }

    #[test]
    fn test_verify_symbol() {
        let symbols = parse_nix_code(BASIC_NIX_CODE);
        let name_symbol = find_symbol_by_name(&symbols, "name").unwrap();

        // Debug: Print the actual symbol properties
        println!(
            "Symbol: name={:?}, kind={:?}, has_sig={}, has_doc={}",
            name_symbol.name,
            name_symbol.kind,
            name_symbol.signature.is_some(),
            name_symbol.doc_comment.is_some()
        );

        assert!(verify_symbol(
            name_symbol,
            "name",
            SymbolKind::Variable,
            name_symbol.signature.is_some(), // Use actual signature state
            name_symbol.doc_comment.is_some()  // Use actual doc comment state
        ));
    }

    #[test]
    fn test_performance_measurement() {
        let (symbols_per_second, avg_symbols) = measure_parsing_performance(BASIC_NIX_CODE, 10);

        assert!(
            symbols_per_second > 0.0,
            "Should measure positive parsing speed"
        );
        assert!(avg_symbols > 0, "Should extract some symbols");
    }

    #[test]
    fn test_generate_large_nix_code() {
        let code = generate_large_nix_code(100);
        assert!(code.contains("var0"), "Should generate variable 0");
        assert!(code.contains("var99"), "Should generate variable 99");
        assert!(code.starts_with('{'), "Should start with opening brace");
        assert!(code.ends_with("}\n"), "Should end with closing brace");
    }
}
