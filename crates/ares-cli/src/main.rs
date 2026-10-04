mod moves_csv;

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
    /// 1-based plate number to slice for multi-plate 3MF projects
    #[arg(long)]
    plate: Option<u32>,
    /// Write the G-code processor move vertices as CSV (3MF input only)
    #[arg(long)]
    dump_moves: Option<PathBuf>,
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
    if extension.as_deref() != Some("3mf") && args.dump_moves.is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--dump-moves requires 3MF project input",
        )
        .into());
    }
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
            let metadata = ares_core::GenerationMetadata::default();
            let context = ares_core::SlicerContext::new(0);
            match &args.dump_moves {
                Some(path) => {
                    let processed = ares_core::slice_project_plate_processed(
                        input, metadata, args.plate, context,
                    )
                    .await?;
                    fs::write(path, moves_csv::format(&processed.result))?;
                    processed.gcode
                }
                None => {
                    ares_core::slice_project_plate(input, metadata, args.plate, context).await?
                }
            }
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
