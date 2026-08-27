use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;

use super::value::{self, Value};

/// Placeholders available to every project G-code template (start, layer
/// change, end), mirroring the shared upstream placeholder parser state
/// (`GCode.cpp:2890-3060`). Template-specific values are added by callers.
pub(super) fn base_config(traversal: &PreparedPostClassicTraversal) -> value::Config {
    let mut config =
        value::Config::from_block(traversal.config_block.as_deref().unwrap_or_default());
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
    ] {
        if let Some(value) = config.get(source).cloned() {
            config.insert(target, value);
        }
    }
    insert_bed_temperature_placeholders(&mut config);
    insert_print_bed_bounds(&mut config);
    config
}

/// `bed_temperature_initial_layer[_single]` from the curr-bed-type
/// first-layer key (`GCode.cpp:3020-3034`).
fn insert_bed_temperature_placeholders(config: &mut value::Config) {
    if let Some(value) = bed_type_first_layer_temperature(config) {
        config.insert("bed_temperature_initial_layer", value.clone());
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
}

fn bed_type_first_layer_temperature(config: &value::Config) -> Option<Value> {
    let bed_type = config
        .get("curr_bed_type")
        .map_or_else(|| "Cool Plate".to_owned(), |value| value.as_string());
    let key = crate::options::first_layer_bed_temperature_key_for(&bed_type)?;
    config.get(key).cloned()
}

/// `print_bed_min`/`print_bed_max`/`print_bed_size` from the printable-area
/// bounding box (`GCode.cpp:2908-2912`).
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
