use std::path::PathBuf;
use std::process;

use anyhow::{Context, Result};
use clap::Parser;
use tracing::info;

/// Muninn — filesystem metadata scanner for corpus analysis.
///
/// Walks one or more directory trees, extracts document metadata at varying
/// depth depending on file format, and produces a JSON report for downstream
/// analysis by Cake Intelligence.
#[derive(Parser, Debug)]
#[command(name = "muninn", version, about)]
struct Cli {
    /// Folder(s) to scan. At least one required.
    #[arg(required = true)]
    scan_roots: Vec<PathBuf>,

    /// Output directory for the report JSON.
    #[arg(short, long, default_value = ".")]
    output: PathBuf,

    /// Skip files larger than this (bytes). Default: 2 GB.
    #[arg(long, default_value_t = 2 * 1024 * 1024 * 1024)]
    max_file_size: u64,

    /// Pages to sample per document for text analysis.
    #[arg(long, default_value_t = 5)]
    text_depth: usize,

    /// Parallel processing threads. Default: num_cpus - 1.
    #[arg(short = 'j', long)]
    threads: Option<usize>,

    /// Exclude machine hostname and OS from the report.
    #[arg(long)]
    no_hostname: bool,

    /// Hash filenames instead of including full paths.
    #[arg(long)]
    hash_filenames: bool,

    /// Additional glob exclusion patterns.
    #[arg(long = "exclude")]
    extra_exclusions: Vec<String>,

    /// Custom cross-reference regex patterns.
    #[arg(long = "xref-pattern")]
    xref_patterns: Vec<String>,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e:#}");
        process::exit(1);
    }
}

fn run() -> Result<()> {
    // Logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("muninn=info".parse().unwrap()),
        )
        .init();

    let cli = Cli::parse();

    // Build ScanConfig from CLI args
    let mut config = muninn_core::ScanConfig::default();
    config.scan_roots = cli.scan_roots;
    config.output_dir = cli.output.clone();
    config.max_file_size_bytes = cli.max_file_size;
    config.text_extraction_depth = cli.text_depth;
    config.include_hostname = !cli.no_hostname;
    config.include_full_paths = !cli.hash_filenames;
    config.custom_xref_patterns = cli.xref_patterns;

    if let Some(threads) = cli.threads {
        config.concurrency = threads;
    }

    config.exclusion_patterns.extend(cli.extra_exclusions);

    // Run the scan
    let scanner = muninn_core::Scanner::new(config);
    let report = scanner.run().context("Scan failed")?;

    // Write report
    let hostname = report
        .scan_metadata
        .hostname
        .as_deref()
        .unwrap_or("unknown");
    let date = chrono::Utc::now().format("%Y%m%d");
    let filename = format!("muninn-report-{hostname}-{date}.json");
    let output_path = cli.output.join(&filename);

    let json = serde_json::to_string_pretty(&report).context("Failed to serialize report")?;
    std::fs::write(&output_path, &json)
        .with_context(|| format!("Failed to write report to {}", output_path.display()))?;

    info!("Report written to {}", output_path.display());
    println!(
        "✓ Scan complete: {} documents processed",
        report.corpus_summary.total_documents
    );
    println!("  Report: {}", output_path.display());

    Ok(())
}
