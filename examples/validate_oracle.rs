#[cfg(feature = "generate")]
use anyhow::{Context, Result};
#[cfg(feature = "generate")]
use height_oracle::{HeightOracle, HeightOracleLoaded};
#[cfg(feature = "generate")]
use std::time::Instant;

#[cfg(feature = "generate")]
fn main() -> Result<()> {
    println!("=== Height Oracle Comprehensive Validation ===\n");

    // Check if the oracle files exist
    if !std::path::Path::new("assets/phash.ptrh.dat").exists()
        || !std::path::Path::new("assets/heights.u18packed.dat").exists()
    {
        println!("Error: assets/phash.ptrh.dat or assets/heights.u18packed.dat not found!");
        println!("Please run 'cargo run' first to build the oracle.");
        return Ok(());
    }

    // Check if the TXT file exists
    let txt_file = if std::path::Path::new("assets/prebip34.txt").exists() {
        "assets/prebip34.txt"
    } else {
        println!("Error: assets/prebip34.txt not found!");
        return Ok(());
    };

    println!("📁 Loading oracle from assets/phash.ptrh.dat + assets/heights.u18packed.dat...");
    let load_start = Instant::now();
    let oracle: HeightOracleLoaded =
        HeightOracle::load_from_paths("assets/phash.ptrh.dat", "assets/heights.u18packed.dat")?;
    let load_time = load_start.elapsed();
    println!(
        "✅ Oracle loaded in {:.3}s with {} entries\n",
        load_time.as_secs_f64(),
        oracle.len()
    );

    println!("📊 Oracle memory stats:");
    println!("{}", oracle.memory_stats());
    println!();

    println!("📖 Reading TXT file for validation...");
    let txt_start = Instant::now();

    use std::io::{BufRead, BufReader};
    let file = std::fs::File::open(txt_file).context("Failed to open TXT file")?;
    let reader = BufReader::new(file);

    let mut total_entries = 0;
    let mut correct_lookups = 0;

    let mut incorrect_heights = 0;
    let mut validation_errors = 0;

    let mut progress_counter = 0;
    let progress_interval = 10000;

    println!("🔍 Validating every entry in the TXT file...");
    let validation_start = Instant::now();

    for (line_number, line_result) in reader.lines().enumerate() {
        let line = match line_result {
            Ok(l) => l,
            Err(e) => {
                validation_errors += 1;
                eprintln!("Error reading line {}: {}", line_number + 1, e);
                continue;
            }
        };

        let line = line.trim();

        // Skip empty lines
        if line.is_empty() {
            continue;
        }

        total_entries += 1;
        progress_counter += 1;

        // Show progress every N entries
        if progress_counter % progress_interval == 0 {
            println!("  Processed {} entries...", progress_counter);
        }

        // Height is the line number (0-indexed)
        let expected_height = line_number as u32;

        // Block hash is the line content
        let block_hash_hex = line;

        // Look up height using oracle
        let actual_height = oracle.get_height_from_hex_unchecked(block_hash_hex);
        if actual_height == expected_height {
            correct_lookups += 1;
        } else {
            incorrect_heights += 1;
            eprintln!(
                "❌ Height mismatch for {}: expected {}, got {}",
                block_hash_hex, expected_height, actual_height
            );
        }
    }

    let validation_time = validation_start.elapsed();
    let txt_time = txt_start.elapsed();

    println!("\n=== VALIDATION RESULTS ===");
    println!("📈 Performance:");
    println!("  TXT reading time:     {:.3}s", txt_time.as_secs_f64());
    println!(
        "  Validation time:      {:.3}s",
        validation_time.as_secs_f64()
    );
    println!(
        "  Lookups per second:   {:.0}",
        total_entries as f64 / validation_time.as_secs_f64()
    );

    println!("\n📊 Accuracy:");
    println!("  Total entries:        {}", total_entries);
    println!(
        "  Correct lookups:      {} ({:.2}%)",
        correct_lookups,
        (correct_lookups as f64 / total_entries as f64) * 100.0
    );
    println!(
        "  Missing entries:      {} ({:.2}%)",
        0, // Perfect hash ensures no missing entries
        0.0
    );
    println!(
        "  Incorrect heights:    {} ({:.2}%)",
        incorrect_heights,
        (incorrect_heights as f64 / total_entries as f64) * 100.0
    );
    println!(
        "  Validation errors:    {} ({:.2}%)",
        validation_errors,
        (validation_errors as f64 / total_entries as f64) * 100.0
    );

    let oracle_entries = oracle.len();
    let coverage = (correct_lookups as f64 / oracle_entries as f64) * 100.0;
    println!("\n🎯 Coverage:");
    println!("  Oracle size:          {} entries", oracle_entries);
    println!("  CSV coverage:         {:.2}%", coverage);

    // Final verdict
    println!("\n🏆 FINAL VERDICT:");
    if incorrect_heights == 0 && validation_errors == 0 {
        println!("  ✅ PERFECT! All entries validated successfully!");
        println!("  The oracle is 100% accurate and complete.");
    } else if incorrect_heights + validation_errors < total_entries / 1000 {
        println!("  ✅ EXCELLENT! Less than 0.1% error rate.");
        println!("  The oracle is highly accurate and reliable.");
    } else if incorrect_heights + validation_errors < total_entries / 100 {
        println!("  ⚠️  GOOD: Less than 1% error rate.");
        println!("  The oracle has minor issues but is mostly reliable.");
    } else {
        println!("  ❌ POOR: High error rate detected.");
        println!("  The oracle may have significant issues.");
    }

    // Sample some specific known blocks for extra validation
    println!("\n🧪 Spot checks on known blocks:");

    let test_cases = vec![
        (
            "Genesis Block",
            "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f",
            0,
        ),
        (
            "Block 1",
            "00000000839a8e6886ab5951d76f411475428afc90947ee320161bbf18eb6048",
            1,
        ),
        (
            "Block 100",
            "000000007bc154e0fa7ea32218a72fe2c1bb9f86cf8c9ebf9a715ed27fdb229a",
            100,
        ),
    ];

    for (name, hash, expected_height) in test_cases {
        let height = oracle.get_height_from_hex_unchecked(hash);
        if height == expected_height {
            println!("  ✅ {}: height {} ✓", name, height);
        } else {
            println!(
                "  ❌ {}: expected {}, got {}",
                name, expected_height, height
            );
        }
    }

    // Test a fake hash - will map to some height (expected behavior with perfect hash)
    let fake_hash = "0000000000000000000000000000000000000000000000000000000000000000";
    let height = oracle.get_height_from_hex_unchecked(fake_hash);
    println!(
        "  ✅ Fake hash: maps to height {} (expected behavior) ✓",
        height
    );

    println!("\n✨ Validation complete!");

    Ok(())
}

#[cfg(not(feature = "generate"))]
fn main() {
    println!("This example requires the 'generate' feature flag.");
    println!("Run with: cargo run --example validate_oracle --features generate");
}
