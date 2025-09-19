//! Benchmarks for Nix parsing performance
//!
//! This benchmark suite measures the performance of Nix symbol extraction
//! to ensure we meet the target of >10,000 symbols/second.

use codanna::FileId;
use codanna::parsing::nix::NixParser;
use codanna::types::SymbolCounter;
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use std::fs;

/// Load real Nix files for benchmarking
fn load_benchmark_files() -> Vec<(String, String)> {
    let fixtures_dir = std::path::Path::new("tests/fixtures/nix");
    let mut files = Vec::new();

    if fixtures_dir.exists() {
        for entry in fs::read_dir(fixtures_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("nix") {
                let name = path.file_name().unwrap().to_str().unwrap().to_string();
                if let Ok(content) = fs::read_to_string(&path) {
                    files.push((name, content));
                }
            }
        }
    }

    // If no files found, create synthetic ones
    if files.is_empty() {
        files.push(("synthetic_small.nix".to_string(), generate_nix_code(100)));
        files.push(("synthetic_medium.nix".to_string(), generate_nix_code(500)));
        files.push(("synthetic_large.nix".to_string(), generate_nix_code(1000)));
    }

    files
}

/// Generate synthetic Nix code for benchmarking
fn generate_nix_code(num_bindings: usize) -> String {
    let mut code = String::from("{\n");

    for i in 0..num_bindings {
        match i % 5 {
            0 => {
                // Simple variable
                code.push_str(&format!("  var{} = \"value{}\";\n", i, i));
            }
            1 => {
                // Function definition
                code.push_str(&format!("  func{} = x: y: x + y + {};\n", i, i));
            }
            2 => {
                // Attribute set
                code.push_str(&format!(
                    "  obj{} = {{ name = \"obj{}\"; value = {}; enabled = true; }};\n",
                    i, i, i
                ));
            }
            3 => {
                // List with interpolation
                code.push_str(&format!(
                    "  list{} = [ {} \"item-${{toString {}}}\" (func{} {} {}) ];\n",
                    i,
                    i,
                    i,
                    i % 10,
                    i,
                    i + 1
                ));
            }
            4 => {
                // Complex expression with let-in
                code.push_str(&format!(
                    "  complex{} = let x = {}; y = x * 2; in {{ inherit x y; sum = x + y; }};\n",
                    i, i
                ));
            }
            _ => unreachable!(),
        }
    }

    code.push_str("}\n");
    code
}

/// Benchmark basic symbol extraction
fn bench_symbol_extraction(c: &mut Criterion) {
    let files = load_benchmark_files();

    let mut group = c.benchmark_group("nix_symbol_extraction");
    group.sample_size(50);

    for (name, content) in files {
        let symbol_count = {
            let mut parser = NixParser::new().unwrap();
            let mut counter = SymbolCounter::new();
            let file_id = FileId(1);
            parser.parse(&content, file_id, &mut counter).len()
        };

        group.bench_with_input(
            BenchmarkId::new("parse", format!("{} ({} symbols)", name, symbol_count)),
            &content,
            |b, content| {
                let mut parser = NixParser::new().unwrap();
                b.iter(|| {
                    let mut counter = SymbolCounter::new();
                    let file_id = FileId(1);
                    black_box(parser.parse(content, file_id, &mut counter))
                });
            },
        );
    }

    group.finish();
}

/// Benchmark parser creation
fn bench_parser_creation(c: &mut Criterion) {
    c.bench_function("nix_parser_creation", |b| {
        b.iter(|| black_box(NixParser::new().unwrap()));
    });
}

/// Benchmark throughput with different file sizes
fn bench_throughput(c: &mut Criterion) {
    let sizes = [100, 500, 1000, 2000, 5000];

    let mut group = c.benchmark_group("nix_throughput");
    group.sample_size(30);

    for size in sizes.iter() {
        let content = generate_nix_code(*size);

        // Measure actual symbol count
        let symbol_count = {
            let mut parser = NixParser::new().unwrap();
            let mut counter = SymbolCounter::new();
            let file_id = FileId(1);
            parser.parse(&content, file_id, &mut counter).len()
        };

        group.bench_with_input(
            BenchmarkId::new("symbols_per_second", format!("{}_bindings", size)),
            &content,
            |b, content| {
                let mut parser = NixParser::new().unwrap();
                b.iter_custom(|iters| {
                    let start = std::time::Instant::now();
                    let mut total_symbols = 0;

                    for _ in 0..iters {
                        let mut counter = SymbolCounter::new();
                        let file_id = FileId(1);
                        let symbols = parser.parse(content, file_id, &mut counter);
                        total_symbols += black_box(symbols.len());
                    }

                    let duration = start.elapsed();

                    // Calculate symbols per second
                    let symbols_per_second = (total_symbols as f64) / duration.as_secs_f64();
                    println!(
                        "Size {}: {:.0} symbols/second ({} symbols per iteration)",
                        size, symbols_per_second, symbol_count
                    );

                    duration
                });
            },
        );
    }

    group.finish();
}

