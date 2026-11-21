use std::{collections::BTreeMap, fs, io, path::Path};

use serde_json::{Value, json};

use crate::{
    SliceArgs,
    failure::{CliFailure, slice_error_to_failure},
};

fn resource_json(path: &Path) -> Result<Value, io::Error> {
    let bytes = fs::read(path)?;
    serde_json::from_slice(bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(&bytes))
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

pub(super) fn load_resources_catalog(
    resources_dir: &Path,
) -> Result<ares_core::PresetCatalogHandle, CliFailure> {
    let profiles_dir = resources_dir.join("profiles");
    let mut vendor_paths = fs::read_dir(&profiles_dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    vendor_paths.sort();
    let mut catalog = ares_core::PresetCatalogHandle::new();
    for index_path in vendor_paths {
        let id = index_path
            .file_stem()
            .and_then(|name| name.to_str())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid vendor filename"))?;
        let index = resource_json(&index_path)?;
        if index.get("name").and_then(Value::as_str).is_none() {
            continue;
        }
        let mut files = BTreeMap::new();
        let entries = [
            "machine_model_list",
            "machine_list",
            "process_list",
            "filament_list",
        ]
        .into_iter()
        .flat_map(|kind| index[kind].as_array().into_iter().flatten());
        for entry in entries {
            let Some(sub_path) = entry["sub_path"].as_str() else {
                continue;
            };
            let path = profiles_dir.join(id).join(sub_path);
            let value = resource_json(&path)?;
            files.insert(sub_path.to_owned(), value);
        }
        let vendor = json!({"id": id, "index": index, "files": files});
        catalog
            .add_vendor(
                &serde_json::to_vec(&vendor)
                    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?,
            )
            .map_err(|error| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("invalid vendor {id} profile JSON: {error}"),
                )
            })?;
    }
    Ok(catalog)
}

pub(super) async fn slice(
    args: &SliceArgs,
    catalog_path: &Path,
    input: &[u8],
    overrides: &ares_core::ConfigOverrides,
    context: ares_core::SlicerContext<'_>,
) -> Result<(), CliFailure> {
    if args
        .input
        .extension()
        .and_then(|extension| extension.to_str())
        != Some("3mf")
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--project-preset-catalog requires 3MF project input",
        )
        .into());
    }
    let input_file = args.input.to_string_lossy();
    let mut upward_report = Vec::new();
    let mut sliced_plate = ares_core::SlicedPlate::default();
    let processed = match ares_core::slice_cli_preset_project(
        input,
        ares_core::CliPresetSliceRequest {
            plate: args.plate,
            overrides,
            orient: args.orient == 1,
            uptodate: args.uptodate,
            record_moves: args.dump_moves.is_some(),
            sliced_plate: &mut sliced_plate,
        },
        context,
        |error| {
            slice_error_to_failure(
                error,
                &args.output,
                #[cfg(target_os = "linux")]
                args.plate.unwrap_or(1),
            )
        },
        || {
            let catalog = fs::read_to_string(catalog_path).map_err(|error| {
                io::Error::new(
                    error.kind(),
                    format!(
                        "cannot read project preset catalog {}: {error}",
                        catalog_path.display()
                    ),
                )
            })?;
            let filename = args
                .input
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidInput, "invalid model filename")
                })?;
            let upward_source = ares_core::UpwardMachineSource::from_project(input)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?;
            Ok(ares_core::CliPresetFiles::ProjectCatalog {
                filename,
                catalog,
                upward_source,
                upward_report: &mut upward_report,
            })
        },
    )? {
        Ok(processed) => processed,
        Err(error) => {
            crate::failure::slicing::check(
                args,
                &sliced_plate,
                &input_file,
                input,
                &upward_report,
            )?;
            let failure = slice_error_to_failure(
                error,
                &args.output,
                #[cfg(target_os = "linux")]
                args.plate.unwrap_or(1),
            )
            .with_upward_report(upward_report)
            .with_config_substitutions(&input_file, crate::substitutions(input));
            #[cfg(target_os = "linux")]
            let failure =
                failure.with_slicing_warnings(args.plate.unwrap_or(1), sliced_plate.warnings);
            return Err(failure);
        }
    };
    crate::failure::slicing::check(args, &sliced_plate, &input_file, input, &upward_report)?;
    if let Some(path) = &args.dump_moves {
        fs::write(path, crate::moves_csv::format(&processed.result))?;
    }
    fs::write(&args.output, processed.gcode)?;
    crate::report_success(
        &input_file,
        crate::substitutions(input),
        #[cfg(target_os = "linux")]
        crate::result_json::Success {
            output: &args.output,
            plate: args.plate.unwrap_or(1),
            plate_index: args.plate.unwrap_or(0),
            sliced_plate,
            upward_machines: &upward_report,
        },
    );
    Ok(())
}
