use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ares")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Slice(SliceArgs),
}

#[derive(Parser)]
struct SliceArgs {
    #[arg(long)]
    options: Option<PathBuf>,
    #[arg(short = 'o')]
    output: PathBuf,
    input: PathBuf,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    match Cli::parse().command {
        Commands::Slice(args) => run_slice(args).await?,
    }

    Ok(())
}

async fn run_slice(args: SliceArgs) -> Result<(), Box<dyn Error>> {
    ensure_supported_input(&args.input)?;

    let extension = args
        .input
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_owned);
    if extension.as_deref() == Some("3mf") && args.options.is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--options is not supported for 3MF project input",
        )
        .into());
    }
    let input = fs::read(args.input)?;
    let output = match extension {
        Some(extension) if extension == "3mf" => {
            let metadata = ares_core::GenerationMetadata::deterministic(2026, 1, 1, 0, 0, 0);
            ares_core::slice_project(input, metadata).await?
        }
        Some(extension) if extension == "stl" => {
            let options = args.options.ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "STL input requires --options")
            })?;
            let options = fs::read(options)?;
            let options = serde_json::from_slice::<ares_core::SliceOptions>(&options)?;
            ares_core::slice(input, options).await?
        }
        _ => unreachable!("input extension was validated above"),
    };
    fs::write(args.output, output)?;

    Ok(())
}

fn ensure_supported_input(path: &Path) -> Result<(), io::Error> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("3mf" | "stl") => Ok(()),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "unsupported input extension: expected .3mf or .stl",
        )),
    }
}
