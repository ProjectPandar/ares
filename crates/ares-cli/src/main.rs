mod failure;
mod moves_csv;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use failure::{CliFailure, slice_error_to_failure};

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
    /// JSON object of user option overrides `{key: string|string[]}`
    /// applied on the project-settings layer (3MF input only; M854)
    #[arg(long)]
    overrides: Option<String>,
    #[arg(short = 'o')]
    output: PathBuf,
    input: PathBuf,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let Commands::Slice(args) = Cli::parse().command;
    if let Err(failure) = run_slice(args).await {
        failure.report();
    }
}

async fn run_slice(args: SliceArgs) -> Result<(), CliFailure> {
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
    if args.overrides.is_some() && extension.as_deref() != Some("3mf") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--overrides requires 3MF project input",
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
    let overrides = match &args.overrides {
        None => ares_core::ConfigOverrides::default(),
        Some(json) => ares_core::ConfigOverrides::from_json(json.as_bytes())
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error.to_string()))?,
    };
    if let Err(error) = overrides.validate_cli_config() {
        eprintln!("Params in command line error: \n{error}\nsetup params error");
        std::process::exit(-2);
    }
    let input = match fs::read(&args.input) {
        Ok(input) => input,
        // `CLI_FILE_NOTFOUND` (-3, Utils.hpp:27): the `No such file:` detail
        // goes to stderr, the generic string to result.json.
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Err(CliFailure::missing_input(
                &args.input,
                #[cfg(target_os = "linux")]
                &args.output,
            ));
        }
        Err(error) => return Err(error.into()),
    };
    let output = match extension {
        Some(extension) if extension == "3mf" => {
            let metadata = ares_core::GenerationMetadata::default();
            let context = ares_core::SlicerContext::new(0);
            match &args.dump_moves {
                Some(path) => {
                    let processed = match ares_core::slice_project_plate_processed_validated(
                        input, metadata, args.plate, &overrides, context,
                    )
                    .await
                    {
                        Ok(processed) => processed,
                        Err(error) => {
                            return Err(slice_error_to_failure(error, &args.output));
                        }
                    };
                    fs::write(path, moves_csv::format(&processed.result))?;
                    processed.gcode
                }
                None => {
                    match ares_core::slice_project_plate_validated(
                        input, metadata, args.plate, &overrides, context,
                    )
                    .await
                    {
                        Ok(gcode) => gcode,
                        Err(error) => {
                            return Err(slice_error_to_failure(error, &args.output));
                        }
                    }
                }
            }
        }
        Some(extension) if extension == "stl" => {
            let options = args.options.ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "STL input requires --options")
            })?;
            let options = fs::read(options)?;
            let options = serde_json::from_slice::<ares_core::SliceOptions>(&options)?;
            match ares_core::slice_stl_validated(input, options).await {
                Ok(gcode) => gcode,
                Err(error) => {
                    return Err(slice_error_to_failure(error, &args.output));
                }
            }
        }
        _ => unreachable!("input extension was validated above"),
    };
    fs::write(&args.output, output)?;

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
