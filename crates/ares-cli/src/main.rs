mod moves_csv;

use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use ares_core::SliceError;
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

/// CLI failure reporting mirrors the upstream adapter: family failures run
/// `flush_and_exit` (`OrcaSlicer.cpp:396-411` — the stdout line and the
/// negative return, which becomes the POSIX exit 256+n) plus
/// `record_exit_reson` (`:416-460`, Linux-only result.json); every other
/// error keeps the plain `Error: <detail>` + exit 1 shape.
enum CliFailure {
    Family(Box<FamilyFailure>),
    Legacy(Box<dyn Error>),
}

struct FamilyFailure {
    return_code: i32,
    plate_index: u32,
    layer_height: f64,
    wall_loops: i32,
    sparse_infill_density: f64,
    error_string: &'static str,
    stderr: String,
    output: PathBuf,
}

impl CliFailure {
    fn report(self) -> ! {
        match self {
            Self::Family(failure) => failure.flush_and_exit(),
            Self::Legacy(error) => {
                eprintln!("Error: {error}");
                std::process::exit(1);
            }
        }
    }
}

impl FamilyFailure {
    fn flush_and_exit(self) -> ! {
        println!("run found error, return {}, exit...", self.return_code);
        eprint!("{}", self.stderr);
        self.record_exit_reason();
        std::process::exit(256 + self.return_code);
    }

    /// `record_exit_reson` (`OrcaSlicer.cpp:416-460`): Linux-only, sorted
    /// keys, tab-indented `dump(1, '\t')` with a trailing newline; failures
    /// to write are swallowed like the upstream `catch (...)`.
    #[cfg(target_os = "linux")]
    fn record_exit_reason(&self) {
        let json_error_string = serde_json::to_string(self.error_string).unwrap();
        let body = format!(
            "{{\n\t\"error_string\": {json_error_string},\n\t\"export_time\": 0,\n\t\
             \"layer_height\": {:?},\n\t\"plate_index\": {},\n\t\"prepare_time\": 0,\n\t\
             \"return_code\": {},\n\t\"sparse_infill_density\": {:?},\n\t\"wall_loops\": {}\n}}\n",
            self.layer_height,
            self.plate_index,
            self.return_code,
            self.sparse_infill_density,
            self.wall_loops,
        );
        let directory = self
            .output
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let _ = fs::write(directory.join("result.json"), body);
    }

    #[cfg(not(target_os = "linux"))]
    fn record_exit_reason(&self) {}
}

impl From<io::Error> for CliFailure {
    fn from(error: io::Error) -> Self {
        Self::Legacy(Box::new(error))
    }
}

impl From<serde_json::Error> for CliFailure {
    fn from(error: serde_json::Error) -> Self {
        Self::Legacy(Box::new(error))
    }
}

/// Exhaustive variant → CLI code mapping (`Utils.hpp:24-59`). The four
/// family variants carry the result.json payload; everything else keeps
/// the legacy `Error:` channel — those surfaces have no native family
/// emulation in this slice (recorded in clusters.md).
fn slice_error_to_failure(error: SliceError, output: &Path) -> CliFailure {
    let family =
        |return_code, plate_index, layer_height, wall_loops, sparse, error_string, stderr| {
            CliFailure::Family(Box::new(FamilyFailure {
                return_code,
                plate_index,
                layer_height,
                wall_loops,
                sparse_infill_density: sparse,
                error_string,
                stderr,
                output: output.to_owned(),
            }))
        };
    match error {
        SliceError::PostProcessNotSupported => family(
            -19,
            0,
            0.0,
            0,
            0.0,
            "post_process is not supported under CLI.",
            String::new(),
        ),
        SliceError::NoSuitableObjects(_) => family(
            -50,
            1,
            0.0,
            0,
            0.0,
            "One of the plate is empty or has no object fully inside it. Please check that the 3mf contains no empty plate in Orca Slicer before uploading.",
            String::new(),
        ),
        SliceError::InvalidValuesInProject(detail) => family(
            -18,
            0,
            0.0,
            0,
            0.0,
            "Invalid parameter value(s) included in the 3mf file.",
            format!("Param values in 3mf/config error: \n{detail}\n"),
        ),
        SliceError::Validate {
            message,
            plate_index,
            layer_height,
            wall_loops,
            sparse_infill_density,
        } => family(
            -51,
            plate_index,
            layer_height,
            wall_loops,
            sparse_infill_density,
            "There are some incorrect slicing parameters in the 3mf. Please verify the slicing of all plates in Orca Slicer before uploading.",
            format!("{message}\n"),
        ),
        SliceError::EmptyInput
        | SliceError::InvalidInput(_)
        | SliceError::ProjectSlicingIncomplete
        | SliceError::UnsupportedProjectFeature(_) => CliFailure::Legacy(Box::new(error)),
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
            return Err(CliFailure::Family(Box::new(FamilyFailure {
                return_code: -3,
                plate_index: 0,
                layer_height: 0.0,
                wall_loops: 0,
                sparse_infill_density: 0.0,
                error_string: "The input files to the slicer are not found.",
                stderr: format!("No such file: {}\n", args.input.display()),
                output: args.output,
            })));
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
                Err(error) => return Err(slice_error_to_failure(error, &args.output)),
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
