mod args;
mod compatible_machine;
mod export_3mf;
mod export_stl;
mod failure;
mod installed_resources;
mod load_settings;
mod moves_csv;
mod native_refresh;
mod preset_sources;
mod project_presets;
#[cfg(target_os = "linux")]
mod result_json;
mod settings_export;
mod slice_plates;

use std::fs;
use std::io;
use std::path::Path;

use args::{Cli, Commands, SliceArgs};
use clap::Parser;
use failure::{CliFailure, slice_error_to_failure, substitution_warnings};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let result = match Cli::parse() {
        Cli {
            command: Some(Commands::Slice(args)),
            ..
        } => {
            if args.export_stl.is_some() || args.export_stls.is_some() || args.repair.is_some() {
                export_stl::run(&args)
            } else if let Some(output) = args.export_3mf.as_ref() {
                export_3mf::run(&args, output)
            } else {
                run_slice(args).await
            }
        }
        Cli {
            export_settings: Some(destination),
            project: Some(project),
            ..
        } => settings_export::run(&project, &destination),
        _ => unreachable!("arguments require either a slice or a settings export"),
    };
    if let Err(failure) = result {
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
    if args.selection.is_some() && extension.as_deref() != Some("3mf") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--slice requires 3MF project input",
        )
        .into());
    }
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
    if let Some(datadir) = &args.datadir
        && !datadir.as_os_str().is_empty()
        && !datadir.exists()
        && let Err(error) = fs::create_dir_all(datadir)
    {
        eprintln!(
            "Could not create data directory: {}: {error}\nsetup params error",
            datadir.display()
        );
        std::process::exit(-2);
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
                &slice_plates::output_directory(&args)
                    .map_or_else(|| args.output().to_owned(), |dir| dir.join("plate_1.gcode")),
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
    if let Some(selection) = args.selection {
        return slice_plates::run(&args, selection, &input, &overrides, context);
    }
    if let Some(catalog) = &args.project_preset_catalog
        && args.load_settings.is_none()
    {
        return project_presets::slice(
            &args,
            project_presets::Entry::Catalog(catalog),
            &input,
            &overrides,
            context,
        )
        .await;
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
    if let Some(refresh) = native_refresh::load(&args, &input, &load_filaments)? {
        return project_presets::slice(
            &args,
            project_presets::Entry::Native(&refresh),
            &input,
            &overrides,
            context,
        )
        .await;
    }
    let output = match extension {
        Some(extension) if extension == "3mf" => {
            let metadata = ares_core::GenerationMetadata::default();
            let resources = installed_resources::resolve(args.resources_dir.as_deref())?;
            let (source, machine_full) = compatible_machine::load(&input, &resources)?;
            let system_catalog = args
                .resources_dir
                .as_deref()
                .map(project_presets::load_resources_catalog)
                .transpose()?;
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
                    let processed = match ares_core::slice_cli_project_plate_validated_with_catalog(
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
                        system_catalog.as_ref(),
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
                                args.output(),
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
                    match ares_core::slice_cli_project_plate_validated_with_catalog(
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
                        system_catalog.as_ref(),
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
                                args.output(),
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
            fs::write(args.output(), gcode)?;
            report_success(
                &input_file,
                substitutions(&input),
                #[cfg(target_os = "linux")]
                result_json::Success {
                    output: args.output(),
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
            let options = args.options.as_ref().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "STL input requires --options")
            })?;
            let options = fs::read(options)?;
            let options = serde_json::from_slice::<ares_core::SliceOptions>(&options)?;
            match ares_core::slice_stl_validated(input, options).await {
                Ok(gcode) => gcode,
                Err(error) => {
                    return Err(slice_error_to_failure(
                        error,
                        args.output(),
                        #[cfg(target_os = "linux")]
                        1,
                    ));
                }
            }
        }
        _ => unreachable!("input extension was validated above"),
    };
    fs::write(args.output(), output)?;

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

fn ensure_supported_input(path: &Path) -> Result<(), io::Error> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("3mf" | "stl") => Ok(()),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "unsupported input extension: expected .3mf or .stl",
        )),
    }
}
