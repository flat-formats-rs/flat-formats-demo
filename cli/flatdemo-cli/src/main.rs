//! FlatDemo CLI: Universal Replay Inspector and GoldSrc Adapter.

use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use flatdemo_adapter_goldsrc::convert_goldsrc_dem_to_fld;
use flatdemo_core::FlatDemoView;

#[derive(Parser)]
#[command(name = "fld")]
#[command(about = "FlatDemo: Universal High-Performance Replay Compiler & Inspector")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inspect a FlatDemo (.fld) container header and seek blocks.
    Inspect {
        /// Path to the .fld file.
        path: PathBuf,
    },
    /// Convert a legacy GoldSrc (.dem) replay into FlatDemo (.fld).
    Dem2fld {
        /// Input .dem file path.
        input: PathBuf,
        /// Output .fld file path.
        output: PathBuf,
        /// Enable in-container compression.
        #[arg(short, long)]
        compress: bool,
    },
    /// Benchmark $O(1)$ random seek latency on a FlatDemo file.
    BenchSeek {
        /// Path to the .fld file.
        path: PathBuf,
        /// Number of random seek lookups.
        #[arg(short, long, default_value_t = 10000)]
        iterations: u32,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Inspect { path } => {
            let bytes = fs::read(&path)
                .with_context(|| format!("Failed to read file: {}", path.display()))?;
            let view =
                FlatDemoView::new(&bytes).with_context(|| "Failed to parse FlatDemo container")?;

            let header = view.header();
            println!("=== FlatDemo Replay Container Inspector ===");
            println!("File: {}", path.display());
            println!("Version: {}", header.version);
            println!("Tickrate: {} Hz", header.tickrate);
            println!("Total Ticks: {}", header.total_ticks);
            println!("Seek Blocks (64 ticks each): {}", header.block_count);
            println!("Seek Table Offset: 0x{:X}", header.seek_table_offset);
            println!("BLAKE3 Merkle Root: {:02x?}", header.merkle_root_128);

            println!("\nSeek Table Overview:");
            println!("-------------------------------------------------------------");
            println!(
                "{:<6} {:<12} {:<10} {:<10} {:<10}",
                "Block", "Start Tick", "Offset", "Disk Size", "Mem Size"
            );
            println!("-------------------------------------------------------------");

            let sample_limit = view.seek_table().len().min(10);
            for (i, block) in view.seek_table().iter().take(sample_limit).enumerate() {
                println!(
                    "{:<6} {:<12} 0x{:<8X} {:<10} {:<10}",
                    i,
                    block.start_tick,
                    block.file_offset,
                    block.compressed_size,
                    block.uncompressed_size
                );
            }
            if view.seek_table().len() > sample_limit {
                println!(
                    "... ({} more blocks)",
                    view.seek_table().len() - sample_limit
                );
            }
        }
        Commands::Dem2fld {
            input,
            output,
            compress,
        } => {
            println!(
                "Converting GoldSrc .dem -> FlatDemo (.fld) [compressed: {}]...",
                compress
            );
            let dem_bytes = fs::read(&input)
                .with_context(|| format!("Failed to read input DEM: {}", input.display()))?;
            let fld_bytes = convert_goldsrc_dem_to_fld(&dem_bytes, compress)
                .with_context(|| "DEM to FlatDemo conversion failed")?;
            fs::write(&output, &fld_bytes)
                .with_context(|| format!("Failed to write FLD: {}", output.display()))?;
            println!(
                "Successfully converted {} ({} bytes) -> {} ({} bytes)",
                input.display(),
                dem_bytes.len(),
                output.display(),
                fld_bytes.len()
            );
        }
        Commands::BenchSeek { path, iterations } => {
            let bytes = fs::read(&path)
                .with_context(|| format!("Failed to read file: {}", path.display()))?;
            let view =
                FlatDemoView::new(&bytes).with_context(|| "Failed to parse FlatDemo container")?;

            let total_ticks = view.header().total_ticks.max(1);
            let start = std::time::Instant::now();
            let mut dummy = 0u64;

            for i in 0..iterations {
                let target_tick = ((i as u64) * 7919) % total_ticks;
                if let Ok((block_idx, block, sub_tick)) = view.seek_tick(target_tick) {
                    dummy = dummy.wrapping_add(
                        block_idx as u64 + block.compressed_size as u64 + sub_tick as u64,
                    );
                }
            }

            let elapsed = start.elapsed();
            let per_seek_ns = elapsed.as_nanos() as f64 / iterations as f64;
            println!("=== $O(1)$ Random Seek Benchmark ===");
            println!("Iterations: {}", iterations);
            println!("Total Time: {:?}", elapsed);
            println!("Latency per Seek: {:.2} ns", per_seek_ns);
            println!("Dummy sum: {}", dummy);
        }
    }

    Ok(())
}
