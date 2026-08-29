mod bounds;
mod timestamp;

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
) -> Result<value::Config, SliceError> {
    let mut config =
        value::Config::from_block(traversal.config_block.as_deref().unwrap_or_default());
    timestamp::insert(&mut config, metadata);
    insert_runtime_placeholders(&mut config, traversal);
    config.insert("current_extruder", Value::number(0.0));
    config.insert("current_hotend", Value::number(-1.0));
    for (target, source) in [
        ("flush_temperatures", "nozzle_temperature_range_high"),
        ("flush_volumetric_speeds", "filament_max_volumetric_speed"),
        (
            "first_layer_temperature",
            "nozzle_temperature_initial_layer",
        ),
        ("max_print_height", "printable_height"),
        ("print_preset", "print_settings_id"),
        ("filament_preset", "filament_settings_id"),
        ("printer_preset", "printer_settings_id"),
        ("physical_printer_preset", "printer_settings_id"),
    ] {
        if let Some(value) = config.get(source).cloned() {
            config.insert(target, value);
        }
    }
    // `retract_length` is exposed to templates from the filament retraction
    // length (`GCode.cpp:2898`).
    let retract_length = &traversal.resolved.views.runtime_gcode.retraction_length;
    config.insert(
        "retract_length",
        value::Value::List(
            retract_length
                .0
                .iter()
                .map(|value| value::Value::number(value.0))
                .collect(),
        ),
    );
    insert_bed_temperature_placeholders(&mut config);
    bounds::insert_print_bed_bounds(&mut config);
    bounds::insert_first_layer_bounds(&mut config, first_layer_bounds);
    insert_outer_wall_volumetric_speed(&mut config, traversal)?;
    bounds::insert_adaptive_bed_mesh(&mut config);
    bounds::insert_head_wrap_detect_zone(&mut config, traversal);
    Ok(config)
}

fn insert_runtime_placeholders(
    config: &mut value::Config,
    traversal: &PreparedPostClassicTraversal,
) {
    for name in [
        "initial_tool",
        "initial_extruder",
        "initial_no_support_tool",
        "initial_no_support_extruder",
        "initial_no_support_hotend",
        "total_toolchanges",
        "current_object_idx",
        "layer_num",
        "layer_z",
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
            .map(|index| Value::number(index.min(filament_count - 1) as f64))
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
                .map(|index| Value::Bool(used.contains(&index)))
                .collect(),
        ),
    );
    config.insert("has_wipe_tower", Value::Bool(false));
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
    for (target, source) in [
        (
            "retraction_distance_when_cut",
            "retraction_distances_when_cut",
        ),
        ("long_retraction_when_cut", "long_retractions_when_cut"),
        (
            "retraction_distance_when_ec",
            "retraction_distances_when_ec",
        ),
        ("long_retraction_when_ec", "long_retractions_when_ec"),
    ] {
        if let Some(value) = config.get(source).and_then(|value| value.index(0)).cloned() {
            config.insert(target, value);
        }
    }
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
        Value::Bool(false),
    );
    let layer_count = traversal
        .objects
        .iter()
        .map(|object| object.records.len())
        .max()
        .unwrap_or(0);
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
    config.insert("max_print_z", Value::number(max_print_z));
    if let Some(value) = config.get("initial_layer_print_height").cloned() {
        config.insert("first_layer_height", value);
    }
    if let Some(value) = config.get("nozzle_temperature").cloned() {
        config.insert("temperature", value);
    }
    let has_tpu = config
        .get("filament_type")
        .is_some_and(|value| value.iter_list().any(|item| item.as_string() == "TPU"));
    config.insert("has_tpu_in_first_layer", Value::Bool(has_tpu));
    let all_bbl = config.get("filament_vendor").is_some_and(|value| {
        let mut vendors = value.iter_list();
        vendors.clone().next().is_some() && vendors.all(|vendor| vendor.as_string() == "Bambu Lab")
    });
    config.insert("is_all_bbl_filament", Value::Bool(all_bbl));
    if let Some(minimum) = config
        .get("temperature_vitrification")
        .into_iter()
        .flat_map(Value::iter_list)
        .filter_map(Value::as_number)
        .reduce(f64::min)
    {
        config.insert("min_vitrification_temperature", Value::number(minimum));
    }
}

fn insert_outer_wall_volumetric_speed(
    config: &mut value::Config,
    traversal: &PreparedPostClassicTraversal,
) -> Result<(), SliceError> {
    let full = &traversal.resolved.views.full;
    let region = &full.process.region;
    let object = &full.process.object;
    let selected_width = match region.outer_wall_line_width {
        crate::FloatOrPercent::Float(0.0) => object.line_width,
        value => value,
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
            _ => value.iter_list().filter_map(|item| item.as_number()).fold(
                None::<f64>,
                |acc, next| {
                    Some(match acc {
                        Some(current) => current.max(next),
                        None => next,
                    })
                },
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
