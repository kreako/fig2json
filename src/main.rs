use anyhow::{anyhow, bail, Context, Result};
use clap::Parser;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "fig2json")]
#[command(version, about = "Convert Figma .fig files to JSON")]
#[command(long_about = "Convert Figma .fig files to JSON\n\n\
    JSON output is pretty-printed by default with indentation.\n\n\
    For regular .fig files:\n  \
    fig2json input.fig [-o output.json] [--compact] [-v]\n\n\
    For ZIP files (extracts all and converts all .fig files inside):\n  \
    fig2json input.zip extract-dir [--compact] [-v]")]
struct Cli {
    /// Input .fig or .zip file path
    input: PathBuf,

    /// Directory to extract ZIP contents (required for ZIP files, converts all .fig files found)
    extract_dir: Option<PathBuf>,

    /// Output JSON file path (default: stdout) - Cannot be used with extract_dir
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Compact JSON output (default is pretty-printed with indentation)
    #[arg(long)]
    compact: bool,

    /// Verbose output for debugging
    #[arg(short, long)]
    verbose: bool,

    /// Generate both transformed .json and raw .raw.json files (without transformations)
    #[arg(long)]
    raw: bool,
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
    }

    // Check if input is a ZIP container
    let is_zip = fig2json::parser::is_zip_container(&bytes);

    // Validate arguments based on file type
    if is_zip {
        // ZIP mode: require extract_dir, forbid -o
        let extract_dir = cli.extract_dir.as_ref()
            .ok_or_else(|| anyhow!("ZIP files require an extraction directory as second argument"))?;

        if cli.output.is_some() {
            bail!("Cannot use -o/--output flag with extraction directory (ZIP mode)");
        }

        if extract_dir.exists() {
            bail!("Extraction directory already exists: {}\nPlease remove it first", extract_dir.display());
        }

        // ZIP extraction mode
        handle_zip_mode(&bytes, extract_dir, cli.compact, cli.verbose, cli.raw)?;
    } else {
        // Regular .fig file mode
        if cli.verbose {
            eprintln!("Converting to JSON...");
        }

        // Determine base directory for image file operations
        let base_dir = if let Some(output_path) = &cli.output {
            output_path.parent()
        } else {
            // If outputting to stdout, use current directory
            Some(std::path::Path::new("."))
        };

        let json = fig2json::convert(&bytes, base_dir).context("Failed to convert .fig file to JSON")?;

        if cli.verbose {
            eprintln!("Conversion successful!");
        }

        // Format output (pretty by default, compact if flag is set)
        let output = if cli.compact {
            serde_json::to_string(&json)?
        } else {
            serde_json::to_string_pretty(&json)?
        };

        // Write output
        match cli.output.as_ref() {
            Some(path) => {
                if cli.verbose {
                    eprintln!("Writing output to: {}", path.display());
                }
                fs::write(path, &output)
                    .with_context(|| format!("Failed to write output file: {}", path.display()))?;
                if cli.verbose {
                    eprintln!("Done!");
                }
            }
            None => {
                println!("{}", output);
            }
        }

        // If --raw flag is set, also generate raw JSON file
        if cli.raw {
            if cli.verbose {
                eprintln!("Converting to raw JSON...");
            }

            let raw_json = fig2json::convert_raw(&bytes).context("Failed to convert .fig file to raw JSON")?;

            let raw_output = if cli.compact {
                serde_json::to_string(&raw_json)?
            } else {
                serde_json::to_string_pretty(&raw_json)?
            };

            // Determine raw output path
            let raw_path = match cli.output.as_ref() {
                Some(path) => {
                    // Derive .raw.json from output path
                    let mut raw = path.clone();
                    raw.set_extension("raw.json");
                    raw
                }
                None => {
                    // Derive from input path
                    cli.input.with_extension("raw.json")
                }
            };

            if cli.verbose {
                eprintln!("Writing raw output to: {}", raw_path.display());
            }

            fs::write(&raw_path, raw_output)
                .with_context(|| format!("Failed to write raw output file: {}", raw_path.display()))?;

            if cli.verbose {
                eprintln!("Raw JSON done!");
            }
        }
    }

    Ok(())
}

