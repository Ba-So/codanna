# Parser Resolution API Reference

## Overview

All language parsers in codanna follow a consistent API pattern for symbol resolution and relationship tracking. This document defines the core contracts and extension patterns.

## Core Components

### 1. ResolutionContext

Each language implements a resolution context that manages symbol scopes and visibility.

#### Required Trait: `ResolutionScope`

```rust
pub trait ResolutionScope {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn add_symbol(&mut self, name: String, symbol_id: SymbolId, scope_level: ScopeLevel);
    fn resolve(&self, name: &str) -> Option<SymbolId>;
    fn clear_local_scope(&mut self);
    fn enter_scope(&mut self, scope_type: ScopeType);
    fn exit_scope(&mut self);
    fn symbols_in_scope(&self) -> Vec<(String, SymbolId, ScopeLevel)>;
    fn resolve_relationship(&self, from_name: &str, to_name: &str, kind: RelationKind, from_file: FileId) -> Option<SymbolId>;
}
```

#### Base Implementation Pattern

All resolution contexts share this structure:

```rust
pub struct LanguageResolutionContext {
    file_id: FileId,
    local_scope: HashMap<String, SymbolId>,      // Function/block locals
    module_symbols: HashMap<String, SymbolId>,    // File-level definitions
    imported_symbols: HashMap<String, SymbolId>,  // Imported/included symbols
    global_symbols: HashMap<String, SymbolId>,    // Global/ambient symbols
    scope_stack: Vec<ScopeType>,                  // Nested scope tracking
}
```

#### Standard Scope Levels

- `Local`: Function parameters, local variables
- `Module`: File-level definitions
- `Package`: Imported symbols from other modules
- `Global`: System-wide visible symbols

### 2. Language-Specific Extensions

Languages extend the base pattern with zero-cost helper methods for their unique features.

#### TypeScript Extensions

```rust
// Additional scopes for TypeScript features
hoisted_scope: HashMap<String, SymbolId>,    // Functions and var declarations
type_space: HashMap<String, SymbolId>,        // Type-only symbols

// Helper methods
pub fn add_symbol_with_context(&mut self, name: String, symbol_id: SymbolId, context: Option<&ScopeContext>)
pub fn add_import_symbol(&mut self, name: String, symbol_id: SymbolId, is_type_only: bool)
```

**Resolution Order**: local → hoisted → imported → module → type_space → global

#### Rust Extensions

```rust
// Additional scopes for Rust features
crate_symbols: HashMap<String, SymbolId>,     // Crate-level public items

// Helper methods
pub fn add_local(&mut self, name: String, symbol_id: SymbolId)
pub fn add_crate_symbol(&mut self, name: String, symbol_id: SymbolId)
```

**Resolution Order**: local → imported → module → crate → global

#### C++ Extensions

```rust
// Additional features for C++
using_directives: Vec<String>,                     // using namespace ...
using_declarations: HashMap<String, SymbolId>,     // using std::vector
inheritance_graph: HashMap<SymbolId, Vec<SymbolId>>, // Class inheritance

// Helper methods
pub fn add_using_directive(&mut self, namespace: String)
pub fn add_using_declaration(&mut self, name: String, symbol_id: SymbolId)
pub fn add_inheritance(&mut self, derived: SymbolId, base: Vec<SymbolId>)
```

**Resolution Order**: local → using_declarations → module → imported → using_directives → global

### 3. InheritanceResolver

Tracks inheritance and trait relationships specific to each language.

```rust
pub trait InheritanceResolver {
    fn add_extends(&mut self, type_name: String, parent_name: String);
    fn add_implements(&mut self, type_name: String, interface_name: String);
    fn get_parent(&self, type_name: &str) -> Option<&str>;
    fn get_interfaces(&self, type_name: &str) -> Vec<&str>;
    fn is_parent_of(&self, parent: &str, child: &str) -> bool;
}
```

### 4. LanguageBehavior Integration

Each language behavior must provide:

```rust
fn create_resolution_context(&self, file_id: FileId) -> Box<dyn ResolutionScope> {
    Box::new(LanguageResolutionContext::new(file_id))
}

fn create_inheritance_resolver(&self) -> Box<dyn InheritanceResolver> {
    Box::new(LanguageInheritanceResolver::new())
}
```

## Design Principles

