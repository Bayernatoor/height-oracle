# Delphi - Bitcoin Block Height Oracle CLI (Embedded)

A lightning-fast command-line tool for Bitcoin block height lookups using perfect hash functions with embedded oracle data.

## Features

- **Ultra-fast lookups**: 2.5M+ lookups per second
- **Pre-BIP34 coverage**: Complete coverage of v1 blocks 0-227,930
- **Self-contained**: No external files needed - oracle data embedded in binary
- **Ultra-lightweight**: Only 860 KiB self-contained binary
- **Simple CLI**: Manual argument parsing for minimal overhead

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
./target/release/delphi 00000000d1145790a8694403d4063f323d499e655c83426834d4ce2f8dd4a2ee
# Output: 170

# ⚠️ Warning: Non v1-blocks are not in the perfect hasmap and guessing their height WILL BE WRONG! 
./target/release/delphi 000000000000000002cce816c0ab2c5c269cb081896b7dcb34b8422d6b74ffa1 # actual: 420,000
# Output: 184468
```

### No External Files Needed

The embedded version includes all oracle data in the binary itself. No need for external asset files!

## Performance

- **Load time**: Instant (embedded data)
- **Lookup time**: ~400μs per lookup
- **Memory usage**: Self-contained in binary

## Prerequisites

The oracle data is embedded directly in the binary. Build requirements:

```bash
# First, generate the oracle assets (see parent ../README.md)
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