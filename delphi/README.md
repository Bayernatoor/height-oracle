# Delphi - Bitcoin Block Height Oracle CLI (Embedded)

A lightning-fast command-line tool for Bitcoin block height lookups using perfect hash functions with embedded oracle data.

## Features

- **Ultra-fast lookups**: 2.5M+ lookups per second
- **Pre-BIP34 coverage**: Complete coverage of blocks 0-227,930
- **Self-contained**: No external files needed - oracle data embedded in binary
- **Ultra-lightweight**: Only 1.1MB binary (no clap, no anyhow dependencies)
- **Simple CLI**: Manual argument parsing for minimal overhead
- **Verbose mode**: Detailed information and performance metrics

## Installation

```bash
cd delphi
cargo build --release
```

## Usage

### Basic Usage

```bash
# Simple height lookup
./target/release/delphi 000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f
# Output: 0

# Verbose output (shows embedded data info)
./target/release/delphi 000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f --verbose
```

### Examples

```bash
# Genesis block
./target/release/delphi 000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f
# Output: 0

# Block 1
./target/release/delphi 00000000839a8e6886ab5951d76f411475428afc90947ee320161bbf18eb6048
# Output: 1

# Block 100 (with verbose output)
./target/release/delphi 000000007bc154e0fa7ea32218a72fe2c1bb9f86cf8c9ebf9a715ed27fdb229a --verbose
```

### No External Files Needed

The embedded version includes all oracle data in the binary itself. No need for external asset files!

## Options

- `BLOCK_HASH`: Bitcoin block hash in hex format (with or without 0x prefix)
- `--verbose, -v`: Show detailed information about the lookup
- `--help, -h`: Show help information

## Performance

- **Load time**: Instant (embedded data)
- **Lookup time**: ~400μs per lookup  
- **Binary size**: **1.1MB** (includes all oracle data) 
- **Stripped size**: **1.0MB** (without debug symbols)
- **Memory usage**: Self-contained in binary

## Prerequisites

The oracle data is embedded directly in the binary. Build requirements:

```bash
# First, generate the oracle assets (needed for embedding)
cd ..
cargo run --features generate --release

# Then build the embedded version
cd delphi
cargo build --release
```

The binary will include all oracle data and requires no external files.

## Use Cases

- **Bitcoin archaeology**: Research pre-BIP34 blocks
- **Transaction analysis**: Determine block heights for historical transactions
- **Scripting**: Batch processing of block hashes
- **Development**: Integration into Bitcoin-related tools

## Exit Codes

- `0`: Success
- `1`: Error (invalid hash, missing files, etc.)