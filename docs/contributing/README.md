[Documentation](../README.md) / **Contributing**

---

# Contributing

Contributions welcome! This section covers development setup and guidelines.

## In This Section

- **[Development](development.md)** - Development environment setup
- **[Adding Languages](adding-languages.md)** - How to add new language parsers
- **[Testing](testing.md)** - Test infrastructure and guidelines

## Quick Start for Contributors

1. Fork the repository
2. Clone your fork
3. Build the project:
   ```bash
   cargo build --release
   ```
4. Run tests:
   ```bash
   cargo test
   ```

## Development Commands

```bash
# Build the project
cargo build --release

# Run tests
cargo test

# Build and run in development mode
cargo run -- <command>
```

## Guidelines

See [CONTRIBUTING.md](../../CONTRIBUTING.md) in the root for detailed contribution guidelines.

## Adding Language Support

When adding new language support:
1. Implement the parser trait
2. Add language-specific resolution if needed
3. Include comprehensive tests
4. Update documentation

## Next Steps

- Read the main [CONTRIBUTING.md](../../CONTRIBUTING.md)
- Explore the [Architecture](../architecture/) to understand internals
- Check [User Guide](../user-guide/) to understand usage patterns

[Back to Documentation](../README.md)