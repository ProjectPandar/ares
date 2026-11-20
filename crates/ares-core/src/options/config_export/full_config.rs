//! GCode.cpp:8244-8354 (345efec64c7b): applied full config projection.
use std::collections::HashSet;

use crate::options::config_export::collector::ConfigEntry;
use crate::options::config_export::value::serialize_config_value;
use crate::options::config_export::writer::append_line;
use crate::options::config_types::format_number;
use crate::options::registry::OptionValueKind as Kind;
use crate::options::{EXPLICIT_RULES, LegacyAction, registry};
use crate::project::raw_settings::ProjectSettingsRaw;
use crate::{ProjectSettings, SliceError};

const BANNED_KEYS: &[&str] = &[
    "filament_extruder_compatibility",
    "prime_volume_mode",
    "flush_multiplier_fast",
    "filament_flush_temp_fast",
    "support_fast_purge_mode",
    "deretract_speed_extruder_change",
    "farthest_point_timelapse",
    "is_imex",
    "imex_firmware_managed_zones",
    "imex_gantry_count",
    "imex_tools_per_gantry",
    "imex_tool_layout",
    "imex_nozzle_clearance_x",
    "imex_nozzle_clearance_y",
    "imex_carriage_margin",
    "imex_viz_theme",
    "imex_mode_names",
    "imex_mode_active_tools",
    "imex_mode_gcodes",
    "imex_parallel_mode",
    "imex_head_filament_map",
    "compatible_printers",
    "compatible_prints",
    "filament_colour_type",
    "print_host",
    "print_host_webui",
    "printhost_apikey",
    "printhost_cafile",
    "printhost_user",
    "printhost_password",
    "printhost_port",
];

