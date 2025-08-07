# Height Oracle - Ultra-Efficient Bitcoin Block Height Oracle

**BTC++ Riga 2025 hackathon submission**

A lightning-fast Rust library for Bitcoin block height lookups using perfect hash functions, optimized for pre-BIP34 blocks (0-227,930).

## 🚀 Features

- **Perfect Hash Functions**: Zero-collision lookups with minimal memory
- **18-bit Height Compression**: Custom packing algorithm for memory efficiency  
- **Zero-Copy Deserialization**: Using `epserde` for ultra-fast loading
- **Feature-Gated Design**: `generate` (building) and `embedded` (runtime) modes
- **Self-Contained CLI**: 908KB binary with embedded oracle data

## 📦 Components

### Library (`height-oracle`)
Core Rust library with perfect hash implementation for Bitcoin block height lookups.

### CLI Tool (`delphi`)
Ultra-minimal command-line tool for instant height lookups.
**Usage documentation**: See [`delphi/README.md`](delphi/README.md)

## 🎯 Performance

- **Lookup Speed**: 2.5M+ lookups per second
- **Memory Efficiency**: 18-bit compressed heights
- **Binary Size**: 908KB self-contained executable
- **Load Time**: Instant (embedded data)

## 🏗️ Quick Start

```bash
# Build the oracle data
cargo run --features generate --release

# Build the CLI tool
cd delphi
cargo build --release

# Test a lookup (Genesis block)
./target/release/delphi 000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f
# Output: 0
```

## 👥 Hackathon Team

- [@ubbabeck](https://github.com/ubbabeck)
- [@bayernator](https://github.com/bayernator)  
- [@FelixWeis](https://github.com/FelixWeis)
- Claude (AI Assistant) - Code architecture, optimization, and implementation
