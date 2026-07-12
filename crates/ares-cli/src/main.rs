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
    options: PathBuf,
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

    let options = fs::read(args.options)?;
    let options = serde_json::from_slice::<ares_core::SliceOptions>(&options)?;
    let input = fs::read(args.input)?;
    let output = ares_core::slice(input, options).await?;
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
