//! 18-bit Height Compression
//!
//! This module provides efficient packing/unpacking of u32 heights using only 18 bits.
//! Maximum supported height is 262,143 (2^18 - 1), which covers all pre-BIP34 blocks.

use std::io::{Read, Write};

pub const MAX_HEIGHT: u32 = (1 << 18) - 1; // 262,143

/// Pack 4 heights into 9 bytes (72 bits total)
///
/// Each height uses 18 bits, for a total of 72 bits (9 bytes).
/// Heights are packed as: h0[18] | h1[18] | h2[18] | h3[18]
pub fn pack_4_heights(heights: &[u32; 4]) -> [u8; 9] {
    // Validate all heights fit in 18 bits
    for &height in heights {
        if height > MAX_HEIGHT {
            panic!("Height {} exceeds maximum {} (18 bits)", height, MAX_HEIGHT);
        }
    }

    let [h0, h1, h2, h3] = *heights;

    // Pack into 72 bits: h0[18] | h1[18] | h2[18] | h3[18]
    // Split: 64 bits in packed_low, 8 bits in packed_high
    let packed_low = h0 as u64 
        | ((h1 as u64) << 18) 
        | ((h2 as u64) << 36) 
        | (((h3 as u64) & 0x3FF) << 54); // h3 split: lower 10 bits
    
    let packed_high = (h3 >> 10) as u8; // Top 8 bits of h3

    // Serialize as 9 bytes little-endian
    let mut result = [0u8; 9];
    result[0..8].copy_from_slice(&packed_low.to_le_bytes());
    result[8] = packed_high;
    result
}

/// Unpack 4 heights from 9 bytes
pub fn unpack_4_heights(bytes: &[u8; 9]) -> [u32; 4] {
    // Read 64-bit value from first 8 bytes
    let packed_low = u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5], bytes[6], bytes[7],
    ]);
    let packed_high = bytes[8];

    // Unpack using 18-bit mask
    let mask = (1u64 << 18) - 1; // 0x3FFFF
    let h0 = (packed_low & mask) as u32;
    let h1 = ((packed_low >> 18) & mask) as u32;
    let h2 = ((packed_low >> 36) & mask) as u32;
    let h3 = (((packed_low >> 54) & 0x3FF) | ((packed_high as u64) << 10)) as u32;

    [h0, h1, h2, h3]
}

/// Serialize height arrays with metadata
///
/// Format: [num_entries: u32][remainder: u8][packed_data: 9*chunks bytes]
pub fn serialize_heights<W: Write>(heights: &[u32], mut writer: W) -> std::io::Result<()> {
    let num_entries = heights.len() as u32;
    let remainder = (num_entries % 4) as u8;
    let chunks = (num_entries + 3) / 4; // Round up division

    // Write metadata
    writer.write_all(&num_entries.to_le_bytes())?;
    writer.write_all(&[remainder])?;

    // Pack and write height data in chunks of 4
    for chunk_idx in 0..chunks {
        let start = (chunk_idx * 4) as usize;
        let end = std::cmp::min(start + 4, heights.len());
        
        // Create a 4-element array, padding with 0 if necessary
        let mut chunk = [0u32; 4];
        for (i, &height) in heights[start..end].iter().enumerate() {
            chunk[i] = height;
        }
        
        let packed = pack_4_heights(&chunk);
        writer.write_all(&packed)?;
    }

    Ok(())
}

/// Deserialize heights from reader
pub fn deserialize_heights<R: Read>(mut reader: R) -> std::io::Result<Vec<u32>> {
    // Read metadata
    let mut num_bytes = [0u8; 4];
    reader.read_exact(&mut num_bytes)?;
    let num_entries = u32::from_le_bytes(num_bytes);

    let mut remainder_bytes = [0u8; 1];
    reader.read_exact(&mut remainder_bytes)?;
    let _remainder = remainder_bytes[0];

    let chunks = (num_entries + 3) / 4; // Round up division
    let mut heights = Vec::with_capacity(num_entries as usize);

    // Read and unpack height data
    for chunk_idx in 0..chunks {
        let mut packed_bytes = [0u8; 9];
        reader.read_exact(&mut packed_bytes)?;
        
        let unpacked = unpack_4_heights(&packed_bytes);
        
        // Only take the valid heights from this chunk
        let start = (chunk_idx * 4) as usize;
        let end = std::cmp::min(start + 4, num_entries as usize);
        let valid_count = end - start;
        
        for i in 0..valid_count {
            heights.push(unpacked[i]);
        }
    }

    Ok(heights)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_pack_unpack_4_heights() {
        let heights = [0, 1, 100, MAX_HEIGHT];
        let packed = pack_4_heights(&heights);
        let unpacked = unpack_4_heights(&packed);
        assert_eq!(heights, unpacked);
    }

    #[test]
    fn test_pack_unpack_edge_cases() {
        // Test all zeros
        let heights = [0, 0, 0, 0];
        let packed = pack_4_heights(&heights);
        let unpacked = unpack_4_heights(&packed);
        assert_eq!(heights, unpacked);

        // Test all max values
        let heights = [MAX_HEIGHT, MAX_HEIGHT, MAX_HEIGHT, MAX_HEIGHT];
        let packed = pack_4_heights(&heights);
        let unpacked = unpack_4_heights(&packed);
        assert_eq!(heights, unpacked);
    }

    #[test]
    #[should_panic(expected = "exceeds maximum")]
    fn test_pack_height_too_large() {
        let heights = [0, 0, 0, MAX_HEIGHT + 1];
        pack_4_heights(&heights);
    }

    #[test]
    fn test_serialize_deserialize() {
        let heights = vec![0, 1, 100, 1000, 10000, MAX_HEIGHT];
        
        let mut buffer = Vec::new();
        serialize_heights(&heights, &mut buffer).unwrap();
        
        let mut cursor = Cursor::new(buffer);
        let deserialized = deserialize_heights(&mut cursor).unwrap();
        
        assert_eq!(heights, deserialized);
    }

    #[test]
    fn test_serialize_empty() {
        let heights = vec![];
        
        let mut buffer = Vec::new();
        serialize_heights(&heights, &mut buffer).unwrap();
        
        let mut cursor = Cursor::new(buffer);
        let deserialized = deserialize_heights(&mut cursor).unwrap();
        
        assert_eq!(heights, deserialized);
    }

    #[test]
    fn test_serialize_not_multiple_of_4() {
        let heights = vec![1, 2, 3, 4, 5]; // 5 elements, not multiple of 4
        
        let mut buffer = Vec::new();
        serialize_heights(&heights, &mut buffer).unwrap();
        
        let mut cursor = Cursor::new(buffer);
        let deserialized = deserialize_heights(&mut cursor).unwrap();
        
        assert_eq!(heights, deserialized);
    }

    #[test]
    fn test_packing_mathematics() {
        // Test the specific bit manipulation from the spec
        let heights = [0x12345, 0x23456, 0x34567, 0x12345]; // All fit in 18 bits
        
        let packed = pack_4_heights(&heights);
        let unpacked = unpack_4_heights(&packed);
        
        assert_eq!(heights, unpacked);
    }
}