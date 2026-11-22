use std::{fs, io, path::Path};

use ares_core::{
    CliProjectSliceRequest, CliUpwardMachine, ConfigOverrides, GenerationMetadata,
    LoadSettingsFile, LoadSettingsSliceRequest, Placement, ProcessedGCode, SliceError, SlicedPlate,
    SlicerContext, UpwardMachineInput, UpwardMachineSource,
};

use crate::{
    args::DirectSliceArgs,
    failure::{CliFailure, slice_error_to_failure},
    preset_sources::read_preset_files,
    slice_plates::{self, PlateOptions},
    split_native_list,
};

pub(super) async fn run(args: DirectSliceArgs) -> Result<(), CliFailure> {
    let datadir = args
        .datadir
        .as_deref()
        .filter(|path| !path.as_os_str().is_empty());
    if let Some(datadir) = datadir
        && !datadir.exists()
        && fs::create_dir_all(datadir).is_err()
    {
        eprintln!(
            "Could not create data directory: {}\nsetup params error",
            datadir.display()
        );
        std::process::exit(-2);
    }
    let selection = args
        .selection
        .expect("--slice is required without a subcommand");
    let plate = u32::try_from(selection).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "select a project plate with --slice <n>",
        )
    })?;
    if plate == 0
        && let Some((_, flag)) = [
            (args.strict, "--strict"),
            (args.load_custom_gcodes.is_some(), "--load-custom-gcodes"),
            (args.export_statistics, "--export-statistics"),
        ]
        .into_iter()
        .find(|(present, _)| *present)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{flag} is not supported with --slice 0 for a saved project"),
        )
        .into());
    }
    let files = (
        args.load_settings.as_deref(),
        args.load_filaments.as_deref(),
    );
    let input = &args.inputs[0];
    let directory = args
        .outputdir
        .as_deref()
        .expect("--outputdir is required without a subcommand");
    let output = directory.join(format!("plate_{}.gcode", plate.max(1)));
    let bytes = match fs::read(input) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Err(CliFailure::missing_input(
                input,
                #[cfg(target_os = "linux")]
                &output,
            ));
        }
        Err(error) => return Err(error.into()),
    };
    if plate == 0 {
        return slice_plates::run(
            PlateOptions {
                input,
                directory: Some(directory),
                datadir,
                load_settings: args.load_settings.as_deref(),
                load_filaments: args.load_filaments.as_deref(),
                project_preset_catalog: None,
                resources_dir: None,
                uptodate: false,
                orient: false,
                #[cfg(target_os = "linux")]
                report_plate: 1,
            },
            selection,
            &bytes,
            &ConfigOverrides::default(),
            SlicerContext::new(0).with_no_check(args.no_check),
        );
    }
    enum ProjectPresets<'a> {
        Replace(Option<&'a str>, Option<&'a str>),
        Saved(Option<UpwardMachineSource>, Option<Vec<u8>>),
    }
    let presets = match files {
        (None, None) => {
            let resources = crate::installed_resources::resolve(None)?;
            let (source, machine) = crate::compatible_machine::load(&bytes, &resources)?;
            ProjectPresets::Saved(source, machine)
        }
        (settings, filaments) => ProjectPresets::Replace(settings, filaments),
    };
    let context = crate::custom_gcodes::load(
        SlicerContext::new(0).with_no_check(args.no_check),
        args.load_custom_gcodes.as_deref(),
        &output,
        plate as i32 - 1,
    )?;
    let mut sliced_plate = SlicedPlate::default();
    let mut upward_machines = Vec::new();
    let processed = match presets {
        ProjectPresets::Replace(settings, filaments) => {
            let processed = slice_with_replacement(
                (input, &bytes, plate, &output),
                (settings, filaments, datadir),
                context,
                &mut sliced_plate,
                args.export_statistics,
            )
            .await?;
            upward_machines = std::mem::take(&mut sliced_plate.upward_machines);
            processed
        }
        ProjectPresets::Saved(source, machine) => {
            ares_core::slice_cli_project_plate_validated(
                &bytes,
                CliProjectSliceRequest {
                    plate: Some(plate),
                    overrides: &ConfigOverrides::default(),
                    record_moves: args.export_statistics,
                    orient: false,
                    upward: CliUpwardMachine {
                        source,
                        input: UpwardMachineInput::FetchOriginal(machine.as_deref()),
                        report: &mut upward_machines,
                    },
                    sliced_plate: &mut sliced_plate,
                },
                GenerationMetadata::default(),
                context,
            )
            .await
        }
    }
    .map_err(|error| {
        #[cfg(target_os = "linux")]
        let detail = match &error {
            ares_core::SliceError::Validate { message, .. } if args.no_check => {
                Some(message.clone())
            }
            _ => None,
        };
        let failure = slice_error_to_failure(
            error,
            &output,
            #[cfg(target_os = "linux")]
            plate,
        )
        .with_upward_report(upward_machines.clone());
        #[cfg(target_os = "linux")]
        let failure = failure.with_validation_detail(detail);
        failure
    })?;
    if args.strict {
        crate::failure::slicing::check_strict(
            #[cfg(target_os = "linux")]
            &output,
            plate,
            &sliced_plate,
            &upward_machines,
            Some((&input.to_string_lossy(), &bytes)),
        )?;
    }
    crate::statistics::write(&output, processed, args.export_statistics, plate)?;
    #[cfg(target_os = "linux")]
    crate::result_json::Success {
        strict_mode: args.strict,
        output: &output,
        plate,
        plate_index: plate,
        sliced_plate,
        upward_machines: &upward_machines,
    }
    .write(&input.to_string_lossy(), crate::substitutions(&bytes));
    Ok(())
}

async fn slice_with_replacement(
    (input, bytes, plate, output): (&Path, &[u8], u32, &Path),
    (settings, filaments, datadir): (Option<&str>, Option<&str>, Option<&Path>),
    context: SlicerContext<'_>,
    sliced_plate: &mut SlicedPlate,
    record_moves: bool,
) -> Result<Result<ProcessedGCode, SliceError>, CliFailure> {
    if let Err(error) = ares_core::validate_load_settings_input(bytes) {
        return Ok(Err(error));
    }
    let settings = read_preset_files(&settings.map(split_native_list).unwrap_or_default())?;
    let filaments = read_preset_files(&filaments.map(split_native_list).unwrap_or_default())?;
    let settings = settings
        .iter()
        .map(|(label, json, source_identity)| LoadSettingsFile {
            label,
            json,
            source_identity: source_identity.as_ref(),
        })
        .collect::<Vec<_>>();
    let filaments = filaments
        .iter()
        .map(|(label, json, source_identity)| LoadSettingsFile {
            label,
            json,
            source_identity: source_identity.as_ref(),
        })
        .collect::<Vec<_>>();
    let loaded_sources = datadir
        .map(|datadir| {
            slice_plates::sources_for_slice(
                Some(datadir),
                settings.iter().chain(&filaments).copied(),
                output,
                #[cfg(target_os = "linux")]
                plate,
            )
        })
        .transpose()?;
    let filename = input
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid model filename"))?
        .to_string_lossy();
    Ok(ares_core::slice_with_load_settings_processed(
        bytes,
        LoadSettingsSliceRequest {
            filename: &filename,
            catalog_json: None,
            loaded_sources: loaded_sources.as_ref(),
            uptodate: false,
            settings: &settings,
            filaments: &filaments,
            plate: Some(plate),
            overrides: &ConfigOverrides::default(),
            placement: Placement::None,
            orient: false,
            sliced_plate,
        },
        GenerationMetadata::default(),
        context,
        record_moves,
    )
    .await)
}
