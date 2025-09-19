# Performance Tests

## Running Performance Tests

Performance tests are marked with `#[ignore]` to prevent them from running in CI where hardware variability can cause false failures.

### Run all tests including performance tests:
```bash
cargo test -- --include-ignored
```

### Run only performance tests:
```bash
cargo test -- --ignored
```

### Run specific performance test:
```bash
cargo test test_parser_performance_benchmark -- --ignored
```

## Performance Targets

| Parser | Target | Measurement |
|--------|--------|-------------|
| Rust | 10,000 symbols/sec | Stable |
| Python | 10,000 symbols/sec | Stable |
| TypeScript | 10,000 symbols/sec | Stable |
| Go | 10,000 symbols/sec | Variable in CI |
| PHP | 10,000 symbols/sec | Stable |

## Why Performance Tests Are Ignored in CI

1. **Hardware Variability** - GitHub Actions runners have inconsistent performance
2. **False Failures** - Tests may fail due to runner load, not code issues
3. **Stability** - CI should test correctness, not performance on shared hardware

## Best Practice

- Run performance tests locally before releases
- Use `cargo bench` for detailed performance analysis
- Monitor trends over time, not absolute values in CI