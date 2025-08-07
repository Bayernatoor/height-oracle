#[cfg(feature = "generate")]
use anyhow::{Context, Result};
#[cfg(feature = "generate")]
use height_oracle::HeightOracle;
#[cfg(feature = "generate")]
use std::path::Path;

#[cfg(feature = "generate")]
fn main() -> Result<()> {
    println!("=== Height Oracle Asset Builder ===\n");

    // Check for input file in order of preference
    let input_file = if Path::new("assets/test_sample.txt").exists() {
        "assets/test_sample.txt"
    } else if Path::new("assets/prebip34.txt").exists() {
        "assets/prebip34.txt"
    } else {
        return Err(anyhow::anyhow!(
            "No input file found. Please provide either assets/test_sample.txt or assets/prebip34.txt"
        ));
    };

    println!("📁 Building oracle from {}...", input_file);
    let oracle = HeightOracle::from_txt(input_file)
        .with_context(|| format!("Failed to build oracle from {}", input_file))?;

    println!("✅ Oracle built with {} entries", oracle.len());
    println!("📊 Memory stats:");
    println!("{}", oracle.memory_stats());

    // Create assets directory if it doesn't exist
    std::fs::create_dir_all("assets").context("Failed to create assets directory")?;

    println!("\n💾 Saving oracle to assets/phash.ptrh.dat + assets/heights.u18packed.dat...");
    oracle
        .save_to_paths("assets/phash.ptrh.dat", "assets/heights.u18packed.dat")
        .context("Failed to save oracle files")?;

    println!("✅ Assets saved successfully!");
    println!("\nYou can now run validation with:");
    println!(
        "cargo run --example validate_oracle --features generate --no-default-features --release"
    );

    Ok(())
}

#[cfg(not(feature = "generate"))]
fn main() {
    println!("This program requires the 'generate' feature flag.");
    println!("Run with: cargo run --features generate");
}
