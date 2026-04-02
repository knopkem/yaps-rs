//! yaps-cli — Command-line interface for the YAPS-RS photo sorter.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use yaps_core::config::{Config, ConflictStrategy, DuplicateStrategy, FileOperation};
use yaps_core::ops::Organizer;

/// YAPS-RS: Yet Another Photo Sorter — Rust Edition
///
/// Organize your photos into structured directories based on EXIF metadata.
#[derive(Parser, Debug)]
#[command(name = "yaps", version, about, long_about = None)]
#[allow(clippy::struct_excessive_bools)]
struct Cli {
    /// Source directory containing photos to organize.
    #[arg(short, long)]
    source: PathBuf,

    /// Target directory for organized output.
    #[arg(short, long)]
    target: PathBuf,

    /// Folder pattern using {tag} placeholders.
    #[arg(long, default_value = "{year}/{month}-{month_long}")]
    folder_pattern: String,

    /// Filename pattern using {tag} placeholders.
    #[arg(long, default_value = "{day}-{month_short}-{hour}{minute}{second}-{filename}")]
    file_pattern: String,

    /// File operation mode.
    #[arg(long, value_enum, default_value = "copy")]
    operation: CliFileOperation,

    /// How to handle filename conflicts.
    #[arg(long, value_enum, default_value = "skip")]
    on_conflict: CliConflictStrategy,

    /// Disable duplicate detection.
    #[arg(long)]
    no_dedup: bool,

    /// Copy duplicates to a special folder instead of skipping them.
    #[arg(long)]
    keep_duplicates: bool,

    /// Don't recurse into subdirectories.
    #[arg(long)]
    no_recursive: bool,

    /// Preview operations without executing them.
    #[arg(long)]
    dry_run: bool,

    /// Increase verbosity (can be repeated: -v, -vv, -vvv).
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Load configuration from a TOML file.
    #[arg(long)]
    config: Option<PathBuf>,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum CliFileOperation {
    Copy,
    Move,
    Hardlink,
    Symlink,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum CliConflictStrategy {
    Skip,
    Rename,
    Overwrite,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let log_level = match cli.verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level)),
        )
        .init();

    // Build config from CLI args (or load from file)
    let config = if let Some(config_path) = &cli.config {
        Config::load(config_path).context("Failed to load configuration file")?
    } else {
        Config {
            source: cli.source,
            target: cli.target,
            recursive: !cli.no_recursive,
            file_operation: match cli.operation {
                CliFileOperation::Copy => FileOperation::Copy,
                CliFileOperation::Move => FileOperation::Move,
                CliFileOperation::Hardlink => FileOperation::Hardlink,
                CliFileOperation::Symlink => FileOperation::Symlink,
            },
            conflict_strategy: match cli.on_conflict {
                CliConflictStrategy::Skip => ConflictStrategy::Skip,
                CliConflictStrategy::Rename => ConflictStrategy::Rename,
                CliConflictStrategy::Overwrite => ConflictStrategy::Overwrite,
            },
            detect_duplicates: !cli.no_dedup,
            duplicate_strategy: if cli.keep_duplicates {
                DuplicateStrategy::CopyToFolder
            } else {
                DuplicateStrategy::Skip
            },
            folder_pattern: cli.folder_pattern,
            file_pattern: cli.file_pattern,
            dry_run: cli.dry_run,
            ..Default::default()
        }
    };

    if config.dry_run {
        println!("🔍 DRY RUN — no files will be modified\n");
    }

    println!(
        "📁 Source:  {}",
        config.source.display()
    );
    println!(
        "📂 Target:  {}",
        config.target.display()
    );
    println!(
        "📋 Pattern: {}/{}",
        config.folder_pattern, config.file_pattern
    );
    println!();

    // Create a simple progress callback
    let progress: yaps_core::ops::organizer::ProgressCallback =
        Box::new(|current: usize, total: usize, msg: &str| {
            if total > 0 {
                eprint!("\r  {msg} [{current}/{total}]");
            } else {
                eprint!("\r  {msg}");
            }
        });

    let report = Organizer::run(&config, Some(&progress))
        .context("Photo sorting operation failed")?;

    // Clear progress line
    eprintln!();

    // Display report
    println!("{report}");

    if report.files_failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}
