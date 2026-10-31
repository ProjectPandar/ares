use crate::{
    OrcaBool, ProjectBedType, ProjectSettings, SliceError,
    options::{EXPLICIT_RULES, LegacyAction},
};

use super::{
    collector::{ConfigEntry, collect_config_entries},
    orca_block_keys,
    transform::transformed_for_export,
    value::serialize_config_value,
};
use crate::options::config_types::format_number;
use crate::options::project_config_views::ProjectConfigViews;
use crate::options::registry::OptionValueKind as Kind;
use crate::project::raw_settings::{ProjectSettingsRaw, RawConfigValue};
/// CLI-oracle export state the interactive `Print::apply` passes never
/// observe (see `transform::apply_cli_oracle_state`).
#[derive(Default)]
pub(super) struct ExportOverrides {
    /// Raw preset `enable_prime_tower` (None when the raw settings lack
    /// the key — keep the resolved value).
    pub(super) enable_prime_tower: Option<OrcaBool>,
    /// Raw preset `filament_map` token (joined `;` list).
    pub(super) raw_filament_map_token: Option<String>,
    #[allow(dead_code)]
    pub(super) is_bbl: bool,
}

impl ExportOverrides {
    /// Bookkeeping lookups read the raw map directly and render with the
    /// generic (non-Floats) token — no export-echo serialization, so no
    /// error path and no silent skips.
    fn from_raw(raw_settings: &ProjectSettingsRaw, is_bbl: bool) -> Self {
        let enable_prime_tower = raw_settings
            .get("enable_prime_tower")
            .and_then(RawConfigValue::token)
            .and_then(|token| match token.as_str() {
                "1" | "true" => Some(OrcaBool(true)),
                "0" | "false" => Some(OrcaBool(false)),
                _ => None,
            });
        let raw_filament_map_token = raw_settings
            .get("filament_map")
            .and_then(RawConfigValue::token);
        Self {
            enable_prime_tower,
            raw_filament_map_token,
            is_bbl,
        }
    }
}

const START: &[u8] = b"; CONFIG_BLOCK_START\n";
const END: &[u8] = b"; CONFIG_BLOCK_END\n\n";
const BANNED_KEYS: [&str; 10] = [
    "compatible_printers",
    "compatible_prints",
    // GCode.cpp (345efec64c7b):8287, GCode::append_full_config.
    "prime_volume_mode",
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
    let transformed = transformed_for_export(
        &views.full,
        &ExportOverrides::from_raw(raw_settings, is_bbl_printer(views)),
    )?;
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
    let filament_count = settings.filament.gcode.filament_diameter.0.len();
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
        // Nullable per-filament overrides echo only when the project set
        if entry.key == "extruder_colour" {
            let colour = serialize_config_value(&settings.filament.gcode.filament_colour)
                .map_err(config_error)?;
            emit(&mut lines, &entry.key, colour.token, false);
            continue;
        }
        emit(&mut lines, &entry.key, entry.token.clone(), entry.is_nil);
    }
    for entry in raw_settings.iter() {
        let (key, token) = entry.map_err(config_error)?;
        if matches!(key, "from" | "name" | "version") {
            continue;
        }
        // Legacy rule sources are consumed by the typed resolution and never
        // reach the exported config upstream: `handle_legacy` renames or
        // drops them at load (Config.cpp:573-587, PrintConfig.cpp:8033+).
        // Deferred bookkeeping rules (different_settings_to_system and the
        // cummulative keys) rename at export time instead and stay in the
        // config block.
        if EXPLICIT_RULES.iter().any(|rule| {
            rule.source == key
                && !matches!(rule.action, LegacyAction::DeferredProfileBookkeeping { .. })
        }) {
            continue;
        }
        // Unknown or alias-named keys never enter the config under that
        // name upstream: `set_deserialize_raw` accepts a key only when the
        // definition table has a definition for it, and stores an alias
        // under its canonical key (`opt_key = opt.first`, Config.cpp:608-
        // 629). Either way the name is not re-serialized into the
        // full-config block (GCode.cpp:5591-5646); canonical keys echo via
        // their own raw entries.
        if crate::options::registry::option_definition(key).is_none() {
            continue;
        }
        if entries.iter().any(|entry| entry.key == key && entry.is_nil) {
            // GCode.cpp:5632-5633: nil canonical options cannot be reinserted
            // by retained raw input after the type-owned echo filter.
            continue;
        }
        if lines.iter().any(|(existing, _)| existing.as_str() == key) {
            continue;
        }
        let Some(rendered) = token else {
            continue;
        };
        // Upstream loads every preset key into the config object and
        // re-serializes it (`Config.cpp` `ConfigOptionFloat::serialize`)
        // in the full-config block, so raw scalar Float preset echoes
        // must go through the single typed float serialization
        // (`format_number`) instead of the verbatim input spelling; a
        // non-float token is invalid input, not a fallback echo.
        // Vector Floats echoes keep the verbatim spelling until a RED
        // freezes their class (nil/empty elements, separator) against
        // upstream `ConfigOptionFloats::serialize`.
        let rendered = match crate::options::registry::option_definition(key)
            .map(|definition| definition.kind)
        {
            Some(Kind::Float) => rendered
                .parse::<f64>()
                .map(format_number)
                .map_err(|_| invalid(format!("invalid Orca option {key}: expected a float")))?,
            _ => rendered.to_owned(),
        };
        let raw_is_nil =
            crate::options::registry::option_definition(key).is_some_and(|definition| {
                use crate::options::registry::OptionValueKind as Kind;
                matches!(
                    definition.kind,
                    Kind::Ints
                        | Kind::IntsNullable
                        | Kind::Floats
                        | Kind::FloatsNullable
                        | Kind::Percents
                        | Kind::PercentsNullable
                        | Kind::Bools
                        | Kind::BoolsNullable
                ) && (rendered.is_empty()
                    || rendered.split(';').all(|value| {
                        value == "nil"
                            || (matches!(definition.kind, Kind::Ints | Kind::IntsNullable)
                                && value == "2147483647")
                    }))
            });
        emit(&mut lines, key, rendered.to_owned(), raw_is_nil);
    }
    // The resize itself is `initialize_cli_loaded_project_config`
    // (OrcaSlicer.cpp:2815-2818) on the loaded raw; this loop only
    // creates an absent key at filament_count + 2 before
    // GCode::append_full_config serializes it.
    for key in ["different_settings_to_system", "inherits_group"] {
        if !raw_settings.contains_key(key) {
            emit(&mut lines, key, ";".repeat(filament_count + 1), false);
        }
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
