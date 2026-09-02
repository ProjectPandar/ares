use crate::{ProjectBedType, ProjectSettings, SliceError};

use super::{
    collector::{ConfigEntry, collect_config_entries},
    orca_block_keys,
    transform::transformed_for_export,
    value::serialize_config_value,
};
use crate::options::project_config_views::ProjectConfigViews;
use crate::project::raw_settings::ProjectSettingsRaw;

const START: &[u8] = b"; CONFIG_BLOCK_START\n";
const END: &[u8] = b"; CONFIG_BLOCK_END\n\n";
const BANNED_KEYS: [&str; 9] = [
    "compatible_printers",
    "compatible_prints",
    "print_host",
    "print_host_webui",
    "printhost_apikey",
    "printhost_cafile",
    "printhost_user",
    "printhost_password",
    "printhost_port",
];

pub(crate) fn write_config_block(
    views: &ProjectConfigViews,
    raw_settings: &ProjectSettingsRaw,
    plate_index: usize,
    output: &mut Vec<u8>,
) -> Result<(), SliceError> {
    let transformed = transformed_for_export(&views.full)?;
    let entries = collect_config_entries(&transformed).map_err(config_error)?;
    let mut scratch = Vec::new();
    scratch.extend_from_slice(START);
    write_canonical_entries(
        &transformed,
        plate_index,
        &entries,
        raw_settings,
        &mut scratch,
    )?;
    write_runtime_tail(
        &views.runtime,
        is_bbl_printer(views),
        &entries,
        &mut scratch,
    )?;
    scratch.extend_from_slice(END);
    output.extend_from_slice(&scratch);
    Ok(())
}

pub(crate) fn write_canonical_entries(
    settings: &ProjectSettings,
    plate_index: usize,
    entries: &[ConfigEntry],
    raw_settings: &ProjectSettingsRaw,
    output: &mut Vec<u8>,
) -> Result<(), SliceError> {
    // Orca emits the full config keys: every static FullPrintConfig key
    // plus any key the project presets define (`GCode.cpp:5636-5643`).
    // Ares registry keys outside that set only appear when the project
    // presets set them.
    let mut lines: Vec<(String, String)> = Vec::new();
    fn emit(lines: &mut Vec<(String, String)>, key: &str, token: String, is_nil: bool) {
        if BANNED_KEYS.contains(&key) || is_nil || token == "nil" {
            return;
        }
        lines.push((key.to_owned(), token));
    }
    for entry in entries {
        if matches!(entry.key.as_str(), "wipe_tower_x" | "wipe_tower_y") {
            // The wipe tower origin is written for every printer regardless
            // of the key set (`GCode.cpp:5646-5650`).
            let values = if entry.key == "wipe_tower_x" {
                &settings.project.print.wipe_tower_x.0
            } else {
                &settings.project.print.wipe_tower_y.0
            };
            let value = values
                .get(plate_index)
                .or_else(|| values.first())
                .ok_or_else(|| invalid(format!("{} must not be empty", entry.key)))?;
            emit(&mut lines, &entry.key, format!("{:.3}", value.0), false);
            // Upstream falls through: the ordinary serialization is written as
            // well (`GCode.cpp:5647-5655`).
        }
        // Orca emits the full config keys: every static FullPrintConfig key
        // plus any key the project presets define (`GCode.cpp:5636-5643`).
        // Ares keys outside that set only appear when the project presets
        // set them.
        let known = orca_block_keys::contains(&entry.key) || raw_settings.contains_key(&entry.key);
        if !known {
            continue;
        }
        if entry.key == "extruder_colour" {
            let colour = serialize_config_value(&settings.filament.gcode.filament_colour)
                .map_err(config_error)?;
            emit(&mut lines, &entry.key, colour.token, false);
            continue;
        }
        emit(&mut lines, &entry.key, entry.token.clone(), entry.is_nil);
    }
    for (key, token) in raw_settings.iter() {
        if matches!(key, "from" | "name" | "version") {
            continue;
        }
        if lines.iter().any(|(existing, _)| existing.as_str() == key) {
            continue;
        }
        let Some(rendered) = token else {
            continue;
        };
        emit(&mut lines, key, rendered.to_owned(), false);
    }
    // Orca iterates a sorted key map; a stable sort keeps the duplicated
    // wipe-tower origin lines adjacent like upstream.
    lines.sort_by(|left, right| left.0.cmp(&right.0));
    for (key, token) in lines {
        append_line(output, &key, &token);
    }
    Ok(())
}

