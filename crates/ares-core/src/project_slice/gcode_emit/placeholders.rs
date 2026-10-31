mod bounds;
mod cut_retraction;

mod timestamp;

pub(super) use cut_retraction::insert_aliases as refresh_filament_aliases;

use crate::{
    GenerationMetadata, SliceError,
    project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

use super::value::{self, Value};

/// Placeholders available to every project G-code template (start, layer
/// change, end), mirroring the shared upstream placeholder parser state
/// (`GCode.cpp:2890-3060`). Template-specific values are added by callers.
pub(super) fn base_config(
    traversal: &PreparedPostClassicTraversal,
    metadata: GenerationMetadata,
    first_layer_bounds: Option<super::footprint::FirstLayerBounds>,
    rng: Option<&value::TemplateRng>,
    layer_count: usize,
) -> Result<value::Config, SliceError> {
    let mut config =
        value::Config::from_block(traversal.config_block.as_deref().unwrap_or_default());
    config.random = rng.cloned();
    timestamp::insert(&mut config, metadata);
    insert_runtime_placeholders(&mut config, traversal, layer_count);
    config.insert("current_extruder", Value::number(0.0));
    config.insert("filament_extruder_id", Value::number(0.0));
    config.insert("current_hotend", Value::number(-1.0));
    for (target, source) in [
        (
            "first_layer_temperature",
            "nozzle_temperature_initial_layer",
        ),
        ("print_preset", "print_settings_id"),
        ("filament_preset", "filament_settings_id"),
        ("printer_preset", "printer_settings_id"),
        ("physical_printer_preset", "printer_settings_id"),
    ] {
        if let Some(value) = config.get(source).cloned() {
            config.insert(target, value);
        }
    }
    // `max_print_height` is an INT placeholder upstream
    // (`GCode.cpp:3045`: `ConfigOptionInt(printable_height)`) — its
    // template arithmetic divides as integer division
    // (`PlaceholderParser.cpp:496-503`), e.g.
    // `{max_print_height / 2 + 10}` truncates.
    if let Some(height) = config
        .get("printable_height")
        .and_then(value::Value::as_number)
    {
        config.insert("max_print_height", value::Value::Integer(height as i64));
    }
    let first_layer_temperatures = value::Value::List(
        traversal
            .resolved
            .views
            .full
            .filament
            .print
            .nozzle_temperature_initial_layer
            .0
            .iter()
            .map(|value| value::Value::Integer(i64::from(value.0)))
            .collect(),
    );
    config.insert("first_layer_temperature", first_layer_temperatures.clone());
    config.insert("nozzle_temperature_initial_layer", first_layer_temperatures);
    insert_flush_placeholders(&mut config, traversal);
    // `retract_length` is exposed to templates from the filament retraction
    // length (`GCode.cpp:2898`).
    let retract_length = &traversal.resolved.views.runtime_gcode.retraction_length;
    let retract_length = value::Value::List(
        retract_length
            .0
            .iter()
            .map(|value| value::Value::number(value.0))
            .collect(),
    );
    config.insert("retract_length", retract_length.clone());
    config.insert("retraction_length", retract_length);
    insert_bed_temperature_placeholders(&mut config);
    bounds::insert_print_bed_bounds(&mut config);
    bounds::insert_first_layer_bounds(&mut config, first_layer_bounds);
    insert_outer_wall_volumetric_speed(&mut config, traversal)?;
    bounds::insert_adaptive_bed_mesh(&mut config);
    bounds::insert_head_wrap_detect_zone(&mut config, traversal);
    Ok(config)
}

fn insert_flush_placeholders(config: &mut value::Config, traversal: &PreparedPostClassicTraversal) {
    let filament = &traversal.resolved.views.full.filament.gcode;
    let print = &traversal.resolved.views.full.filament.print;
    let speeds = filament
        .filament_flush_volumetric_speed
        .iter()
        .enumerate()
        .map(|(index, value)| flush_speed(value, &filament.filament_max_volumetric_speed, index))
        .map(Value::number)
        .collect();
    config.insert("flush_volumetric_speeds", Value::List(speeds));

    let temperatures = filament
        .filament_flush_temp
        .iter()
        .enumerate()
        .map(|(index, value)| flush_temperature(value, &print.nozzle_temperature_range_high, index))
        .map(|value| Value::number(f64::from(value)))
        .collect();
    config.insert("flush_temperatures", Value::List(temperatures));
}

fn flush_speed(
    value: &crate::Nullable<crate::OrcaFloat>,
    fallback: &crate::OrcaFloats,
    index: usize,
) -> f64 {
    match value {
        crate::Nullable::Value(value) if value.0 != 0.0 => value.0,
        _ => float_at(fallback, index),
    }
}

fn flush_temperature(
    value: &crate::Nullable<crate::OrcaInt>,
    fallback: &crate::OrcaInts,
    index: usize,
) -> i32 {
    match value {
        crate::Nullable::Value(value) if value.0 != 0 => value.0,
        _ => int_at(fallback, index),
    }
}

fn float_at(values: &crate::OrcaFloats, index: usize) -> f64 {
    values
        .0
        .get(index)
        .or_else(|| values.0.first())
        .map_or(0.0, |value| value.0)
}

fn int_at(values: &crate::OrcaInts, index: usize) -> i32 {
    values
        .0
        .get(index)
        .or_else(|| values.0.first())
        .map_or(0, |value| value.0)
}

// `layer_num`/`layer_z` belong to per-site DynamicConfig overlays only:
// GCode.cpp:783-784,853-854,1359-1360,3149,3476-3478,4636-4684,
// 5159-5160,5219-5220,6755 (plus set_extruder's filament overlays).
// Base-only renders such as machine_pause_gcode/template_custom_gcode
// (:4240/:4246) must reject those names.
fn insert_runtime_placeholders(
    config: &mut value::Config,
    traversal: &PreparedPostClassicTraversal,
    layer_count: usize,
) {
    for name in [
        "initial_tool",
        "initial_extruder",
        "initial_no_support_tool",
        "initial_no_support_extruder",
        "initial_no_support_hotend",
        "total_toolchanges",
        "current_object_idx",
    ] {
        config.insert(name, Value::number(0.0));
    }
    let filament_count = traversal.resolved.logical_filament_count.max(1);
    let extruder_count = config
        .get("nozzle_diameter")
        .map(|value| value.iter_list().count().max(1))
        .unwrap_or(1);
    let first = Value::List(
        (0..filament_count.max(extruder_count))
            .map(|index| Value::Integer((index.min(filament_count - 1)) as i64))
            .collect(),
    );
    for name in [
        "first_tools",
        "first_filaments",
        "first_non_support_tools",
        "first_non_support_filaments",
    ] {
        config.insert(name, first.clone());
    }
    config.insert("num_extruders", Value::number(extruder_count as f64));
    let used = crate::project_slice::extruders::collect_project_object_extruders(
        traversal.project.objects(),
        &traversal.resolved.objects,
        traversal.resolved.logical_filament_count,
    )
    .into_iter()
    .flatten()
    .collect::<std::collections::BTreeSet<_>>();
    let usage_len = config
        .get("filament_diameter")
        .map(|value| value.iter_list().count())
        .unwrap_or(0)
        .max(64);
    config.insert(
        "is_extruder_used",
        Value::List(
            (0..usage_len)
                .map(|index| Value::option_bool(used.contains(&index)))
                .collect(),
        ),
    );
    config.insert("has_wipe_tower", Value::option_bool(false));
    config.insert("plate_name", Value::String(String::new()));
    config.insert("plate_number", Value::String("1".to_owned()));
    let model_name = traversal
        .project
        .objects()
        .first()
        .map_or_else(String::new, |object| object.name().to_owned());
    let input_filename = model_name
        .rsplit_once('.')
        .map_or(model_name.as_str(), |(stem, _)| stem)
        .to_owned();
    config.insert("model_name", Value::String(model_name));
    config.insert("input_filename_base", Value::String(input_filename));
    config.insert("zhop", Value::number(0.0));
    cut_retraction::insert(config, &traversal.resolved.views.runtime_gcode);
    refresh_filament_aliases(config, 0);
    config.insert(
        "position",
        Value::List(vec![
            Value::number(0.0),
            Value::number(0.0),
            Value::number(0.0),
        ]),
    );
    config.insert(
        "has_single_extruder_multi_material_priming",
        Value::option_bool(false),
    );
    config.insert("total_layer_count", Value::number(layer_count as f64));
    let max_print_z = traversal
        .objects
        .iter()
        .map(|object| {
            object
                .records
                .iter()
                .filter_map(Option::as_ref)
                .map(|record| record.layer_height)
                .sum::<f64>()
        })
        .fold(0.0, f64::max);
    // GCode.cpp:2968-2973: one shared integer-ceiling binding for all templates.
    config.insert("max_print_z", Value::Integer(max_print_z.ceil() as i64));
    if let Some(value) = config.get("initial_layer_print_height").cloned() {
        config.insert("first_layer_height", value);
    }
    if let Some(value) = config.get("nozzle_temperature").cloned() {
        config.insert("temperature", value);
    }
    let has_tpu = config
        .get("filament_type")
        .is_some_and(|value| value.iter_list().any(|item| item.as_string() == "TPU"));
    config.insert("has_tpu_in_first_layer", Value::option_bool(has_tpu));
    let all_bbl = config.get("filament_vendor").is_some_and(|value| {
        let mut vendors = value.iter_list();
        vendors.clone().next().is_some() && vendors.all(|vendor| vendor.as_string() == "Bambu Lab")
    });
    config.insert("is_all_bbl_filament", Value::option_bool(all_bbl));
    if let Some(minimum) = config
        .get("temperature_vitrification")
        .into_iter()
        .flat_map(Value::iter_list)
        .filter_map(Value::as_number)
        .reduce(f64::min)
    {
        config.insert(
            "min_vitrification_temperature",
            Value::Integer(minimum as i64),
        );
    }
    // `GCode.cpp:3009-3037`: `overall_chamber_temperature` is the maximum
    // `chamber_temperature` over the writer's extruders (the used
    // filaments). Registered as an INT placeholder.
    if let Some(value) = config.get("chamber_temperature") {
        let overall = value
            .iter_list()
            .enumerate()
            .filter(|(index, _)| used.contains(index))
            .filter_map(|(_, item)| item.as_number())
            .fold(0.0_f64, f64::max);
        config.insert(
            "overall_chamber_temperature",
            Value::Integer(overall as i64),
        );
    }
    // `GCode.cpp:2729-2743,2871` + `ToolOrdering::cal_max_additional_fan`
    // (`ToolOrdering.cpp:983-996`): maximum `additional_cooling_fan_speed`
    // over the filaments used on any layer. Registered as a FLOAT
    // placeholder (upstream stores `float`).
    if let Some(value) = config.get("additional_cooling_fan_speed") {
        let max_fan = value
            .iter_list()
            .enumerate()
            .filter(|(index, _)| used.contains(index))
            .filter_map(|(_, item)| item.as_number())
            .fold(0.0_f64, f64::max);
        config.insert("max_additional_fan", Value::number(max_fan));
    }
}

fn insert_outer_wall_volumetric_speed(
    config: &mut value::Config,
    traversal: &PreparedPostClassicTraversal,
) -> Result<(), SliceError> {
    let full = &traversal.resolved.views.full;
    let region = &full.process.region;
    let object = &full.process.object;
    let selected_width = if region.outer_wall_line_width.is_non_positive() {
        object.line_width
    } else {
        region.outer_wall_line_width
    };
    let nozzle = full
        .project
        .print
        .nozzle_diameter
        .0
        .first()
        .map_or(0.4, |diameter| diameter.0) as f32;
    let flow = crate::project_slice::perimeters::flow::build_nonbridging_flow(
        selected_width,
        object.layer_height.0 as f32,
        nozzle,
    )?;
    let maximum = full
        .filament
        .gcode
        .filament_max_volumetric_speed
        .0
        .first()
        .map_or(0.0, |speed| speed.0);
    config.insert(
        "outer_wall_volumetric_speed",
        Value::number((region.outer_wall_speed.0 * flow.mm3_per_mm).min(maximum)),
    );
    Ok(())
}

/// `bed_temperature_initial_layer[_single]` from the curr-bed-type
/// first-layer key (`GCode.cpp:3020-3034`).
fn insert_bed_temperature_placeholders(config: &mut value::Config) {
    if let Some(value) = bed_type_first_layer_temperature(config) {
        config.insert("bed_temperature_initial_layer", value.clone());
        config.insert("first_layer_bed_temperature", value.clone());
        let single = match config.get("bed_temperature_formula").map(Value::as_string) {
            Some(formula) if formula == "by_first_filament" => {
                value.index(0).and_then(|item| item.as_number())
            }
            _ => Some(
                value
                    .iter_list()
                    .filter_map(|item| item.as_number())
                    .fold(0.0, f64::max),
            ),
        };
        if let Some(single) = single {
            config.insert(
                "bed_temperature_initial_layer_single",
                Value::number(single),
            );
        }
    }
    if let Some(key) = bed_type_temperature_key(config)
        && let Some(value) = config.get(key).cloned()
    {
        config.insert("bed_temperature", value);
    }
}

fn bed_type_first_layer_temperature(config: &value::Config) -> Option<Value> {
    let bed_type = config
        .get("curr_bed_type")
        .map_or_else(|| "Cool Plate".to_owned(), |value| value.as_string());
    let key = crate::options::first_layer_bed_temperature_key_for(&bed_type)?;
    config.get(key).cloned()
}

fn bed_type_temperature_key(config: &value::Config) -> Option<&'static str> {
    let bed_type = config
        .get("curr_bed_type")
        .map_or_else(|| "Cool Plate".to_owned(), Value::as_string);
    crate::options::first_layer_bed_temperature_key_for(&bed_type)?.strip_suffix("_initial_layer")
}
