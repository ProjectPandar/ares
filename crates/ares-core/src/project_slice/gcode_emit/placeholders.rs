use crate::{
    GenerationMetadata, project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

use super::value::{self, Value};

/// Placeholders available to every project G-code template (start, layer
/// change, end), mirroring the shared upstream placeholder parser state
/// (`GCode.cpp:2890-3060`). Template-specific values are added by callers.
pub(super) fn base_config(
    traversal: &PreparedPostClassicTraversal,
    metadata: GenerationMetadata,
) -> value::Config {
    let mut config =
        value::Config::from_block(traversal.config_block.as_deref().unwrap_or_default());
    insert_timestamp(&mut config, metadata);
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
    insert_bed_temperature_placeholders(&mut config);
    insert_print_bed_bounds(&mut config);
    insert_first_layer_bounds(&mut config, traversal);
    insert_adaptive_bed_mesh(&mut config);
    config
}

fn insert_timestamp(config: &mut value::Config, metadata: GenerationMetadata) {
    let (year, month, day, hour, minute, second) = metadata.timestamp();
    config.insert(
        "timestamp",
        Value::String(format!(
            "{year:04}{month:02}{day:02}-{hour:02}{minute:02}{second:02}"
        )),
    );
    for (name, number) in [
        ("year", year as f64),
        ("month", f64::from(month)),
        ("day", f64::from(day)),
        ("hour", f64::from(hour)),
        ("minute", f64::from(minute)),
        ("second", f64::from(second)),
    ] {
        config.insert(name, Value::number(number));
    }
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
    ] {
        config.insert(name, Value::number(0.0));
    }
    let filament_count = traversal.resolved.logical_filament_count.max(1);
    let first = Value::List(
        (0..filament_count)
            .map(|index| Value::number(index as f64))
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
    let extruder_count = config
        .get("nozzle_diameter")
        .map(|value| value.iter_list().count().max(1))
        .unwrap_or(1);
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
    insert_outer_wall_volumetric_speed(config, traversal);
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
) {
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
    )
    .expect("validated project outer-wall flow remains valid");
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

fn insert_first_layer_bounds(config: &mut value::Config, traversal: &PreparedPostClassicTraversal) {
    let Some((min_x, min_y, size_x, size_y)) = super::footprint::first_layer_bounds(traversal)
    else {
        return;
    };
    let point = |x, y| Value::List(vec![Value::number(x), Value::number(y)]);
    config.insert("first_layer_print_min", point(min_x, min_y));
    config.insert(
        "first_layer_print_max",
        point(min_x + size_x, min_y + size_y),
    );
    config.insert("first_layer_print_size", point(size_x, size_y));
}

/// `print_bed_min`/`print_bed_max`/`print_bed_size` from the printable-area
/// bounding box (`GCode.cpp:2908-2912`).
fn insert_adaptive_bed_mesh(config: &mut value::Config) {
    let Some(mesh_min) = point_value(config, "bed_mesh_min") else {
        return;
    };
    let Some(mesh_max) = point_value(config, "bed_mesh_max") else {
        return;
    };
    let bounds_min = point_value(config, "first_layer_print_min").unwrap_or(mesh_min);
    let bounds_max = point_value(config, "first_layer_print_max").unwrap_or(mesh_max);
    let margin = config
        .get("adaptive_bed_mesh_margin")
        .and_then(Value::as_number)
        .unwrap_or(0.0);
    let minimum = [
        mesh_min[0].max(bounds_min[0] - margin),
        mesh_min[1].max(bounds_min[1] - margin),
    ];
    let maximum = [
        mesh_max[0].min(bounds_max[0] + margin),
        mesh_max[1].min(bounds_max[1] + margin),
    ];
    let distance = point_value(config, "bed_mesh_probe_distance").unwrap_or([50.0, 50.0]);
    let mut probe_count = [
        ((maximum[0] - minimum[0]) / distance[0].max(1.0)).ceil() + 1.0,
        ((maximum[1] - minimum[1]) / distance[1].max(1.0)).ceil() + 1.0,
    ];
    probe_count[0] = probe_count[0].max(3.0);
    probe_count[1] = probe_count[1].max(3.0);
    let algorithm = if probe_count[0] * probe_count[1] <= 6.0 {
        "lagrange"
    } else {
        if config.get("gcode_flavor").map(Value::as_string).as_deref() == Some("klipper") {
            probe_count[0] = probe_count[0].max(4.0);
            probe_count[1] = probe_count[1].max(4.0);
        }
        "bicubic"
    };
    let point = |coordinates: [f64; 2]| {
        Value::List(
            coordinates
                .into_iter()
                .map(Value::number)
                .collect::<Vec<_>>(),
        )
    };
    config.insert("adaptive_bed_mesh_min", point(minimum));
    config.insert("adaptive_bed_mesh_max", point(maximum));
    config.insert("bed_mesh_probe_count", point(probe_count));
    config.insert("bed_mesh_algo", Value::String(algorithm.to_owned()));
}

fn point_value(config: &value::Config, key: &str) -> Option<[f64; 2]> {
    let value = config.get(key)?;
    Some([value.index(0)?.as_number()?, value.index(1)?.as_number()?])
}

fn insert_print_bed_bounds(config: &mut value::Config) {
    let Some(area) = config.get("printable_area") else {
        return;
    };
    let corners = area
        .iter_list()
        .filter_map(|corner| {
            let text = corner.as_string();
            let (x, y) = text.split_once('x')?;
            Some((x.parse::<f64>().ok()?, y.parse::<f64>().ok()?))
        })
        .collect::<Vec<_>>();
    if corners.len() < 3 {
        return;
    }
    let min_x = corners
        .iter()
        .map(|(x, _)| *x)
        .fold(f64::INFINITY, f64::min);
    let min_y = corners
        .iter()
        .map(|(_, y)| *y)
        .fold(f64::INFINITY, f64::min);
    let max_x = corners
        .iter()
        .map(|(x, _)| *x)
        .fold(f64::NEG_INFINITY, f64::max);
    let max_y = corners
        .iter()
        .map(|(_, y)| *y)
        .fold(f64::NEG_INFINITY, f64::max);
    let point = |x: f64, y: f64| Value::List(vec![Value::number(x), Value::number(y)]);
    config.insert("print_bed_min", point(min_x, min_y));
    config.insert("print_bed_max", point(max_x, max_y));
    config.insert("print_bed_size", point(max_x - min_x, max_y - min_y));
}
