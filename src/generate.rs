//! Oracle generation and builder functionality
//!
//! This module contains all the code for building oracles from CSV files,
//! serialization/deserialization, and file I/O operations.

use crate::{packing, BlockHash, PtrHashType};
use anyhow::{Context, Result};
use epserde::prelude::*;
use std::io::{Read, Write};
use std::path::Path;

/// Memory usage statistics for the height oracle
#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// Bits per element for the PtrHash structure
    pub ptrhash_bits_per_element: f64,
    /// Bits per element for the heights vector
    pub heights_bits_per_element: f64,
    /// Total bits per element
    pub total_bits_per_element: f64,
    /// Number of elements
    pub num_elements: usize,
}

impl MemoryStats {
    /// Total memory usage in bytes
    pub fn total_bytes(&self) -> usize {
        ((self.total_bits_per_element * self.num_elements as f64) / 8.0).ceil() as usize
    }

    /// Total memory usage in KB
    pub fn total_kb(&self) -> f64 {
        self.total_bytes() as f64 / 1024.0
    }

    /// Total memory usage in MB
    pub fn total_mb(&self) -> f64 {
        self.total_kb() / 1024.0
    }
}

impl std::fmt::Display for MemoryStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Memory Statistics:")?;
        writeln!(f, "  Elements: {}", self.num_elements)?;
        writeln!(
            f,
            "  PtrHash: {:.2} bits/element",
            self.ptrhash_bits_per_element
        )?;
        writeln!(
            f,
            "  Heights: {:.2} bits/element",
            self.heights_bits_per_element
        )?;
        writeln!(
            f,
            "  Total: {:.2} bits/element ({:.1} KB)",
            self.total_bits_per_element,
            self.total_kb()
        )?;
        Ok(())
    }
}

/// Height lookup oracle using perfect hash function - in-memory version
///
/// Only available with "generate" feature for building oracles.
pub struct HeightOracle {
    /// Perfect hash function mapping BlockHash -> index
    phash: PtrHashType,
    /// Vector mapping index -> height
    heights: Vec<u32>,
}

/// Height lookup oracle using perfect hash function - loaded from disk
///
/// Only available with "generate" feature for loading oracles from disk.
pub struct HeightOracleLoaded {
    /// Perfect hash function mapping BlockHash -> index (loaded from disk)
    phash: PtrHashType,
    /// Vector mapping index -> height
    heights: Vec<u32>,
}

/// Minimal wrapper for height data serialization
#[derive(Clone)]
struct HeightData {
    heights: Vec<u32>,
}

impl HeightData {
    fn new(heights: Vec<u32>) -> Self {
        Self { heights }
    }

    fn serialize_to_writer<W: Write>(&self, writer: W) -> Result<()> {
        packing::serialize_heights(&self.heights, writer).context("Failed to serialize heights")
    }

    fn deserialize_from_reader<R: Read>(reader: R) -> Result<Self> {
        let heights =
            packing::deserialize_heights(reader).context("Failed to deserialize heights")?;
        Ok(Self::new(heights))
    }

    fn into_heights(self) -> Vec<u32> {
        self.heights
    }
}

impl HeightOracle {
    /// Create a new height oracle from a text file with one hash per line
    pub fn from_txt(txt_path: &str) -> Result<Self> {
        let (block_hashes, heights) = Self::parse_txt(txt_path)?;

        // Building perfect hash function
        // Build the perfect hash function
        let hash_to_index =
            ptr_hash::DefaultPtrHash::new(&block_hashes, ptr_hash::PtrHashParams::default());

        // Create mapping from perfect hash index to height
        let mut height_map = vec![0u32; block_hashes.len()];

        for (block_hash, height) in block_hashes.iter().zip(heights.iter()) {
            let index = hash_to_index.index(block_hash);
            height_map[index] = *height;
        }

        Ok(HeightOracle {
            phash: hash_to_index,
            heights: height_map,
        })
    }

