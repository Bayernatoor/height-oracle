//! # Height Oracle
//!
//! A Rust library for ultra-efficient Bitcoin block height lookups using perfect hash functions.
//! Maps `BlockHash` → `height` for all pre-BIP34 blocks (0 to 227,930) with ~3.35 bits/element storage efficiency.

// Core types and constants
pub type BlockHash = [u8; 32]; // Network byte order
pub const BIP34_ACTIVATION_HEIGHT: u32 = 227_931;

// PtrHash type configuration
pub type PtrHashType =
    ptr_hash::DefaultPtrHash<ptr_hash::hash::FxHash, BlockHash, ptr_hash::bucket_fn::CubicEps>;

// Import always-available modules
pub mod packing;

// Feature-gated modules
#[cfg(feature = "generate")]
pub mod generate;

#[cfg(feature = "embedded")]
pub mod embedded;

// Re-exports based on features
#[cfg(feature = "generate")]
pub use generate::{HeightOracle, HeightOracleLoaded, MemoryStats};

#[cfg(feature = "embedded")]
pub use embedded::{guess_height_prebip34block_unchecked, HeightOracleEmbedded};

/// Parse a Bitcoin block hash from hex string to network byte order
///
/// Bitcoin uses reverse hex format, so this function:
/// 1. Validates the hex string (64 characters)
/// 2. Parses hex to bytes
/// 3. Reverses bytes to get network byte order
pub fn parse_block_hash(hex_str: &str) -> Result<BlockHash, String> {
    // Remove 0x prefix if present
    let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);

    // Validate 64 hex characters exactly
    if hex_str.len() != 64 {
        return Err("Block hash must be exactly 64 hex characters".to_string());
    }

    // Parse hex to bytes
    let mut bytes = [0u8; 32];
    for (i, chunk) in hex_str.as_bytes().chunks(2).enumerate() {
        let hex_byte = std::str::from_utf8(chunk).map_err(|_| "Invalid UTF-8")?;
        bytes[i] = u8::from_str_radix(hex_byte, 16).map_err(|_| "Invalid hex")?;
    }

    // CRITICAL: Bitcoin uses reverse hex, so reverse to get network byte order
    bytes.reverse();
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_block_hash() {
        // Test genesis block hash
        let hex = "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f";
        let result = parse_block_hash(hex).unwrap();

        // Should be 32 bytes
        assert_eq!(result.len(), 32);

        // Test with 0x prefix
        let hex_with_prefix = "0x000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f";
        let result_with_prefix = parse_block_hash(hex_with_prefix).unwrap();
        assert_eq!(result, result_with_prefix);

        // Test invalid length
        assert!(parse_block_hash("123").is_err());

        // Test invalid hex
        assert!(parse_block_hash(
            "gggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggg"
        )
        .is_err());
    }

    #[test]
    fn test_block_hash_reverse() {
        // Simple test to verify reverse behavior
        let hex = "0000000000000000000000000000000000000000000000000000000000000001";
        let result = parse_block_hash(hex).unwrap();

        // After parsing and reversing, the last byte should be 1
        assert_eq!(result[31], 1);
        assert_eq!(result[0], 0);
    }
}
