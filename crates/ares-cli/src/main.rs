mod compatible_machine;
mod failure;
mod moves_csv;
mod project_presets;
#[cfg(target_os = "linux")]
mod result_json;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use failure::{CliFailure, slice_error_to_failure, substitution_warnings};

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
    /// Complete preset catalog supplied explicitly for project-derived refresh
    #[arg(long, conflicts_with_all = ["load_settings", "load_filaments", "resources_dir"])]
    project_preset_catalog: Option<PathBuf>,
    /// Refresh unlisted project values from the supplied system presets
    #[arg(long, requires = "project_preset_catalog")]
    uptodate: bool,
    #[arg(long)]
    options: Option<PathBuf>,
    /// 1-based plate number to slice for multi-plate 3MF projects
    #[arg(long)]
    plate: Option<u32>,
    /// Resource root containing profiles/BBL/machine_full (default: resources beside the executable)
    #[arg(long)]
    resources_dir: Option<PathBuf>,
    /// Write the G-code processor move vertices as CSV (3MF input only)
    #[arg(long)]
    dump_moves: Option<PathBuf>,
    /// JSON object of user option overrides `{key: string|string[]}`
    /// applied on the project-settings layer (3MF input only; M854)
    #[arg(long)]
    overrides: Option<String>,
    /// OrcaSlicer-native preset files, `;` separated (machine;process;
    /// `PrintConfig.cpp:10890-10895`)
    #[arg(long)]
    load_settings: Option<String>,
    /// OrcaSlicer-native filament preset files, `;` separated
    /// (`PrintConfig.cpp:10896-10901`); empty slots are dropped like
    /// `OrcaSlicer.cpp:2202-2215` with the CLI-default
    /// `use_first_fila_as_default=false`
    #[arg(long)]
    load_filaments: Option<String>,
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
    // The file name as the CLI received it (`m_input_files`), which the substitution report names.
    let input_file = args.input.to_string_lossy().into_owned();
    if let Some(catalog) = &args.project_preset_catalog {
        return project_presets::slice(&args, catalog, &input, &overrides).await;
    }
    let load_settings = args
        .load_settings
        .as_deref()
        .map(split_native_list)
        .unwrap_or_default();
    let load_filaments = args
        .load_filaments
        .as_deref()
        .map(split_native_list)
        .unwrap_or_default();
    if !load_settings.is_empty() || !load_filaments.is_empty() {
        let filename = args
            .input
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid model filename"))?;
        // `OrcaSlicer.cpp:1682-1690`: the 3MF load block admits the
        // project before `:1953-2024` reads the preset files — probe the
        // admission first so the embedded `-19` gate wins over the
        // adapter's missing-file `-5`.
        if let Err(error) = ares_core::validate_load_settings_input(&input) {
            return Err(slice_error_to_failure(
                error,
                &args.output,
                #[cfg(target_os = "linux")]
                args.plate.unwrap_or(1),
            ));
        }
        let settings = read_preset_files(&load_settings)?;
        let filaments = read_preset_files(&load_filaments)?;
        let settings = settings
            .iter()
            .map(|(label, json)| ares_core::LoadSettingsFile { label, json })
            .collect::<Vec<_>>();
        let filaments = filaments
            .iter()
            .map(|(label, json)| ares_core::LoadSettingsFile { label, json })
            .collect::<Vec<_>>();
        let mut sliced_plate = ares_core::SlicedPlate::default();
        let gcode = match ares_core::slice_with_load_settings(
            &input,
            ares_core::LoadSettingsSliceRequest {
                filename,
                settings: &settings,
                filaments: &filaments,
                plate: args.plate,
                overrides: &overrides,
                placement: ares_core::Placement::None,
                sliced_plate: &mut sliced_plate,
            },
            ares_core::GenerationMetadata::default(),
            ares_core::SlicerContext::new(0),
        )
        .await
        {
            Ok(gcode) => gcode,
            Err(error) => {
                let failure = slice_error_to_failure(
                    error,
                    &args.output,
                    #[cfg(target_os = "linux")]
                    // The request above selects args.plate; CLI :5681-5690
                    // selects that same one-based plate (default first), and
                    // :5723/:433 records index+1 for the -50 result.
                    args.plate.unwrap_or(1),
                )
                .with_config_substitutions(&input_file, substitutions(&input));
                #[cfg(target_os = "linux")]
                let failure =
                    failure.with_slicing_warnings(args.plate.unwrap_or(1), sliced_plate.warnings);
                return Err(failure);
            }
        };
        fs::write(&args.output, gcode)?;
        report_success(
            &input_file,
            substitutions(&input),
            #[cfg(target_os = "linux")]
            result_json::Success {
                output: &args.output,
                plate: args.plate.unwrap_or(1),
                plate_index: args.plate.unwrap_or(0),
                sliced_plate,
                upward_machines: &[],
            },
        );
        return Ok(());
    }
    let output = match extension {
        Some(extension) if extension == "3mf" => {
            let metadata = ares_core::GenerationMetadata::default();
            let context = ares_core::SlicerContext::new(0);
            let (source, machine_full) =
                compatible_machine::load(&input, args.resources_dir.as_deref())?;
            let mut upward_report = Vec::new();
            let mut sliced_plate = ares_core::SlicedPlate::default();
            let upward = ares_core::CliUpwardMachine {
                source,
                input: ares_core::UpwardMachineInput::FetchOriginal(machine_full.as_deref()),
                report: &mut upward_report,
            };
            // OrcaSlicer.cpp:5681-5690 iterates from plate index0 when
            // --slice0 is used, or selects the explicit one-based plate.
            // This byte API slices one view; its default is that first plate.
            let gcode = match &args.dump_moves {
                Some(path) => {
                    let processed = match ares_core::slice_cli_project_plate_validated(
                        &input,
                        ares_core::CliProjectSliceRequest {
                            plate: args.plate,
                            overrides: &overrides,
                            record_moves: true,
                            upward,
                            sliced_plate: &mut sliced_plate,
                        },
                        metadata,
                        context,
                    )
                    .await
                    {
                        Ok(processed) => processed,
                        Err(error) => {
                            let failure = slice_error_to_failure(
                                error,
                                &args.output,
                                #[cfg(target_os = "linux")]
                                args.plate.unwrap_or(1),
                            )
                            .with_upward_report(upward_report)
                            .with_config_substitutions(&input_file, substitutions(&input));
                            #[cfg(target_os = "linux")]
                            let failure = failure.with_slicing_warnings(
                                args.plate.unwrap_or(1),
                                sliced_plate.warnings,
                            );
                            return Err(failure);
                        }
                    };
                    fs::write(path, moves_csv::format(&processed.result))?;
                    processed.gcode
                }
                None => {
                    match ares_core::slice_cli_project_plate_validated(
                        &input,
                        ares_core::CliProjectSliceRequest {
                            plate: args.plate,
                            overrides: &overrides,
                            record_moves: false,
                            upward,
                            sliced_plate: &mut sliced_plate,
                        },
                        metadata,
                        context,
                    )
                    .await
                    {
                        Ok(processed) => processed.gcode,
                        Err(error) => {
                            let failure = slice_error_to_failure(
                                error,
                                &args.output,
                                #[cfg(target_os = "linux")]
                                args.plate.unwrap_or(1),
                            )
                            .with_upward_report(upward_report)
                            .with_config_substitutions(&input_file, substitutions(&input));
                            #[cfg(target_os = "linux")]
                            let failure = failure.with_slicing_warnings(
                                args.plate.unwrap_or(1),
                                sliced_plate.warnings,
                            );
                            return Err(failure);
                        }
                    }
                }
            };
            fs::write(&args.output, gcode)?;
            report_success(
                &input_file,
                substitutions(&input),
                #[cfg(target_os = "linux")]
                result_json::Success {
                    output: &args.output,
                    plate: args.plate.unwrap_or(1),
                    plate_index: args.plate.unwrap_or(0),
                    sliced_plate,
                    upward_machines: &upward_report,
                },
            );
            return Ok(());
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
                    return Err(slice_error_to_failure(
                        error,
                        &args.output,
                        #[cfg(target_os = "linux")]
                        1,
                    ));
                }
            }
        }
        _ => unreachable!("input extension was validated above"),
    };
    fs::write(&args.output, output)?;

    Ok(())
}

