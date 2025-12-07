# Installation

Detailed installation instructions for all platforms.

## Requirements

- Rust 1.75+ (for development)
- ~150MB for model storage (downloaded on first use)
- A few MB for index storage (varies by codebase size)

## Install from Crates.io

```bash
cargo install codanna --all-features
```

## System Dependencies

### Linux (Ubuntu/Debian)
```bash
sudo apt update && sudo apt install pkg-config libssl-dev
```

### Linux (CentOS/RHEL)
```bash
sudo yum install pkgconfig openssl-devel
```

### Linux (Fedora)
```bash
sudo dnf install pkgconfig openssl-devel
```

### macOS
No additional dependencies required.

## Verify Installation

After installation, verify Codanna is working:

```bash
# Check version
codanna --version

# Initialize configuration
codanna init

# Test MCP connection (for AI assistant integration)
codanna mcp-test
```

## Build from Source

If you prefer to build from source:

```bash
# Clone the repository
git clone https://github.com/anthropics/codanna.git
cd codanna

# Build with all features
cargo build --release --all-features

# Binary will be at target/release/codanna
```

## Development Setup

For development:

```bash
# Build the project
cargo build --release

# Run tests
cargo test

# Build and run in development mode
cargo run -- <command>
```

## Troubleshooting

### Linux: Missing pkg-config
If you see errors about pkg-config, install the system dependencies listed above for your distribution.

### Model Download
The embedding model (~150MB) downloads automatically on first use. Ensure you have a stable internet connection for the initial download.

## Next Steps

- Continue with [First Index](first-index.md) to create your first code index
- See [Configuration](../user-guide/configuration.md) for customization options
- Set up [Integrations](../integrations/) with your AI assistant