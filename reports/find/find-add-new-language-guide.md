# Find Report: Adding a New Language to Codanna

**Generated**: August 22, 2025 at 10:14 PM  
**Original Query**: "How to add a new language"  
**Optimized Query**: "language parser implementation registration architecture"

## Summary

Codanna uses a sophisticated modular language registry system that enables self-registering language parsers. Adding a new language requires implementing 5 specific files in a dedicated subdirectory, registering with the global registry, and following established patterns for symbol extraction, relationship tracking, and language-specific behaviors. The architecture is mature and production-ready, supporting 5 languages currently (Rust, Python, TypeScript, PHP, Go).

## Key Findings

### Primary Discoveries
- **Language Registry System**: Self-registering modular architecture at `src/parsing/registry.rs:354`
- **Directory Structure Pattern**: Each language needs exactly 5 files in `src/parsing/{language}/`
- **Registration Process**: Languages register via `initialize_registry()` function at `src/parsing/registry.rs:368`
- **Template Implementation**: Rust language serves as reference at `src/parsing/rust/definition.rs:14`

### Code Locations
| Component | File | Line | Purpose |
|-----------|------|------|---------|
| Language Registry | `src/parsing/registry.rs` | 163 | Core registry managing all languages |
| Registry Initialization | `src/parsing/registry.rs` | 368 | Where new languages get registered |
| Language Definition Trait | `src/parsing/registry.rs` | 126 | Interface all languages must implement |
| Rust Example | `src/parsing/rust/definition.rs` | 14 | Complete language implementation example |
| Language Support Guide | `contributing/development/language-support.md` | 1 | Detailed implementation guide |

## Step-by-Step Implementation Guide

### Step 1: Create Language Directory Structure

Create exactly 5 files in `src/parsing/{your_language}/`:

```
src/parsing/{language}/
├── mod.rs        # Module re-exports and public API
├── parser.rs     # Symbol extraction, calls, implementations, imports
├── behavior.rs   # Module paths, visibility, basic language behaviors  
├── resolution.rs # Language-specific symbol resolution logic
└── definition.rs # Language ID, extensions, factory methods
```

### Step 2: Implement LanguageDefinition

Start with `definition.rs` based on the Rust example:

```rust
//! {Language} language definition for the registry

use std::sync::Arc;
use super::{YourLanguageBehavior, YourLanguageParser};
use crate::parsing::{LanguageBehavior, LanguageDefinition, LanguageId, LanguageParser};
use crate::{IndexResult, Settings};

pub struct YourLanguage;

impl YourLanguage {
    pub const ID: LanguageId = LanguageId::new("your_language");
}

impl LanguageDefinition for YourLanguage {
    fn id(&self) -> LanguageId { Self::ID }
    fn name(&self) -> &'static str { "Your Language" }
    fn extensions(&self) -> &'static [&'static str] { &["ext1", "ext2"] }
    
    fn create_parser(&self, settings: &Settings) -> IndexResult<Box<dyn LanguageParser>> {
        let parser = YourLanguageParser::new()?;
        Ok(Box::new(parser))
    }
    
    fn create_behavior(&self) -> Box<dyn LanguageBehavior> {
        Box::new(YourLanguageBehavior::new())
    }
    
    fn default_enabled(&self) -> bool { false }
}

pub(crate) fn register(registry: &mut crate::parsing::LanguageRegistry) {
    registry.register(Arc::new(YourLanguage));
}
```

### Step 3: Implement Required Traits

- **LanguageParser**: Extract symbols, calls, implementations, imports
- **LanguageBehavior**: Handle module paths, visibility, resolution
- **ResolutionScope**: Language-specific symbol resolution logic

Key implementation requirements:
- Extract complete signatures for all symbol types
- Support relationship tracking (calls, implements, defines)
- Handle language-specific visibility and scoping rules
- Process documentation comments for semantic search

### Step 4: Add Tree-sitter Dependency

Add to `Cargo.toml`:
```toml
tree-sitter-{your_language} = "0.x"
```

### Step 5: Register with Global Registry

Add your language to `src/parsing/registry.rs` at line 378:

```rust
fn initialize_registry(registry: &mut LanguageRegistry) {
    // Existing languages...
    super::your_language::register(registry);
}
```

