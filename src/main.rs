//! Command-line interface for exceldiff.

use anyhow::{Context, Result};
use clap::Parser;
use std::process;

use exceldiff::{DiffType, ExcelDiffWriter, ExcelReader, FileReader, WorksheetDiffer};

/// Get the version string (set by build.rs)
const VERSION: &str = env!("EXCELDIFF_VERSION");

#[derive(Parser)]
#[command(name = "exceldiff")]
#[command(author = "Johan <mumasoft@github>")]
#[command(version = VERSION)]
#[command(about = "Compare two Excel worksheets and highlight differences", long_about = None)]
struct Cli {
    /// Path to the first Excel file (baseline)
    #[arg(value_name = "FILE1")]
    file1: String,

    /// Path to the second Excel file (comparison)
    #[arg(value_name = "FILE2")]
    file2: String,

    /// Output file path
    #[arg(short, long, default_value = "diff_output.xlsx")]
    output: String,

    /// Sheet name in first file (default: first sheet)
    #[arg(long)]
    sheet1: Option<String>,

    /// Sheet name in second file (default: first sheet)
    #[arg(long)]
    sheet2: Option<String>,

    /// Only output rows with differences (exclude identical rows)
    #[arg(long)]
    diff_only: bool,

    /// Do not include header row when using --diff-only
    #[arg(long)]
    no_header: bool,

    /// Ignore whitespace differences (trim and collapse whitespace in string values)
    #[arg(long)]
    ignore_whitespace: bool,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {:#}", e);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    let reader = ExcelReader::new();

    // Validate file formats
    if !reader.supports(&cli.file1) {
        anyhow::bail!("{} is not a .xlsx file", cli.file1);
    }

    if !reader.supports(&cli.file2) {
        anyhow::bail!("{} is not a .xlsx file", cli.file2);
    }

    // Show available sheets if needed
    if cli.sheet1.is_none() {
        let sheets = reader
            .get_sheet_names(&cli.file1)
            .with_context(|| format!("Failed to read sheet names from {}", cli.file1))?;
        if let Some(first_sheet) = sheets.first() {
            println!("Reading first sheet from {}: '{}'", cli.file1, first_sheet);
        }
    }

    if cli.sheet2.is_none() {
        let sheets = reader
            .get_sheet_names(&cli.file2)
            .with_context(|| format!("Failed to read sheet names from {}", cli.file2))?;
        if let Some(first_sheet) = sheets.first() {
            println!("Reading first sheet from {}: '{}'", cli.file2, first_sheet);
        }
    }

    // Read worksheets
    println!("\nReading {}...", cli.file1);
    let data1 = reader
        .read(&cli.file1, cli.sheet1.as_deref())
        .with_context(|| format!("Failed to read {}", cli.file1))?;
    println!("  Loaded {} rows", data1.len());

    println!("Reading {}...", cli.file2);
    let data2 = reader
        .read(&cli.file2, cli.sheet2.as_deref())
        .with_context(|| format!("Failed to read {}", cli.file2))?;
    println!("  Loaded {} rows", data2.len());

    // Perform diff
    println!("\nComparing worksheets...");
    if cli.ignore_whitespace {
        println!("  Ignoring whitespace differences");
    }
    let differ = WorksheetDiffer::with_options(cli.ignore_whitespace);
    let diffs = differ.compare(&data1, &data2);

    // Count diff types
    let mut stats = std::collections::HashMap::new();
    stats.insert(DiffType::Identical, 0);
    stats.insert(DiffType::Modified, 0);
    stats.insert(DiffType::Removed, 0);
    stats.insert(DiffType::Added, 0);

    for diff in &diffs {
        *stats.entry(diff.diff_type).or_insert(0) += 1;
    }

    println!("\nDiff Summary:");
    println!("  Identical rows: {}", stats[&DiffType::Identical]);
    println!("  Modified rows:  {}", stats[&DiffType::Modified]);
    println!("  Removed rows:   {}", stats[&DiffType::Removed]);
    println!("  Added rows:     {}", stats[&DiffType::Added]);

    // Write output
    println!("\nWriting diff to {}...", cli.output);
    let writer = ExcelDiffWriter::new();
    let include_header = cli.diff_only && !cli.no_header;
    writer
        .write(&diffs, &cli.output, cli.diff_only, include_header)
        .with_context(|| format!("Failed to write output to {}", cli.output))?;

    if cli.diff_only {
        let output_rows = diffs
            .iter()
            .filter(|d| d.diff_type != DiffType::Identical)
            .count();
        let total_rows = if include_header {
            output_rows + 1
        } else {
            output_rows
        };
        println!("\nDone! Diff written to {} ({} rows)", cli.output, total_rows);
    } else {
        println!("\nDone! Diff written to {}", cli.output);
    }

    Ok(())
}