pub(super) fn write_entries(
    settings: &ProjectSettings,
    plate_index: usize,
    entries: &[ConfigEntry],
    raw_settings: &ProjectSettingsRaw,
    output: &mut Vec<u8>,
) -> Result<(), SliceError> {
    // GCode.cpp:8338-8351: serialize the applied FullPrintConfig, not a
    // historical output-key list. Registry ownership supplies absent defaults.
    let filament_count = settings.filament.gcode.filament_diameter.0.len();
    let mut lines: Vec<(String, String)> = Vec::new();
    let entry_keys: HashSet<&str> = entries.iter().map(|entry| entry.key.as_str()).collect();
    let nil_entry_keys: HashSet<&str> = entries
        .iter()
        .filter(|entry| entry.is_nil)
        .map(|entry| entry.key.as_str())
        .collect();
    let mut emitted_keys = HashSet::with_capacity(registry::option_metadata_table().len());
    fn emit<'a>(
        lines: &mut Vec<(String, String)>,
        emitted_keys: &mut HashSet<&'a str>,
        key: &'a str,
        token: String,
        is_nil: bool,
    ) {
        if BANNED_KEYS.contains(&key) || is_nil {
            return;
        }
        emitted_keys.insert(key);
        lines.push((key.to_owned(), token));
    }
    for entry in entries {
        if matches!(entry.key.as_str(), "wipe_tower_x" | "wipe_tower_y") {
            // GCode.cpp:8340-8341 selects the plate's fixed-precision origin.
            let values = if entry.key == "wipe_tower_x" {
                &settings.project.print.wipe_tower_x.0
            } else {
                &settings.project.print.wipe_tower_y.0
            };
            let value = values
                .get(plate_index)
                .or_else(|| values.first())
                .ok_or_else(|| invalid(format!("{} must not be empty", entry.key)))?;
            emit(
                &mut lines,
                &mut emitted_keys,
                &entry.key,
                format!("{:.3}", value.0),
                false,
            );
            // GCode.cpp:8342-8347 also writes the ordinary serialization.
        }
        // Preset metadata that was never applied is not a config member.
        let known = registry::option_definition(&entry.key).is_some_and(|definition| {
            definition.full_config_default || raw_settings.contains_key(&entry.key)
        });
        if !known {
            continue;
        }
        if entry.key == "extruder_colour" {
            let colour = serialize_config_value(&settings.filament.gcode.filament_colour)
                .map_err(config_error)?;
            emit(
                &mut lines,
                &mut emitted_keys,
                &entry.key,
                colour.token,
                false,
            );
            continue;
        }
        emit(
            &mut lines,
            &mut emitted_keys,
            &entry.key,
            entry.token.clone(),
            entry.is_nil,
        );
    }
    for entry in raw_settings.iter() {
        let (key, token) = entry.map_err(config_error)?;
        if matches!(key, "from" | "name" | "version") {
            continue;
        }
        // PrintConfig.cpp:10002-10035 consumes obsolete/renamed input keys.
        // Deferred bookkeeping remains part of the applied config.
        if EXPLICIT_RULES.iter().any(|rule| {
            rule.source == key
                && !matches!(rule.action, LegacyAction::DeferredProfileBookkeeping { .. })
        }) {
            continue;
        }
        // GCode.cpp:8338-8351 only sees registered canonical config keys.
        if crate::options::registry::option_definition(key).is_none() {
            continue;
        }
        if nil_entry_keys.contains(key) {
            // GCode.cpp:8339: applied nil options cannot be reinserted by raw input.
            continue;
        }
        if emitted_keys.contains(key) {
            continue;
        }
        let Some(rendered) = token else {
            continue;
        };
        // Raw vectors already use their type-owned serializer and nil policy;
        // scalar Float echoes use the same numeric formatter as typed values.
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
        emit(
            &mut lines,
            &mut emitted_keys,
            key,
            rendered.to_owned(),
            raw_is_nil,
        );
    }
    // FullPrintConfig/default preset construction owns absent defaults, not a
    // historical dump-key whitelist. Nil applied values must not be resurrected.
    for metadata in registry::option_metadata_table() {
        let definition = registry::option_definition(metadata.key).unwrap();
        if !definition.full_config_default
            || definition.default_is_nil
            || raw_settings.contains_key(metadata.key)
            || entry_keys.contains(metadata.key)
            || emitted_keys.contains(metadata.key)
        {
            continue;
        }
        emit(
            &mut lines,
            &mut emitted_keys,
            metadata.key,
            definition.default_wire_value.to_owned(),
            false,
        );
    }
    applied_filament_maps(settings, raw_settings, &mut lines);
    // The resize itself is `initialize_cli_loaded_project_config`
    // (OrcaSlicer.cpp:3202-3205) on the loaded raw; this loop only
    // creates an absent key at filament_count + 2 before
    // GCode::append_full_config serializes it.
    for key in ["different_settings_to_system", "inherits_group"] {
        if !raw_settings.contains_key(key) {
            emit(
                &mut lines,
                &mut emitted_keys,
                key,
                ";".repeat(filament_count + 1),
                false,
            );
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

fn applied_filament_maps(
    settings: &ProjectSettings,
    raw: &ProjectSettingsRaw,
    lines: &mut Vec<(String, String)>,
) {
    use crate::options::project_variants::{canonical_variant, get_index_for_extruder};
    let filament_map = &settings.project.gcode.filament_map.0;
    let extruders = &settings.printer.gcode.extruder_type.0;
    let volumes = &settings.project.gcode.nozzle_volume_type.0;
    // PrintApply.cpp:1603-1627 / GCode.cpp:8274-8275: copy applied slots,
    // not the registered {1} placeholder. Reuse the existing variant resolver.
    let slots = filament_map
        .iter()
        .map(|id| {
            let physical = (id.0 - 1) as usize;
            let extruder = extruders.get(physical).unwrap_or(&extruders[0]);
            let volume = volumes.get(physical).unwrap_or(&volumes[0]);
            let variant = canonical_variant(*extruder, *volume);
            get_index_for_extruder(
                (id.0, &variant),
                Some(&settings.process.region.print_extruder_id.0),
                &settings.process.region.print_extruder_variant.0,
                &settings.printer.remaining.extruder_variant_list.0,
                1,
            )
            .unwrap_or(physical)
            .to_string()
        })
        .collect::<Vec<_>>()
        .join(",");
    lines.retain(|(key, _)| key != "filament_map_2");
    lines.push(("filament_map_2".to_owned(), slots));

    let multiple_nozzles = raw
        .get("extruder_max_nozzle_count")
        .and_then(|value| value.token())
        .is_some_and(|value| {
            value
                .split([';', ','])
                .any(|element| element.parse::<i32>().is_ok_and(|count| count > 1))
        });
    // ToolOrdering.cpp:1768-1773,1892-1907 and 3035-3087: publish the
    // applied physical-nozzle map, including expanded multi-nozzle groups.
    let nozzles = settings
        .project
        .gcode
        .filament_nozzle_map
        .0
        .iter()
        .map(|id| id.0.to_string())
        .collect::<Vec<_>>()
        .join(",");
    lines.retain(|(key, _)| key != "filament_nozzle_map");
    lines.push(("filament_nozzle_map".to_owned(), nozzles));
    if !multiple_nozzles {
        let volumes = settings
            .project
            .gcode
            .filament_volume_map
            .0
            .iter()
            .map(|value| value.0.to_string())
            .collect::<Vec<_>>()
            .join(",");
        lines.retain(|(key, _)| key != "filament_volume_map");
        lines.push(("filament_volume_map".to_owned(), volumes));
    }
}

fn config_error(error: impl std::fmt::Display) -> SliceError {
    invalid(error.to_string())
}
fn invalid(message: impl Into<String>) -> SliceError {
    SliceError::InvalidInput(message.into())
}
