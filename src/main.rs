use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "fig2json")]
#[command(version, about = "Convert Figma .fig files to JSON")]
struct Cli {
    /// Input .fig file path
    input: PathBuf,

    /// Output JSON file path (default: stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Pretty-print JSON output
    #[arg(short, long)]
    pretty: bool,

    /// Verbose output for debugging
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        eprintln!("Reading input file: {}", cli.input.display());
    }

    // Read input file
    let bytes = fs::read(&cli.input)
        .with_context(|| format!("Failed to read input file: {}", cli.input.display()))?;

    if cli.verbose {
        eprintln!("File size: {} bytes", bytes.len());
        eprintln!("Converting to JSON...");
    }

    // Convert to JSON
    let json = fig2json::convert(&bytes).context("Failed to convert .fig file to JSON")?;

    if cli.verbose {
        eprintln!("Conversion successful!");
    }

    // Format output
    let output = if cli.pretty {
        serde_json::to_string_pretty(&json)?
    } else {
        serde_json::to_string(&json)?
    };

    // Write output
    match cli.output {
        Some(path) => {
            if cli.verbose {
                eprintln!("Writing output to: {}", path.display());
            }
            fs::write(&path, output)
                .with_context(|| format!("Failed to write output file: {}", path.display()))?;
            if cli.verbose {
                eprintln!("Done!");
            }
        }
        None => {
            println!("{}", output);
        }
    }

    Ok(())
}
