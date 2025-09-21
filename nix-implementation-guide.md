# Nix Language Support Implementation Guide

This document provides comprehensive implementation details for adding Nix language support to the Codanna code intelligence system. It compiles all research findings and provides everything needed for immediate implementation.

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Codanna Architecture Analysis](#codanna-architecture-analysis)
3. [Nix Language Analysis & Symbol Mapping](#nix-language-analysis--symbol-mapping)
4. [tree-sitter-nix Integration](#tree-sitter-nix-integration)
5. [Implementation Specifications](#implementation-specifications)
6. [Registry Integration Requirements](#registry-integration-requirements)
7. [Testing Strategy](#testing-strategy)
8. [Implementation Timeline](#implementation-timeline)
9. [Verification Checklist](#verification-checklist)

## Executive Summary

Adding Nix language support to Codanna requires implementing four core components following established patterns: NixParser (symbol extraction), NixBehavior (language-specific behaviors), registry integration (language registration), and comprehensive testing. The implementation leverages tree-sitter-nix v0.6.1 for parsing and follows the same architectural patterns used by existing languages (Go, Rust, Python, TypeScript, PHP).

**Key Changes Required:**
- Language enum updates with Nix variant
- tree-sitter-nix dependency addition  
- New nix module with parser, behavior, and resolution components
- Registry integration and initialization updates
- Comprehensive test suite

**Implementation Effort:** 11-17 hours across 5 phases

## Codanna Architecture Analysis

### Language Parser System Architecture

Codanna uses a modular language parser system with the following key components:

#### 1. Language Registry System (`src/parsing/registry.rs`)
- **LanguageRegistry**: Central registry for all language definitions
- **LanguageDefinition trait**: Defines interface for language implementations
- **LanguageId**: Unique identifier for language types
- **Initialization**: Languages registered via `initialize_registry()` function

#### 2. Parser Factory System (`src/parsing/factory.rs`)
- **ParserFactory**: Creates parsers and behaviors for enabled languages
- **Language Detection**: Automatic detection from file extensions
- **Configuration Integration**: Respects language enable/disable settings

#### 3. Core Interfaces
- **LanguageParser trait**: Defines symbol extraction interface
- **LanguageBehavior trait**: Defines language-specific behaviors
- **ParserContext**: Manages parsing state and scope

#### 4. Existing Language Pattern Analysis

Based on Go language implementation (`src/parsing/go/`):

```rust
// Standard module structure
pub mod behavior;     // Language-specific behaviors
pub mod definition;   // Language registration
pub mod parser;       // Core parsing logic  
pub mod resolution;   // Symbol resolution context
```

**Key Integration Points:**
- `src/parsing/mod.rs`: Module exports and public API
- `src/parsing/registry.rs`: Language registration in `initialize_registry()`
- `src/parsing/language.rs`: Language enum with all methods updated

## Nix Language Analysis & Symbol Mapping

### Nix Language Constructs

Nix is a purely functional language for package management and system configuration. Key constructs requiring symbol extraction:

#### 1. Function Definitions
```nix
# Named function
myFunction = x: y: x + y;

# Function with pattern matching  
{ pkgs, lib, ... }: 
  # function body

# Lambda functions
map (x: x + 1) [1 2 3]
```
**Symbol Mapping:** `SymbolKind::Function`

#### 2. Variable Bindings
```nix
# Simple binding
let x = 42; in x

# Multiple bindings
let 
  a = 1;
  b = 2; 
in a + b

# Attribute set binding
rec {
  x = 1;
  y = x + 1;
}
```
**Symbol Mapping:** `SymbolKind::Variable`

#### 3. Attribute Sets (Objects/Records)
```nix
{
  name = "example";
  version = "1.0";
  dependencies = [ "foo" "bar" ];
}

# Recursive attribute sets
rec {
  a = 1;
  b = a + 1;
}
```
**Symbol Mapping:** `SymbolKind::Object` (or custom `SymbolKind::AttributeSet`)

#### 4. List Constructs
```nix
[ 1 2 3 4 ]
[ "a" "b" "c" ]
```
**Symbol Mapping:** `SymbolKind::Array`

#### 5. Conditional Expressions
```nix
if condition then value else other
```
**Symbol Mapping:** Control flow (no symbol extraction needed)

#### 6. With Expressions (Scope Introduction)
```nix
with pkgs; [
  git
  vim
  firefox
]
```
**Symbol Mapping:** Scope management in resolution context

### Nix-Specific Considerations

1. **No Explicit Type Declarations**: All types are inferred
2. **Lazy Evaluation**: Expressions evaluated on demand
3. **Immutable Bindings**: No variable reassignment
4. **Path Literals**: Special syntax for file paths (`/nix/store/...`, `./local/path`)
5. **String Interpolation**: `"Hello ${name}"`

## tree-sitter-nix Integration

### Parser Capabilities Assessment

**tree-sitter-nix v0.6.1** provides comprehensive parsing support for Nix language constructs:

#### Supported Node Types
- `source_file`: Root AST node
- `binding`: Variable/function bindings (`let`, `rec`, attribute assignments)
- `function`: Function definitions and lambda expressions  
- `attrset`: Attribute sets (objects)
- `list`: List constructs
- `identifier`: Variable/function names
- `string`: String literals and interpolated strings
- `path`: Path literals
- `conditional`: If-then-else expressions
- `with`: With expressions
- `comment`: Comments

#### Integration Requirements

**Cargo.toml Dependencies:**
```toml
tree-sitter = "0.25.8"
tree-sitter-nix = "0.6.1"
```

**Build Integration:**
- No additional build scripts required
- tree-sitter-nix includes pre-built parser
- Compatible with Rust 1.75+ (Edition 2024)

#### Performance Characteristics
- **Parsing Speed**: ~50,000+ nodes/second (typical tree-sitter performance)
- **Memory Usage**: ~50-100 bytes per AST node
- **Concurrent Safety**: Tree-sitter parsers are thread-safe after initialization

## Implementation Specifications

### 1. NixParser Implementation (`src/parsing/nix/parser.rs`)

```rust
use crate::parsing::{LanguageParser, ParserContext};
use crate::symbol::{Symbol, SymbolKind};
use crate::{IndexError, IndexResult};
use tree_sitter::{Language, Node, Parser, Query, QueryCursor, Tree};

/// Nix language parser using tree-sitter-nix
pub struct NixParser {
    parser: Parser,
    language: Language,
}

impl NixParser {
    pub fn new() -> IndexResult<Self> {
        let language = tree_sitter_nix::language();
        let mut parser = Parser::new();
        parser.set_language(language)
            .map_err(|e| IndexError::General(format!("Failed to set Nix language: {}", e)))?;
        
        Ok(Self { parser, language })
    }
    
    fn extract_bindings(&self, node: Node, source: &[u8], symbols: &mut Vec<Symbol>) {
        // Extract let bindings, attribute bindings, function definitions
        // Implementation details based on tree-sitter-nix node types
    }
    
    fn extract_functions(&self, node: Node, source: &[u8], symbols: &mut Vec<Symbol>) {
        // Extract function definitions and lambda expressions
    }
    
    fn extract_attribute_sets(&self, node: Node, source: &[u8], symbols: &mut Vec<Symbol>) {
        // Extract attribute set definitions
    }
}

impl LanguageParser for NixParser {
    fn language(&self) -> crate::parsing::Language {
        crate::parsing::Language::Nix
    }
    
    fn parse(&mut self, source: &[u8], _context: &ParserContext) -> IndexResult<Vec<Symbol>> {
        let tree = self.parser.parse(source, None)
            .ok_or_else(|| IndexError::General("Failed to parse Nix source".to_string()))?;
            
        let mut symbols = Vec::new();
        self.walk_tree(tree.root_node(), source, &mut symbols);
        Ok(symbols)
    }
}
```

### 2. NixBehavior Implementation (`src/parsing/nix/behavior.rs`)

```rust
use crate::parsing::{LanguageBehavior, LanguageMetadata};
use tree_sitter::Language;

/// Nix-specific language behaviors
pub struct NixBehavior {
    metadata: LanguageMetadata,
}

impl NixBehavior {
    pub fn new() -> Self {
        let metadata = LanguageMetadata::from_language(tree_sitter_nix::language());
        Self { metadata }
    }
}

impl LanguageBehavior for NixBehavior {
    fn get_language(&self) -> Language {
        tree_sitter_nix::language()
    }
    
    fn format_symbol_signature(&self, symbol: &crate::Symbol) -> String {
        match symbol.kind {
            crate::SymbolKind::Function => {
                format!("{} = <function>", symbol.name)
            },
            crate::SymbolKind::Variable => {
                format!("{} = <value>", symbol.name) 
            },
            _ => symbol.name.clone(),
        }
    }
    
    fn should_include_in_index(&self, symbol: &crate::Symbol) -> bool {
        // Include functions, variables, and attribute sets
        matches!(symbol.kind, 
            crate::SymbolKind::Function | 
            crate::SymbolKind::Variable |
            crate::SymbolKind::Object
        )
    }
    
    fn get_scope_delimiter(&self) -> &str {
        "." // Nix uses dot notation for attribute access
    }
}
```

### 3. NixResolutionContext Implementation (`src/parsing/nix/resolution.rs`)

```rust
use crate::parsing::{GenericResolutionContext, InheritanceResolver, ResolutionScope, ScopeLevel};

/// Nix-specific resolution context
pub struct NixResolutionContext {
    scopes: Vec<ResolutionScope>,
}

impl NixResolutionContext {
    pub fn new() -> Self {
        Self {
            scopes: vec![ResolutionScope::new(ScopeLevel::Global)],
        }
    }
    
    /// Handle let-in expression scoping
    pub fn enter_let_scope(&mut self) {
        self.scopes.push(ResolutionScope::new(ScopeLevel::Local));
    }
    
    /// Handle with expression scoping  
    pub fn enter_with_scope(&mut self) {
        self.scopes.push(ResolutionScope::new(ScopeLevel::Local));
    }
    
    /// Handle attribute set scoping
    pub fn enter_attrset_scope(&mut self) {
        self.scopes.push(ResolutionScope::new(ScopeLevel::Local));
    }
}

impl GenericResolutionContext for NixResolutionContext {
    // Implement required trait methods
}

/// Nix inheritance resolver (minimal - Nix doesn't have classical inheritance)
pub struct NixInheritanceResolver;

impl InheritanceResolver for NixInheritanceResolver {
    // Minimal implementation - Nix uses composition over inheritance
}
```

## Registry Integration Requirements

### 1. Language Enum Updates (`src/parsing/language.rs`)

Add Nix variant to the Language enum and update all associated methods:

```rust
// Line 10-17: Add Nix to enum
pub enum Language {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Php,
    Go,
    Nix,  // ADD THIS
}

// Update all match statements in methods:
// - to_language_id() - line 27
// - from_language_id() - line 47  
// - from_extension() - line 77
// - extensions() - line 99
// - config_key() - line 111
// - name() - line 123
```

### 2. Dependency Addition (`Cargo.toml`)

```toml
tree-sitter-typescript = "0.23.2"
tree-sitter-nix = "0.6.1"  # ADD THIS LINE
walkdir = "2.5.0"
```

### 3. Module Integration (`src/parsing/mod.rs`)

```rust
pub mod nix;         // ADD MODULE
pub mod typescript;

// Add exports
pub use nix::{NixBehavior, NixParser}; // ADD EXPORTS
```

### 4. Registry Registration (`src/parsing/registry.rs`)

```rust
fn initialize_registry(registry: &mut LanguageRegistry) {
    super::rust::register(registry);
    super::python::register(registry);
    super::php::register(registry);
    super::typescript::register(registry);
    super::go::register(registry);
    super::nix::register(registry); // ADD THIS LINE
}
```

### 5. Language Definition (`src/parsing/nix/definition.rs`)

Complete implementation following Go language pattern with NixLanguage struct implementing LanguageDefinition trait.

## Testing Strategy

### 1. Unit Tests Structure

```
src/parsing/nix/
├── parser_tests.rs      # Parser unit tests
├── behavior_tests.rs    # Behavior unit tests  
└── test_helpers.rs      # Test utilities
```

### 2. Integration Tests

```
tests/fixtures/nix/
├── basic_bindings.nix   # Simple variable bindings
├── functions.nix        # Function definitions
├── attribute_sets.nix   # Attribute set constructs
├── complex_example.nix  # Real-world Nix file
└── expected_symbols.json # Expected extraction results
```

### 3. Test Cases Coverage

#### Basic Language Constructs
- Variable bindings (`let x = 1; in x`)
- Function definitions (`f = x: x + 1`)
- Attribute sets (`{ name = "value"; }`)
- Lists (`[ 1 2 3 ]`)

#### Advanced Constructs  
- Recursive attribute sets (`rec { ... }`)
- With expressions (`with pkgs; [ ... ]`)
- String interpolation (`"Hello ${name}"`)
- Path literals (`./path/to/file`)

#### Real-World Examples
- NixOS configuration snippets
- Package definitions
- Flake.nix files
- Shell.nix files

### 4. Performance Tests

```rust
#[bench]
fn bench_nix_parsing_performance(b: &mut Bencher) {
    let source = include_bytes!("../fixtures/large_flake.nix");
    let mut parser = NixParser::new().unwrap();
    
    b.iter(|| {
        parser.parse(source, &ParserContext::default())
    });
}
```

### 5. Verification Commands

```bash
# Unit tests
cargo test nix --lib

# Integration tests  
cargo test nix --test '*'

# Performance benchmarks
cargo bench nix

# Full verification
cargo test && cargo clippy -- -D warnings && cargo fmt --check
```

## Implementation Timeline

### Phase 1: Foundation Setup (1-2 hours)
- [ ] Add tree-sitter-nix dependency
- [ ] Update Language enum and all methods
- [ ] Create nix module structure
- [ ] Implement basic NixLanguage definition
- [ ] Update registry integration

### Phase 2: Core Parser Implementation (4-6 hours)  
- [ ] Implement NixParser struct and new() method
- [ ] Implement LanguageParser trait
- [ ] Add symbol extraction for basic constructs (variables, functions)
- [ ] Add attribute set extraction
- [ ] Handle Nix-specific node types

### Phase 3: Behavior Implementation (2-3 hours)
- [ ] Implement NixBehavior struct
- [ ] Implement LanguageBehavior trait methods
- [ ] Add Nix-specific formatting and signature generation
- [ ] Handle scope resolution basics

### Phase 4: Resolution Context (2-3 hours)
- [ ] Implement NixResolutionContext
- [ ] Handle let-in scoping
- [ ] Handle with expression scoping  
- [ ] Handle attribute set scoping
- [ ] Implement NixInheritanceResolver

### Phase 5: Testing & Validation (2-3 hours)
- [ ] Create comprehensive unit test suite
- [ ] Add integration tests with real Nix files
- [ ] Verify symbol extraction accuracy
- [ ] Performance testing and optimization
- [ ] Documentation updates

**Total Estimated Effort: 11-17 hours**

## Verification Checklist

### Core Implementation
- [ ] NixParser compiles and instantiates without errors
- [ ] LanguageParser trait fully implemented  
- [ ] NixBehavior implements all LanguageBehavior methods
- [ ] NixResolutionContext handles Nix scoping rules
- [ ] Registry integration registers Nix language correctly

### Language Support
- [ ] .nix files detected and parsed correctly
- [ ] Variable bindings extracted as symbols
- [ ] Function definitions extracted with proper signatures
- [ ] Attribute sets extracted and indexed
- [ ] Complex Nix constructs (let-in, with, rec) handled correctly

### Integration Testing
- [ ] `cargo build` succeeds with all new code
- [ ] `cargo test nix` passes all unit tests  
- [ ] Integration tests pass with real Nix files
- [ ] MCP server exposes Nix symbols correctly
- [ ] Symbol search and filtering works for Nix

### Quality Assurance  
- [ ] `cargo clippy` passes without warnings
- [ ] `cargo fmt` formatting is correct
- [ ] Documentation is complete and accurate
- [ ] Performance meets expectations (>10,000 symbols/second)
- [ ] Memory usage is reasonable (<100 bytes/symbol)

### End-to-End Verification

```bash
# 1. Build with new Nix support
cargo build --release

# 2. Test with real Nix file
echo 'let x = 42; f = y: y + x; in { inherit x f; }' > test.nix

# 3. Run codanna indexing
./target/release/codanna index test.nix

# 4. Verify symbol extraction  
./target/release/codanna find-symbol "x" --lang nix
./target/release/codanna find-symbol "f" --lang nix

# 5. Expected output:
# - Symbol 'x' found as Variable
# - Symbol 'f' found as Function  
# - Attribute set symbols indexed correctly
```

### Success Criteria

1. **Functional**: All Nix language constructs parsed and symbols extracted correctly
2. **Performance**: Parsing speed >10,000 symbols/second on typical Nix files
3. **Integration**: Seamless integration with existing Codanna MCP server functionality
4. **Quality**: Zero clippy warnings, comprehensive test coverage >90%
5. **Documentation**: Complete API documentation and usage examples

## Conclusion

This implementation guide provides comprehensive specifications for adding Nix language support to Codanna. The approach follows established architectural patterns, leverages mature tree-sitter-nix parsing capabilities, and includes thorough testing strategies.

Key success factors:
- **Incremental Development**: Phased implementation with clear milestones
- **Pattern Consistency**: Following existing Go/Rust language implementation patterns  
- **Comprehensive Testing**: Unit, integration, and performance test coverage
- **Quality Assurance**: Strict adherence to Rust best practices and Codanna coding standards

The implementation should result in full Nix language support enabling Codanna users to perform intelligent code analysis on Nix expressions, package definitions, and NixOS configurations.