### Step 6: Update Module Exports

Add to `src/parsing/mod.rs`:
```rust
pub mod your_language;
pub use your_language::{YourLanguageBehavior, YourLanguageParser};
```

## Notable Findings

### Interesting Patterns
- **Self-Registration Architecture**: Languages automatically register themselves on first registry access using lazy initialization
- **Extension-Based Routing**: File extensions automatically map to language parsers without manual configuration
- **Settings Integration**: Each language can be enabled/disabled via `.codanna/settings.toml` without code changes
- **Performance Focus**: All operations target >10,000 symbols/second with ~100 bytes per symbol memory usage

### Code Quality Observations  
- **Excellent Separation of Concerns**: Registry handles discovery, parsers handle extraction, behaviors handle language-specific logic
- **Type Safety**: Uses `LanguageId` newtype and compile-time constants for zero-cost abstractions
- **Comprehensive Error Handling**: Registry errors include actionable suggestions for configuration issues
- **Test Coverage**: Each language has dedicated test files and integration tests

## Claude's Assessment

### Honest Feedback
The language registration system is exceptionally well-designed with clear separation of concerns and excellent extensibility. The modular architecture allows adding new languages without modifying core code, and the self-registration pattern eliminates manual configuration. The trait-based design enables both generic operations and language-specific customization.

### Strengths of Current Implementation
- **Zero-cost Abstractions**: Uses static strings and compile-time constants
- **Runtime Configurability**: Languages can be enabled/disabled without recompilation
- **Comprehensive API**: Covers symbol extraction, relationship tracking, and documentation
- **Performance Oriented**: Targets aggressive performance metrics and achieves them
- **Production Ready**: Successfully supports 5 diverse languages with real-world usage

### Recommendations
- **For developers**: Follow the established 5-file pattern exactly - it's been battle-tested
- **For architecture**: Consider ABI-15 exploration phase before implementing (as mentioned in guide)
- **For maintenance**: The registry system handles complexity well, focus implementation effort on parser accuracy

## Search Journey

### Query Evolution
1. Original: "How to add a new language"
2. Optimized: "language parser implementation registration architecture"

The semantic search initially returned irrelevant Go example files, but targeted grep searches for "register.*language|LanguageDefinition" immediately found all relevant files.

### Search Results Quality
- Semantic search effectiveness: Low (found examples instead of implementation)
- Full-text search needed: Yes (grep was much more effective)
- Total relevant results found: 15 files containing the core architecture

## Related Areas

### Connected Components
- Symbol extraction patterns in existing language parsers
- Test frameworks in `tests/test_language_regression.rs`
- Configuration handling in `src/config.rs`
- MCP server integration for new language tooling

### Follow-up Questions
- What specific tree-sitter node patterns does your target language use?
- How does your language handle scoping and visibility rules?
- What performance characteristics can you achieve with your parser implementation?
- Do you need custom resolution logic or can you use default implementations?

## Performance Requirements

All language implementations must meet these targets:
- **Parsing Speed**: >10,000 symbols/second
- **Memory Usage**: ~100 bytes per symbol including metadata
- **Search Latency**: <10ms for semantic search operations
- **Startup Time**: <300ms for all CLI operations

Example benchmarks from existing languages:
- **Rust**: 91,318 symbols/sec (9.1x faster than target)
- **Python**: 75,047 symbols/sec (7.5x faster than target)  
- **TypeScript**: 82,156 symbols/sec (8.2x faster than target)
- **Go**: 74,655 symbols/sec (7.5x faster than target)

## Implementation Checklist

- [ ] Create 5-file directory structure
- [ ] Implement LanguageDefinition trait
- [ ] Implement LanguageParser with symbol extraction
- [ ] Implement LanguageBehavior with language-specific logic
- [ ] Implement ResolutionScope for symbol resolution
- [ ] Add tree-sitter dependency to Cargo.toml
- [ ] Register with global registry
- [ ] Update module exports
- [ ] Write comprehensive tests
- [ ] Run performance benchmarks
- [ ] Validate against existing language patterns

---

*This report was generated using the `/find` command workflow.*
*Claude version: Sonnet 4*