/// Benchmark complex Nix constructs
fn bench_complex_constructs(c: &mut Criterion) {
    let mut group = c.benchmark_group("nix_complex_constructs");

    // Complex recursive attribute set
    let recursive_code = r#"
    rec {
      a = 1;
      b = a + 2;
      c = b * 3;
      d = c + a + b;
      e = d * (a + b + c);
      f = { inherit a b c d e; sum = a + b + c + d + e; };
      g = f.sum * 2;
      h = builtins.length [ a b c d e f g ];
    }
    "#;

    // Complex function with let-in
    let function_code = r#"
    {
      processData = { input, config ? {}, debug ? false }: let
        defaultConfig = {
          timeout = 30;
          retries = 3;
          batchSize = 100;
        };
        mergedConfig = defaultConfig // config;
        
        process = data: let
          chunks = lib.chunksOf mergedConfig.batchSize data;
          results = map (chunk: processChunk chunk mergedConfig) chunks;
        in flatten results;
        
        processChunk = chunk: cfg: let
          validated = filter (item: item != null) chunk;
          transformed = map (item: transformItem item cfg) validated;
        in transformed;
        
        transformItem = item: cfg: item // {
          processed = true;
          timestamp = builtins.currentTime;
          config = cfg;
        };
        
      in process input;
    }
    "#;

    // Complex string interpolation
    let interpolation_code = r#"
    let
      name = "myapp";
      version = "1.2.3";
      env = "production";
      
      config = ''
        [application]
        name = "${name}"
        version = "${version}"
        environment = "${env}"
        
        [server]
        host = "${if env == "production" then "prod.example.com" else "dev.example.com"}"
        port = ${toString (if env == "production" then 443 else 8080)}
        
        [database]
        url = "postgresql://${name}_${env}:${builtins.hashString "sha256" (name + version)}@db.example.com/mydb"
        
        [features]
        ${lib.concatMapStringsSep "\n" (f: "${f} = true") [ "feature1" "feature2" "feature3" ]}
      '';
      
    in { inherit config; }
    "#;

    let test_cases = vec![
        ("recursive_attribute_set", recursive_code),
        ("complex_function", function_code),
        ("string_interpolation", interpolation_code),
    ];

    for (name, code) in test_cases {
        group.bench_function(name, |b| {
            let mut parser = NixParser::new().unwrap();
            b.iter(|| {
                let mut counter = SymbolCounter::new();
                let file_id = FileId(1);
                black_box(parser.parse(code, file_id, &mut counter))
            });
        });
    }

    group.finish();
}

/// Benchmark memory usage patterns
fn bench_memory_usage(c: &mut Criterion) {
    let sizes = [100, 500, 1000];

    let mut group = c.benchmark_group("nix_memory_usage");

    for size in sizes.iter() {
        let content = generate_nix_code(*size);

        group.bench_with_input(
            BenchmarkId::new("parse_and_hold", format!("{}_symbols", size)),
            &content,
            |b, content| {
                b.iter(|| {
                    let mut parser = NixParser::new().unwrap();
                    let mut counter = SymbolCounter::new();
                    let file_id = FileId(1);
                    let symbols = parser.parse(content, file_id, &mut counter);

                    // Hold symbols in memory briefly to test memory patterns
                    let symbol_count = symbols.len();
                    let has_functions = symbols
                        .iter()
                        .any(|s| s.kind == codanna::SymbolKind::Function);

                    black_box((symbol_count, has_functions))
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_parser_creation,
    bench_symbol_extraction,
    bench_throughput,
    bench_complex_constructs,
    bench_memory_usage
);
criterion_main!(benches);
