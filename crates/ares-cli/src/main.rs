mod args;
mod assembly;
mod compatible_machine;
mod custom_gcodes;
mod diagnostics;
mod direct_project;
mod direct_stl;
mod downward;
mod export_3mf;
mod export_stl;
mod failure;
mod gcode_file_sink;
mod installed_resources;
mod load_settings;
mod moves_csv;
mod native_refresh;
mod positional_warnings;
mod preset_sources;
mod project_presets;
#[cfg(target_os = "linux")]
mod result_json;
mod settings_export;
mod slice_plates;
mod statistics;

use std::{fs, io, path::Path};

use args::{Commands, SliceArgs, require_single_input};
use failure::{CliFailure, slice_error_to_failure, substitution_warnings};
use positional_warnings::PositionalWarnings;
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let (cli, export_before_slice) = args::parse();
    #[cfg(target_os = "linux")]
    let strict_mode = cli.direct.strict;
    let mut warnings = PositionalWarnings::default();
    let result = match cli.command {
        Some(Commands::Slice(args)) => {
            if args.export_stl.is_some() || args.export_stls.is_some() || args.repair.is_some() {
                export_stl::run(&args)
            } else {
                run_slice(args, &mut warnings).await
            }
        }
        None => match cli.direct.export_3mf.as_deref() {
            Some(output) => export_3mf::run_direct(&cli.direct, output, &mut warnings).await,
            None => direct_stl::run(cli.direct, export_before_slice, &mut warnings).await,
        },
    };
    if let Err(failure) = result {
        #[cfg(target_os = "linux")]
        let failure = warnings.attach(failure, strict_mode);
        failure.report(
            #[cfg(target_os = "linux")]
            strict_mode,
        );
    }
}

async fn run_slice(args: SliceArgs, warnings: &mut PositionalWarnings) -> Result<(), CliFailure> {
    if assembly::requested(&args) {
        return assembly::slice(&args).await;
    }
    require_single_input(&args.inputs);
    ensure_supported_input(args.input())?;

    let extension = args
        .input()
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
    let input = match fs::read(args.input()) {
        Ok(input) => input,
        // `CLI_FILE_NOTFOUND` (-3, Utils.hpp:27): the `No such file:` detail
        // goes to stderr, the generic string to result.json.
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Err(CliFailure::missing_input(
                args.input(),
                #[cfg(target_os = "linux")]
                &slice_plates::output_directory(&args)
                    .map_or_else(|| args.output().to_owned(), |dir| dir.join("plate_1.gcode")),
            ));
        }
        Err(error) => return Err(error.into()),
    };
    // `PrintBase::set_status` without a GUI callback logs `Percent %1%: %2%` (`PrintBase.cpp:127`).
    let mut print_status = |status: &ares_core::SlicingStatus<'_>| {
        eprintln!("Percent {}: {}", status.percent, status.text);
    };
    let mut context = args
        .project_actions
        .context(ares_core::SlicerContext::new(0))
        .with_allow_mix_temp(args.allow_mix_temp);
    if args.progress {
        context.set_status_callback(&mut print_status);
    }
    if let Some(selection) = args.selection {
        return slice_plates::run(
            (&args).into(),
            selection,
            &input,
            &overrides,
            (context, warnings),
        );
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
    if extension.as_deref() == Some("3mf") {
        let map_err = |error| {
            slice_error_to_failure(
                error,
                args.output(),
                #[cfg(target_os = "linux")]
                args.plate.unwrap_or(1),
            )
        };
        let admitted =
            ares_core::CliPresetProject::open(&input, args.allow_newer_file).map_err(map_err)?;
        admitted.check_post_process(&context).map_err(map_err)?;
        warnings.record(args.input(), args.output(), admitted.substitutions());
        if let Some(catalog) = &args.project_preset_catalog
            && args.load_settings.is_none()
        {
            return project_presets::slice(
                &args,
                project_presets::Entry::Catalog(catalog),
                admitted,
                &overrides,
                (context, warnings),
            );
        }
        if !load_settings.is_empty() || !load_filaments.is_empty() {
            return load_settings::slice(
                &args,
                admitted,
                &overrides,
                (&load_settings, &load_filaments),
                (context, warnings),
            )
            .await;
        }
        if native_refresh::requested(&args)
            && let Some(refresh) = native_refresh::load_admitted(&args, &admitted, &load_filaments)?
        {
            return project_presets::slice(
                &args,
                project_presets::Entry::Native(&refresh),
                admitted,
                &overrides,
                (context, warnings),
            );
        }
        let resources = installed_resources::resolve(args.resources_dir.as_deref())?;
        return project_presets::slice_embedded(
            &args,
            admitted,
            &resources,
            &overrides,
            (context, warnings),
        );
    }
    if args.project_preset_catalog.is_some() && args.load_settings.is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--project-preset-catalog requires 3MF project input",
        )
        .into());
    }
    if !load_settings.is_empty() || !load_filaments.is_empty() {
        let admitted = ares_core::CliPresetProject::admit(&input, &context).map_err(|error| {
            slice_error_to_failure(
                error,
                args.output(),
                #[cfg(target_os = "linux")]
                args.plate.unwrap_or(1),
            )
        })?;
        return load_settings::slice(
            &args,
            admitted,
            &overrides,
            (&load_settings, &load_filaments),
            (context, warnings),
        )
        .await;
    }
    let output = match extension {
        Some(extension) if extension == "stl" => {
            if args.orient == Some(1) {
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
            let filename = args
                .input()
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("input.stl");
            match ares_core::slice_stl_named_validated(input, filename, options).await {
                Ok(gcode) => gcode,
                Err(ares_core::SliceError::OptionStlModelRead(_)) => {
                    return Err(CliFailure::invalid_stl_input(
                        args.input(),
                        #[cfg(target_os = "linux")]
                        args.output(),
                    ));
                }
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
    result.write(result_json::substitution_records(file, substitutions).collect());
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

fn check_positional_project_version(
    input: &[u8],
    allow_newer_file: bool,
    report_path: &Path,
) -> Result<(), CliFailure> {
    // For routes without a shared project load, leave archive errors to the action's importer.
    if let Err(error @ ares_core::SliceError::FileVersionNotSupported) =
        ares_core::validate_project_version(input, allow_newer_file)
    {
        return Err(slice_error_to_failure(
            error,
            report_path,
            #[cfg(target_os = "linux")]
            0,
        ));
    }
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