/// Handle ZIP extraction mode: extract all files and convert all .fig files found
fn handle_zip_mode(zip_bytes: &[u8], extract_dir: &PathBuf, compact: bool, verbose: bool, raw: bool) -> Result<()> {
    if verbose {
        eprintln!("ZIP file detected - extracting to: {}", extract_dir.display());
    }

    // Extract entire ZIP to directory
    fig2json::parser::extract_zip_to_directory(zip_bytes, extract_dir)
        .context("Failed to extract ZIP file")?;

    if verbose {
        eprintln!("ZIP extracted successfully");
        eprintln!("Searching for .fig files...");
    }

    // Find all .fig files in extracted contents
    let fig_files = find_fig_files(extract_dir)?;

    if fig_files.is_empty() {
        bail!("No .fig files found in ZIP archive");
    }

    let file_count = fig_files.len();

    if verbose {
        eprintln!("Found {} .fig file(s)", file_count);
    }

    // Convert each .fig file
    for fig_path in fig_files {
        let relative_path = fig_path.strip_prefix(extract_dir)
            .unwrap_or(&fig_path);

        if verbose {
            eprintln!("Converting: {}", relative_path.display());
        }

        // Read .fig file
        let fig_bytes = fs::read(&fig_path)
            .with_context(|| format!("Failed to read .fig file: {}", fig_path.display()))?;

        // Determine base directory for image file operations (parent of .fig file)
        let base_dir = fig_path.parent();

        // Convert to JSON
        let json = fig2json::convert(&fig_bytes, base_dir)
            .with_context(|| format!("Failed to convert: {}", fig_path.display()))?;

        // Format output (pretty by default, compact if flag is set)
        let output = if compact {
            serde_json::to_string(&json)?
        } else {
            serde_json::to_string_pretty(&json)?
        };

        // Determine output path: same as .fig but with .json extension
        let output_path = fig_path.with_extension("json");

        // Write JSON file
        fs::write(&output_path, output)
            .with_context(|| format!("Failed to write output: {}", output_path.display()))?;

        if verbose {
            eprintln!("  → {}", output_path.strip_prefix(extract_dir).unwrap_or(&output_path).display());
        }

        // If --raw flag is set, also generate raw JSON file
        if raw {
            let raw_json = fig2json::convert_raw(&fig_bytes)
                .with_context(|| format!("Failed to convert to raw JSON: {}", fig_path.display()))?;

            let raw_output = if compact {
                serde_json::to_string(&raw_json)?
            } else {
                serde_json::to_string_pretty(&raw_json)?
            };

            // Determine raw output path: same as .fig but with .raw.json extension
            let raw_output_path = fig_path.with_extension("raw.json");

            // Write raw JSON file
            fs::write(&raw_output_path, raw_output)
                .with_context(|| format!("Failed to write raw output: {}", raw_output_path.display()))?;

            if verbose {
                eprintln!("  → {}", raw_output_path.strip_prefix(extract_dir).unwrap_or(&raw_output_path).display());
            }
        }
    }

    if verbose {
        eprintln!("Done! Converted {} file(s)", file_count);
    }

    Ok(())
}

/// Recursively find all .fig files in a directory
fn find_fig_files(dir: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut fig_files = Vec::new();

    fn visit_dir(dir: &PathBuf, fig_files: &mut Vec<PathBuf>) -> Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    visit_dir(&path, fig_files)?;
                } else if path.extension().and_then(|s| s.to_str()) == Some("fig") {
                    fig_files.push(path);
                }
            }
        }
        Ok(())
    }

    visit_dir(dir, &mut fig_files)?;
    Ok(fig_files)
}