/// The end of a successful 3MF slice: the load's substitution report on stderr
/// (`cli_report_config_substitutions`, `OrcaSlicer.cpp:520-534`) and, on Linux, the success
/// result.json.
fn report_success(
    file: &str,
    substitutions: Vec<ares_core::ConfigSubstitution>,
    #[cfg(target_os = "linux")] result: result_json::Success<'_>,
) {
    eprint!("{}", substitution_warnings(file, &substitutions));
    #[cfg(target_os = "linux")]
    result.write(file, substitutions);
}

/// The 3MF load's substitutions (`OrcaSlicer.cpp:1737-1758`). A project whose settings cannot be
/// read fails its load before the report (`:1992`), so it has none to report.
fn substitutions(input: &[u8]) -> Vec<ares_core::ConfigSubstitution> {
    ares_core::project_config_substitutions(input).unwrap_or_default()
}

/// Splits one native coStrings CLI value (`"a;b;c"`, `PrintConfig.cpp:10890-10901`).
fn split_native_list(value: &str) -> Vec<&str> {
    value.split(';').collect()
}

/// Reads the referenced preset files, keeping the path for upstream error
/// text (`Loading setting file "x" failed: ...`, `OrcaSlicer.cpp:2021-2023`).
/// Empty labels stay empty (dropped list slots).
fn read_preset_files(labels: &[&str]) -> Result<Vec<(String, String)>, CliFailure> {
    labels
        .iter()
        .map(|label| {
            let json = if label.is_empty() {
                String::new()
            } else {
                fs::read_to_string(label).map_err(|error| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Loading setting file \"{label}\" failed: {error}"),
                    )
                })?
            };
            Ok(((*label).to_owned(), json))
        })
        .collect()
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