fn is_bbl_printer(views: &ProjectConfigViews) -> bool {
    views
        .full
        .printer
        .remaining
        .printer_model
        .0
        .starts_with("Bambu Lab")
}

fn write_runtime_tail(
    runtime: &ProjectSettings,
    bbl_printer: bool,
    entries: &[ConfigEntry],
    output: &mut Vec<u8>,
) -> Result<(), SliceError> {
    let (bed_key, bed_values) = match runtime.project.print.curr_bed_type {
        ProjectBedType::SupertackPlate => (
            "supertack_plate_temp_initial_layer",
            &runtime.filament.print.supertack_plate_temp_initial_layer,
        ),
        ProjectBedType::CoolPlate => (
            "cool_plate_temp_initial_layer",
            &runtime.filament.print.cool_plate_temp_initial_layer,
        ),
        ProjectBedType::TexturedCoolPlate => (
            "textured_cool_plate_temp_initial_layer",
            &runtime
                .filament
                .print
                .textured_cool_plate_temp_initial_layer,
        ),
        ProjectBedType::EngineeringPlate => (
            "eng_plate_temp_initial_layer",
            &runtime.filament.print.eng_plate_temp_initial_layer,
        ),
        ProjectBedType::HighTempPlate => (
            "hot_plate_temp_initial_layer",
            &runtime.filament.print.hot_plate_temp_initial_layer,
        ),
        ProjectBedType::TexturedPeiPlate => (
            "textured_plate_temp_initial_layer",
            &runtime.filament.print.textured_plate_temp_initial_layer,
        ),
        ProjectBedType::DefaultPlate => {
            return Err(invalid(
                "curr_bed_type Default Plate has no first-layer temperature",
            ));
        }
    };
    let bed = bed_values
        .0
        .first()
        .ok_or_else(|| invalid(format!("{bed_key} must not be empty")))?;
    let nozzle = runtime
        .filament
        .print
        .nozzle_temperature_initial_layer
        .0
        .first()
        .ok_or_else(|| invalid("nozzle_temperature_initial_layer must not be empty"))?;
    append_line(output, "first_layer_bed_temperature", &bed.0.to_string());
    // Upstream appends compatible info after the config keys: the non-BBL
    // footer also mirrors the printable_area serialization as bed_shape and
    // re-formats the initial layer print height at %.3f
    // (`GCode.cpp:3554-3562`); the BBL block stops at the two temperature
    // lines (`GCode.cpp:2648-2657`).
    if !bbl_printer {
        let area = entries
            .iter()
            .find(|entry| entry.key == "printable_area")
            .map(|entry| entry.token.as_str())
            .unwrap_or_default();
        append_line(output, "bed_shape", area);
        append_line(output, "first_layer_temperature", &nozzle.0.to_string());
        if let Some(height) = entries
            .iter()
            .find(|entry| entry.key == "initial_layer_print_height")
            .and_then(|entry| entry.token.parse::<f64>().ok())
        {
            append_line(output, "first_layer_height", &format!("{height:.3}"));
        }
    } else {
        append_line(output, "first_layer_temperature", &nozzle.0.to_string());
    }
    Ok(())
}

fn append_line(output: &mut Vec<u8>, key: &str, token: &str) {
    output.extend_from_slice(b"; ");
    output.extend_from_slice(key.as_bytes());
    output.extend_from_slice(b" = ");
    output.extend_from_slice(token.as_bytes());
    output.push(b'\n');
}

fn config_error(error: impl std::fmt::Display) -> SliceError {
    invalid(error.to_string())
}

fn invalid(message: impl Into<String>) -> SliceError {
    SliceError::InvalidInput(message.into())
}