    /// Parse text file with one hash per line (height = line number)
    fn parse_txt(txt_path: &str) -> Result<(Vec<BlockHash>, Vec<u32>)> {
        use std::io::{BufRead, BufReader};

        let file = std::fs::File::open(txt_path)
            .with_context(|| format!("Failed to open file: {txt_path}"))?;
        let reader = BufReader::new(file);

        let mut block_hashes = Vec::new();
        let mut heights = Vec::new();

        for (line_number, line_result) in reader.lines().enumerate() {
            let line = line_result.context("Failed to read line")?;
            let line = line.trim();

            // Skip empty lines
            if line.is_empty() {
                continue;
            }

            // New: if the line is a placeholder 'x' (we may mark version-2 blocks with 'x'), skip it
            if line == "x" {
                continue;
            }

            // Height is the line number (0-indexed)
            let height = line_number as u32;

            // Convert from reverse hex to network byte order
            let block_hash = crate::parse_block_hash(line)
                .map_err(|e| anyhow::anyhow!(e))
                .with_context(|| {
                    format!("Failed to parse block hash on line {}", line_number + 1)
                })?;

            heights.push(height);
            block_hashes.push(block_hash);
        }

        // Parsed block hashes from text file
        Ok((block_hashes, heights))
    }

    /// Look up the height for a given block hash (unchecked)
    ///
    /// IMPORTANT: This function always returns a height, but does NOT validate
    /// that the input hash was in the original dataset. For unknown hashes,
    /// it will return a height corresponding to some other block.
    ///
    /// The caller must ensure the input hash is from the valid domain
    /// (i.e., was in the original CSV file used to build the oracle).
    ///
    /// Note: We don't store the original hashes to save memory, so validation
    /// is not possible at runtime. Validation should be done during testing
    /// with the original CSV data.
    pub fn get_height_unchecked(&self, block_hash: &BlockHash) -> u32 {
        let index = self.phash.index(block_hash);
        self.heights[index]
    }

    /// Look up the height for a given block hash in reverse hex format (unchecked)
    ///
    /// # Panics
    ///
    /// Panics if the hex string is invalid. The caller must ensure the input
    /// is valid hex. Use a separate validation function if error handling is needed.
    pub fn get_height_from_hex_unchecked(&self, hex_str: &str) -> u32 {
        let block_hash: BlockHash = crate::parse_block_hash(hex_str)
            .unwrap_or_else(|_| panic!("Invalid hex string in unchecked function: {hex_str}"));
        self.get_height_unchecked(&block_hash)
    }

    /// Get the number of blocks in the oracle
    pub fn len(&self) -> usize {
        self.heights.len()
    }

    /// Check if the oracle is empty
    pub fn is_empty(&self) -> bool {
        self.heights.is_empty()
    }

    /// Save the oracle to disk using explicit file paths
    pub fn save_to_paths<P1: AsRef<Path>, P2: AsRef<Path>>(
        &self,
        ptrhash_path: P1,
        meta_path: P2,
    ) -> Result<()> {
        let ptrhash_path = ptrhash_path.as_ref();
        let meta_path = meta_path.as_ref();

        // Save PtrHash using epserde
        let hash_file = std::fs::File::create(ptrhash_path).with_context(|| {
            format!("Failed to create PtrHash file: {}", ptrhash_path.display())
        })?;
        self.phash
            .serialize(&mut std::io::BufWriter::new(hash_file))
            .context("Failed to serialize PtrHash")?;

        // Save metadata using 18-bit packed heights (25% space savings!)
        let height_data = HeightData::new(self.heights.clone());

        let meta_file = std::fs::File::create(meta_path)
            .with_context(|| format!("Failed to create metadata file: {}", meta_path.display()))?;
        height_data
            .serialize_to_writer(std::io::BufWriter::new(meta_file))
            .context("Failed to serialize metadata")?;

        Ok(())
    }