### Zero-Cost Abstractions

- Use borrowed strings (`&str`) in resolve operations
- Return `Option<SymbolId>` without allocation
- Helper methods are inlined for specific language features
- No virtual dispatch in hot paths

### Separation of Concerns

- **Parser**: Extracts symbols and relationships from AST
- **ResolutionContext**: Manages symbol visibility and scoping
- **InheritanceResolver**: Tracks type relationships
- **LanguageBehavior**: Orchestrates resolution with language rules

### Progressive Enhancement

Languages can use the generic implementation if they don't need special features:

```rust
// Simple languages (Go, PHP) use generic
Box::new(GenericResolutionContext::new(file_id))

// Complex languages use specific implementations
Box::new(TypeScriptResolutionContext::new(file_id))
```

## Import/Include Handling

All languages track their import mechanisms:

| Language | Mechanism | Tracking Method |
|----------|-----------|-----------------|
| Rust | `use` statements | `add_import()` + `add_import_symbol()` |
| TypeScript | `import` statements | `add_import()` + `add_import_symbol(is_type_only)` |
| Python | `import`/`from` | `add_import()` + module path tracking |
| Go | `import` declarations | Package-level symbols |
| C/C++ | `#include` directives | `add_include()` + header symbols |
| PHP | `use`/`namespace` | Namespace tracking |

## Scope Management

### Enter/Exit Patterns

```rust
// Parser calls when entering a new scope
context.enter_scope(ScopeType::Function { hoisting: false });

// Process symbols in scope...

// Clear and exit
context.exit_scope();  // Automatically clears locals if appropriate
```

### Scope Types

- `Global`: Top-level file scope
- `Module`: Module/namespace scope
- `Function { hoisting: bool }`: Function body scope
- `Block`: Block scope (if/for/etc)
- `Class`: Class body scope

## Relationship Resolution

The `resolve_relationship()` method handles cross-references:

```rust
match kind {
    RelationKind::Extends => // Resolve parent class/interface
    RelationKind::Implements => // Resolve implemented interface
    RelationKind::Calls => // Resolve function/method call
    RelationKind::Uses => // Resolve type usage
    RelationKind::Imports => // Resolve import target
}
```

## Implementation Checklist

When implementing a new language parser:

1. **Decide complexity level**:
   - Simple scoping → Use `GenericResolutionContext`
   - Complex scoping → Create `LanguageResolutionContext`

2. **If creating specific context**:
   - Define language-specific scopes
   - Implement `ResolutionScope` trait
   - Define resolution order in `resolve()`
   - Add helper methods for language features

3. **Wire to behavior**:
   - Override `create_resolution_context()`
   - Override `create_inheritance_resolver()` if needed

4. **Handle imports**:
   - Track import statements during parsing
   - Call appropriate `add_import*()` methods
   - Resolve imported symbols to SymbolIds

## Testing Resolution

Each resolution implementation should test:

- Symbol visibility at different scope levels
- Shadowing behavior
- Import resolution
- Inheritance chain resolution
- Qualified name resolution (e.g., `Class.method`)

## Performance Considerations

- Resolution happens during indexing, not at query time
- Symbol maps are built once per file
- Use `&str` for lookups to avoid allocations
- Cache resolution results when possible
- Clear scopes promptly to minimize memory usage

## Common Pitfalls

1. **Not clearing local scope**: Memory leak in long files
2. **Wrong resolution order**: Shadows or missing symbols
3. **Missing scope tracking**: Incorrect visibility
4. **Not handling qualified names**: `Class.method` patterns
5. **Ignoring language-specific features**: Hoisting, namespaces, etc.

## Current Implementation Status

| Language | Resolution Type | Inheritance | Import Tracking | Status |
|----------|----------------|-------------|-----------------|---------|
| TypeScript | TypeScriptResolutionContext | ✅ | ✅ | Production |
| Rust | RustResolutionContext | ✅ | ✅ | Production |
| Python | GenericResolutionContext | ✅ | ✅ | Production |
| Go | GenericResolutionContext | ✅ | ✅ | Production |
| PHP | GenericResolutionContext | ✅ | ✅ | Production |
| C | CResolutionContext | ❌ | ✅ | Needs Integration |
| C++ | CppResolutionContext | ✅ | ✅ | Needs Integration |