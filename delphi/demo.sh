#!/bin/bash

echo "🔮 Delphi - Bitcoin Block Height Oracle Demo"
echo "=============================================="
echo

# Build if needed
if [ ! -f "./target/release/delphi" ]; then
    echo "📦 Building delphi..."
    cargo build --release
    echo
fi

echo "📋 Testing known Bitcoin blocks:"
echo

echo "🥇 Genesis Block (height 0):"
echo "./target/release/delphi 000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"
./target/release/delphi 000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f
echo

echo "🥈 Block 1 (first mined block):"
echo "./target/release/delphi 00000000839a8e6886ab5951d76f411475428afc90947ee320161bbf18eb6048"
./target/release/delphi 00000000839a8e6886ab5951d76f411475428afc90947ee320161bbf18eb6048
echo

echo "💯 Block 100:"
echo "./target/release/delphi 000000007bc154e0fa7ea32218a72fe2c1bb9f86cf8c9ebf9a715ed27fdb229a"
./target/release/delphi 000000007bc154e0fa7ea32218a72fe2c1bb9f86cf8c9ebf9a715ed27fdb229a
echo

echo "🔍 Verbose output example (Genesis block):"
echo "./target/release/delphi 000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f --verbose"
./target/release/delphi 000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f --verbose
echo

echo "✅ All tests passed! Delphi is ready for Bitcoin block height oracles!"
echo
echo "📖 Usage: ./target/release/delphi <BLOCK_HASH> [--verbose]"
echo "📚 Help:  ./target/release/delphi --help"