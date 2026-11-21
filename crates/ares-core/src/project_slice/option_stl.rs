use crate::{
    ConfigOverrides, GenerationMetadata, Project, SliceError, SliceOptions, SlicerContext,
    options::{flush_matrix::PreparationContext, json_deserialize, registry::option_definitions},
    project::raw_settings::{ProjectSettingsRaw, RawConfigValue},
};

use super::{ProjectInput, ProjectSource};

pub(crate) fn load(
    input: &[u8],
    filename: &str,
    options: &SliceOptions,
) -> Result<(Project, PreparationContext), SliceError> {
    let mut raw = ProjectSettingsRaw::default();
    for (key, value) in options.values() {
        let value = serde_json::from_value::<RawConfigValue>(value.clone())
            .map_err(|error| SliceError::InvalidInput(format!("invalid {key}: {error}")))?;
        raw.insert(key.clone(), value);
    }
    raw.handle_legacy();
    raw.normalize_wire_vector_shapes()
        .map_err(SliceError::InvalidInput)?;
    raw.normalize_scalar_arrays()
        .map_err(SliceError::InvalidInput)?;

    // OrcaSlicer.cpp:3202–3205,4157–4223: no loaded filament preset columns.
    let changed = raw
        .entries()
        .map(|(key, _)| key.clone())
        .collect::<Vec<_>>();
    let keys = changed.iter().map(String::as_str).collect::<Vec<_>>();
    raw.insert_strings("inherits_group", &[String::new(), String::new()]);
    raw.insert_strings(
        "different_settings_to_system",
        &[String::new(), String::new()],
    );
    crate::options::overrides::dirty_columns::record_changed(&mut raw, &keys)?;
    if !raw.contains_key("print_compatible_printers") {
        raw.insert_strings("print_compatible_printers", &[]);
    }

    // OrcaSlicer.cpp:4248–4259: overlay the options on the static FFF
    // FullPrintConfig, then normalize the completed filament vectors.
    for def in option_definitions() {
        if def.full_config_default && !raw.contains_key(def.key) {
            let value =
                json_deserialize::decode(def.kind, def.default_wire_value).map_err(|error| {
                    SliceError::InvalidInput(format!("invalid default {}: {error}", def.key))
                })?;
            raw.insert(def.key.to_owned(), value);
        }
    }
    raw.normalize_filament_values_to_variants();
    // PrintApply.cpp:1345–1347 creates these IDs even without selected presets.
    for key in [
        "print_settings_id",
        "printer_settings_id",
        "filament_settings_id",
    ] {
        if !raw.contains_key(key) {
            let def = crate::option_definition(key).unwrap();
            let default = json_deserialize::decode(def.kind, def.default_wire_value)
                .map_err(SliceError::InvalidInput)?;
            raw.insert(key.to_owned(), default);
        }
    }
    let settings = json_deserialize::deserialize_config_values(raw.clone().into_object())
        .map_err(SliceError::InvalidInput)?
        .resolve();
    let preparation = PreparationContext::capture(&raw, &ConfigOverrides::default())?;
    let mut project = Project::from_stl(input, filename, settings, raw)
        .map_err(|error| SliceError::OptionStlModelRead(Box::new(error)))?;
    // The single non-project instance is dropped before default placement.
    // OrcaSlicer.cpp:1978 and :5619–6064.
    project.drop_instances_to_ground();
    let center =
        super::profile_entry::bed_center(&project.settings().printer.remaining.printable_area);
    project.center_instances_around_point(center);
    Ok((project, preparation))
}

pub async fn slice_stl_validated(
    input: impl AsRef<[u8]>,
    options: SliceOptions,
) -> Result<Vec<u8>, SliceError> {
    slice(input.as_ref(), "input.stl", &options, true).map_err(byte_api_error)
}

pub async fn slice_stl_named_validated(
    input: impl AsRef<[u8]>,
    filename: &str,
    options: SliceOptions,
) -> Result<Vec<u8>, SliceError> {
    slice(input.as_ref(), filename, &options, true)
}

/// Option-only STL action with the existing plate report output.
pub async fn slice_cli_stl_with_defaults(
    input: &[u8],
    filename: &str,
    report: &mut crate::SlicedPlate,
) -> Result<Vec<u8>, SliceError> {
    slice_cli_stl_with_defaults_processed(input, filename, report, false)
        .await
        .map(|processed| processed.gcode)
}

pub async fn slice_cli_stl_with_defaults_processed(
    input: &[u8],
    filename: &str,
    report: &mut crate::SlicedPlate,
    record_moves: bool,
) -> Result<crate::ProcessedGCode, SliceError> {
    slice_with_report(
        input,
        filename,
        &SliceOptions::default(),
        (true, record_moves),
        Some(report),
    )
}

pub(crate) fn slice(
    bytes: &[u8],
    filename: &str,
    options: &SliceOptions,
    validate_print: bool,
) -> Result<Vec<u8>, SliceError> {
    slice_with_report(bytes, filename, options, (validate_print, false), None)
        .map(|processed| processed.gcode)
}

fn slice_with_report(
    bytes: &[u8],
    filename: &str,
    options: &SliceOptions,
    (validate_print, record_moves): (bool, bool),
    sliced_plate: Option<&mut crate::SlicedPlate>,
) -> Result<crate::ProcessedGCode, SliceError> {
    super::slice_source(
        ProjectSource {
            input: ProjectInput::OptionStl {
                bytes,
                filename,
                options,
            },
            validate_print,
            sliced_plate,
            ..ProjectSource::from(bytes)
        },
        GenerationMetadata::default(),
        record_moves,
        SlicerContext::new(0),
    )
}

pub(crate) fn byte_api_error(error: SliceError) -> SliceError {
    match error {
        SliceError::OptionStlModelRead(error) => *error,
        SliceError::Validate { message, .. } => SliceError::InvalidInput(message),
        other => other,
    }
}
