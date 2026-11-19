use std::{collections::BTreeMap, fs, path::Path};

use ares_core::{Assembly, AssemblyError, AssemblyFile, AssemblyList, ConfigOverrides};

use crate::{
    CliFailure, SliceArgs, read_preset_files, report_success, slice_error_to_failure,
    split_native_list,
};

pub(super) fn requested(args: &SliceArgs) -> bool {
    args.assemble
        || args.load_assemble_list.is_some()
        || args.assemble_list.is_some()
        || args.arrange.is_some()
        || !args.clone_objects.is_empty()
        || !args.load_filament_ids.is_empty()
}

fn validate(args: &SliceArgs) -> Result<(), AssemblyError> {
    let invalid = |message: &str| AssemblyError::Invalid(message.into());
    let count = args.inputs.len();
    for (label, values) in [
        ("clone_objects", &args.clone_objects),
        ("loaded_filament_ids", &args.load_filament_ids),
    ] {
        if !values.is_empty() {
            if values.len() != count {
                return Err(AssemblyError::Invalid(format!(
                    "{label} size {} should be the same with input files size {count}",
                    values.len()
                )));
            }
            if args.load_filaments.is_none() {
                return Err(AssemblyError::Invalid(format!(
                    "{label} should be used with load_filaments together"
                )));
            }
        }
    }
    if args.load_assemble_list.is_some() {
        if args.assemble || count != 0 || args.arrange.is_some() || args.orient.is_some() {
            return Err(invalid(
                "load_assemble_list should not be used with input model files to load and should not be sued with transforms",
            ));
        }
    } else if args.assemble {
        let first_is_3mf = args
            .inputs
            .first()
            .and_then(|path| path.extension())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("3mf"));
        if !args.clone_objects.is_empty() {
            if first_is_3mf {
                return Err(invalid(
                    "can not load 3mf when set loaded_filament_ids or clone_objects",
                ));
            }
            for (index, &value) in args.clone_objects.iter().enumerate() {
                if value > 512 {
                    return Err(AssemblyError::Invalid(format!(
                        "invalid clone count {value} at index {}, max 512",
                        index + 1
                    )));
                }
            }
            return Err(invalid(
                "Invalid params: can not set assemble and clone_objects together.",
            ));
        }
        if first_is_3mf && !args.load_filament_ids.is_empty() {
            return Err(invalid(
                "can not load 3mf when set loaded_filament_ids or clone_objects",
            ));
        }
        if let Some(filaments) = &args.load_filaments {
            let max = split_native_list(filaments).len() as i32;
            for (index, &id) in args.load_filament_ids.iter().enumerate() {
                if id > max {
                    return Err(AssemblyError::Invalid(format!(
                        "invalid filament_id {id} at index {}, max {max}",
                        index + 1
                    )));
                }
            }
        }
        if count == 0 {
            return Err(invalid(
                "Invalid params: --assemble needs at least one input model.",
            ));
        }
        if args.arrange != Some(0) || args.orient.is_some() {
            return Err(invalid(
                "positional assembly requires --arrange 0 and no other transforms",
            ));
        }
    } else {
        return Err(invalid(
            "assembly parameters require --assemble or --load-assemble-list",
        ));
    }
    if args.options.is_some()
        || args.dump_moves.is_some()
        || args.overrides.is_some()
        || args.project_preset_catalog.is_some()
    {
        return Err(invalid(
            "assembly does not support adapter options, move export, overrides or preset catalog refresh",
        ));
    }
    Ok(())
}

fn read(path: &Path) -> Result<Vec<u8>, AssemblyError> {
    fs::read(path).map_err(|error| match error.kind() {
        std::io::ErrorKind::NotFound => {
            AssemblyError::Missing(format!("No such file: {}", path.display()))
        }
        _ => AssemblyError::Data(format!("{}: {error}", path.display())),
    })
}

