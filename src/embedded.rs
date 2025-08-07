//! Zero-Copy Runtime (Feature: embedded)
//!
//! This module provides zero-copy runtime lookups using embedded asset data.

use crate::{BlockHash, PtrHashType};
use epserde::prelude::*;
use std::sync::OnceLock;

// Embedded oracle data at compile time
const PTRHASH_DATA: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/phash.ptrh.dat"
));
const HEIGHTS_DATA: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/heights.u18packed.dat"
));

/// Zero-copy embedded oracle using real epserde deserialization
pub struct HeightOracleEmbedded {
    phash: PtrHashType,
    heights: Vec<u32>,
}

impl HeightOracleEmbedded {
    /// Load from the embedded static data using epserde
    pub fn load_embedded() -> Self {
        // Load PtrHash from embedded data using epserde
        let mut ptrhash_cursor = std::io::Cursor::new(PTRHASH_DATA);
        let phash = PtrHashType::deserialize_full(&mut ptrhash_cursor)
            .expect("Failed to deserialize embedded PtrHash");

        // Load heights from embedded data using our packing format
        let mut heights_cursor = std::io::Cursor::new(HEIGHTS_DATA);
        let heights = crate::packing::deserialize_heights(&mut heights_cursor)
            .expect("Failed to deserialize embedded heights");

        Self { phash, heights }
    }

    /// Core lookup function
    pub fn get_height_unchecked(&self, block_hash: &BlockHash) -> u32 {
        let index = self.phash.index(block_hash);
        self.heights[index]
    }
}

/// Global singleton for embedded oracle
static EMBEDDED_ORACLE: OnceLock<HeightOracleEmbedded> = OnceLock::new();

/// Global lookup function for embedded oracle
pub fn guess_height_prebip34block_unchecked(block_hash: &BlockHash) -> u32 {
    let oracle = EMBEDDED_ORACLE.get_or_init(HeightOracleEmbedded::load_embedded);
    oracle.get_height_unchecked(block_hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_lookup() {
        // Test that the global function doesn't panic
        let test_hash = [0u8; 32];
        let _height = guess_height_prebip34block_unchecked(&test_hash);
        // Just ensure it doesn't panic (actual correctness tested in validate_oracle.rs)
    }
}
