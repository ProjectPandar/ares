mod args;
mod compatible_machine;
mod direct_project;
mod direct_stl;
mod failure;
mod load_settings;
mod moves_csv;
mod project_presets;
#[cfg(target_os = "linux")]
mod result_json;
mod settings_export;
mod statistics;

use std::fs;
use std::io;
use std::path::Path;

use args::{Cli, Commands, SliceArgs};
use clap::{CommandFactory, FromArgMatches};
use failure::{CliFailure, slice_error_to_failure, substitution_warnings};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let matches = Cli::command().get_matches();
    let mut cli = Cli::from_arg_matches(&matches).unwrap_or_else(|error| error.exit());
    // OrcaSlicer.cpp:6297,6394–6395: actions take effect in argument order.
    if matches
        .index_of("mtcpp")
        .zip(matches.index_of("selection"))
        .is_some_and(|(limit, slice)| limit > slice)
    {
        cli.direct.mtcpp = None;
    }
    #[cfg(target_os = "linux")]
    let strict_mode = cli.direct.strict;
    let result = match cli.command {
        Some(Commands::Slice(args)) => run_slice(args).await,
        None => match (&cli.direct.export_settings, cli.direct.inputs.as_slice()) {
            (Some(destination), [project]) => settings_export::run(project, destination),
            (Some(_), _) => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "--export-settings requires a saved 3MF project",
            )
            .into()),
            (None, _) => direct_stl::run(cli.direct).await,
        },
    };
    if let Err(failure) = result {
        failure.report(
            #[cfg(target_os = "linux")]
            strict_mode,
        );
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
    // `PrintBase::set_status` without a GUI callback logs `Percent %1%: %2%` (`PrintBase.cpp:127`).
    let mut print_status = |status: &ares_core::SlicingStatus<'_>| {
        eprintln!("Percent {}: {}", status.percent, status.text);
    };
    let mut context = ares_core::SlicerContext::new(0);
    if args.progress {
        context.set_status_callback(&mut print_status);
    }
    if let Some(catalog) = &args.project_preset_catalog
        && args.load_settings.is_none()
    {
        return project_presets::slice(&args, catalog, &input, &overrides, context).await;
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
        return load_settings::slice(
            &args,
            &input,
            &overrides,
            (&load_settings, &load_filaments),
            context,
        )
        .await;
    }
    let output = match extension {
        Some(extension) if extension == "3mf" => {
            let metadata = ares_core::GenerationMetadata::default();
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
                            orient: args.orient == 1,
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
                            failure::slicing::check(
                                &args,
                                &sliced_plate,
                                &input_file,
                                &input,
                                &upward_report,
                            )?;
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
                    failure::slicing::check(
                        &args,
                        &sliced_plate,
                        &input_file,
                        &input,
                        &upward_report,
                    )?;
                    fs::write(path, moves_csv::format(&processed.result))?;
                    processed.gcode
                }
                None => {
                    match ares_core::slice_cli_project_plate_validated(
                        &input,
                        ares_core::CliProjectSliceRequest {
                            orient: args.orient == 1,
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
                            failure::slicing::check(
                                &args,
                                &sliced_plate,
                                &input_file,
                                &input,
                                &upward_report,
                            )?;
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
            failure::slicing::check(&args, &sliced_plate, &input_file, &input, &upward_report)?;
            fs::write(&args.output, gcode)?;
            report_success(
                &input_file,
                substitutions(&input),
                #[cfg(target_os = "linux")]
                result_json::Success {
                    strict_mode: false,
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
            if args.orient == 1 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "automatic orientation of STL input requires --load-settings or --project-preset-catalog",
                ).into());
            }
            let options = args.options.ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "STL input requires --options")
            })?;
            let options = fs::read(options)?;
            let options = serde_json::from_slice::<ares_core::SliceOptions>(&options)?;
            let filename = args
                .input
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("input.stl");
            match ares_core::slice_stl_named_validated(input, filename, options).await {
                Ok(gcode) => gcode,
                Err(ares_core::SliceError::OptionStlModelRead(_)) => {
                    return Err(CliFailure::invalid_stl_input(
                        &args.input,
                        #[cfg(target_os = "linux")]
                        &args.output,
                    ));
                }
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