fn read_list(path: &Path) -> Result<AssemblyList, AssemblyError> {
    let bytes = read(path).map_err(|error| match error {
        AssemblyError::Data(detail) => AssemblyError::Config(detail),
        error => error,
    })?;
    AssemblyList::from_json(&bytes)
}

fn preset_files(presets: &[(String, String)]) -> Vec<ares_core::LoadSettingsFile<'_>> {
    presets
        .iter()
        .map(|(label, json)| ares_core::LoadSettingsFile { label, json })
        .collect()
}

pub(super) async fn slice(args: &SliceArgs) -> Result<(), CliFailure> {
    if args.assemble_list.is_some() {
        eprintln!("Invalid option --assemble-list");
        std::process::exit(-2);
    }
    let failure = |error| {
        CliFailure::assembly(
            error,
            #[cfg(target_os = "linux")]
            &args.output,
        )
    };
    validate(args).map_err(failure)?;
    let list = args
        .load_assemble_list
        .as_deref()
        .map(read_list)
        .transpose()
        .map_err(failure)?;
    let mut files = BTreeMap::new();
    let paths = list.as_ref().map_or_else(
        || {
            args.inputs
                .iter()
                .map(|path| path.to_string_lossy())
                .collect::<Vec<_>>()
        },
        |list| list.paths().map(std::borrow::Cow::Borrowed).collect(),
    );
    // Read once per name but preserve order and duplicates in the positional model stream.
    for path in &paths {
        if !files.contains_key(path) {
            files.insert(path, read(Path::new(path.as_ref())).map_err(failure)?);
        }
    }
    let inputs = paths
        .iter()
        .map(|path| AssemblyFile {
            path,
            bytes: &files[path],
        })
        .collect::<Vec<_>>();
    let assembly = match &list {
        Some(list) => Assembly::from_list(list, &inputs),
        None => Assembly::from_models(&inputs, &args.load_filament_ids),
    }
    .map_err(failure)?;
    let settings = args
        .load_settings
        .as_deref()
        .map(split_native_list)
        .unwrap_or_default();
    let filaments = args
        .load_filaments
        .as_deref()
        .map(split_native_list)
        .unwrap_or_default();
    let settings = read_preset_files(&settings)?;
    let filaments = read_preset_files(&filaments)?;
    let settings = preset_files(&settings);
    let filaments = preset_files(&filaments);
    let mut sliced_plate = ares_core::SlicedPlate::default();
    let mut progress = |status: &ares_core::SlicingStatus<'_>| {
        eprintln!("Percent {}: {}", status.percent, status.text)
    };
    let mut context = ares_core::SlicerContext::new(0);
    if args.progress {
        context.set_status_callback(&mut progress);
    }
    let output = ares_core::slice_assembly(
        &assembly,
        ares_core::LoadSettingsSliceRequest {
            filename: assembly
                .archive()
                .map_or("Assembly", |archive| archive.path),
            settings: &settings,
            filaments: &filaments,
            catalog_json: None,
            uptodate: false,
            plate: args.plate,
            overrides: &ConfigOverrides::default(),
            placement: ares_core::Placement::None,
            orient: false,
            sliced_plate: &mut sliced_plate,
        },
        ares_core::GenerationMetadata::default(),
        context,
    )
    .await
    .map_err(|error| {
        slice_error_to_failure(
            error,
            &args.output,
            #[cfg(target_os = "linux")]
            args.plate.unwrap_or(1),
        )
    })?;
    fs::write(&args.output, output)?;
    let input = assembly.archive();
    report_success(
        input.map_or("Assembly", |file| file.path),
        input.map_or_else(Vec::new, |file| crate::substitutions(file.bytes)),
        #[cfg(target_os = "linux")]
        crate::result_json::Success {
            output: &args.output,
            plate: args.plate.unwrap_or(1),
            plate_index: args.plate.unwrap_or(0),
            sliced_plate,
            upward_machines: &[],
        },
    );
    Ok(())
}