    /// Load the oracle from disk using explicit file paths
    pub fn load_from_paths<P1: AsRef<Path>, P2: AsRef<Path>>(
        ptrhash_path: P1,
        meta_path: P2,
    ) -> Result<HeightOracleLoaded> {
        let ptrhash_path = ptrhash_path.as_ref();
        let meta_path = meta_path.as_ref();

        // Load PtrHash using epserde full deserialization
        let hash_file = std::fs::File::open(ptrhash_path)
            .with_context(|| format!("Failed to open PtrHash file: {}", ptrhash_path.display()))?;
        let hash_to_index = PtrHashType::deserialize_full(&mut std::io::BufReader::new(hash_file))
            .context("Failed to deserialize PtrHash")?;

        // Load metadata using 18-bit packed heights
        let meta_file = std::fs::File::open(meta_path)
            .with_context(|| format!("Failed to open metadata file: {}", meta_path.display()))?;
        let height_data = HeightData::deserialize_from_reader(std::io::BufReader::new(meta_file))
            .context("Failed to deserialize metadata")?;

        Ok(HeightOracleLoaded {
            phash: hash_to_index,
            heights: height_data.into_heights(),
        })
    }

    /// Memory usage statistics
    pub fn memory_stats(&self) -> MemoryStats {
        let (pilots_bits, remap_bits) = self.phash.bits_per_element();
        let ptrhash_bits = pilots_bits + remap_bits;
        let heights_bits = (self.heights.len() * 4 * 8) as f64 / self.heights.len() as f64;

        MemoryStats {
            ptrhash_bits_per_element: ptrhash_bits,
            heights_bits_per_element: heights_bits,
            total_bits_per_element: ptrhash_bits + heights_bits,
            num_elements: self.heights.len(),
        }
    }
}

impl HeightOracleLoaded {
    /// Look up the height for a given block hash (unchecked)
    ///
    /// IMPORTANT: This function always returns a height, but does NOT validate
    /// that the input hash was in the original dataset. For unknown hashes,
    /// it will return a height corresponding to some other block in the range
    ///
    /// The caller must ensure the input hash is from the valid domain
    /// (i.e., was in the original CSV file used to build the oracle).
    pub fn get_height_unchecked(&self, block_hash: &BlockHash) -> u32 {
        let index = self.phash.index(block_hash);
        self.heights[index]
    }

    /// Look up the height for a given block hash in reverse hex format (unchecked)
    ///
    /// # Panics
    ///
    /// Panics if the hex string is invalid. The caller must ensure the input
    /// is valid hex. Use a separate validation function if error handling is needed.
    pub fn get_height_from_hex_unchecked(&self, hex_str: &str) -> u32 {
        let block_hash: BlockHash = crate::parse_block_hash(hex_str)
            .unwrap_or_else(|_| panic!("Invalid hex string in unchecked function: {hex_str}"));
        self.get_height_unchecked(&block_hash)
    }

    /// Get the number of blocks in the oracle
    pub fn len(&self) -> usize {
        self.heights.len()
    }

    /// Check if the oracle is empty
    pub fn is_empty(&self) -> bool {
        self.heights.is_empty()
    }

    /// Memory usage statistics
    pub fn memory_stats(&self) -> MemoryStats {
        let (pilots_bits, remap_bits) = self.phash.bits_per_element();
        let ptrhash_bits = pilots_bits + remap_bits;
        let heights_bits = (self.heights.len() * 4 * 8) as f64 / self.heights.len() as f64;

        MemoryStats {
            ptrhash_bits_per_element: ptrhash_bits,
            heights_bits_per_element: heights_bits,
            total_bits_per_element: ptrhash_bits + heights_bits,
            num_elements: self.heights.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_conversion() {
        let hex = "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f";
        let result: BlockHash = crate::parse_block_hash(hex).unwrap();

        // First few bytes of genesis block hash in network order
        assert_eq!(result[0], 0x6f);
        assert_eq!(result[1], 0xe2);
        assert_eq!(result[2], 0x8c);
        assert_eq!(result[3], 0x0a);
    }
}
