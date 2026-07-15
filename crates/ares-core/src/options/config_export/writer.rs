use crate::{ProjectBedType, ProjectSettings, SliceError};

use super::{
    collector::{ConfigEntry, collect_config_entries},
    transform::transformed_for_export,
    value::serialize_config_value,
};
use crate::options::project_config_views::ProjectConfigViews;

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

pub(crate) fn is_bambu_project(settings: &ProjectSettings) -> bool {
    settings
        .printer
        .remaining
        .printer_model
        .0
        .starts_with("Bambu Lab")
}

pub(crate) fn write_config_block(
    views: &ProjectConfigViews,
    plate_index: usize,
    output: &mut Vec<u8>,
) -> Result<(), SliceError> {
    let transformed = transformed_for_export(&views.full)?;
    let entries = collect_config_entries(&transformed).map_err(config_error)?;
    let mut scratch = Vec::new();
    scratch.extend_from_slice(START);
    write_canonical_entries(&transformed, plate_index, &entries, &mut scratch)?;
    write_runtime_tail(&views.runtime, &mut scratch)?;
    scratch.extend_from_slice(END);
    output.extend_from_slice(&scratch);
    Ok(())
}

pub(crate) fn write_canonical_entries(
    settings: &ProjectSettings,
    plate_index: usize,
    entries: &[ConfigEntry],
    output: &mut Vec<u8>,
) -> Result<(), SliceError> {
    for entry in entries {
        if BANNED_KEYS.contains(&entry.key.as_str()) || entry.is_nil {
            continue;
        }
        if matches!(entry.key.as_str(), "wipe_tower_x" | "wipe_tower_y") {
            let values = if entry.key == "wipe_tower_x" {
                &settings.project.print.wipe_tower_x.0
            } else {
                &settings.project.print.wipe_tower_y.0
            };
            let value = values
                .get(plate_index)
                .or_else(|| values.first())
                .ok_or_else(|| invalid(format!("{} must not be empty", entry.key)))?;
            append_line(output, &entry.key, &format!("{:.3}", value.0));
        }
        if entry.key == "extruder_colour" {
            let colour = serialize_config_value(&settings.filament.gcode.filament_colour)
                .map_err(config_error)?;
            append_line(output, &entry.key, &colour.token);
        } else {
            append_line(output, &entry.key, &entry.token);
        }
    }
    Ok(())
}

fn write_runtime_tail(runtime: &ProjectSettings, output: &mut Vec<u8>) -> Result<(), SliceError> {
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
    append_line(output, "first_layer_temperature", &nozzle.0.to_string());
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
