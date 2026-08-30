use serde_json::{Map, Value};

/// Baseline option overrides for the smoke matrix. The classic wall
/// generator is the ported perimeter engine; Arachne dispatch is not
/// implemented yet and is tracked as its own slice.
pub(crate) fn smoke_overrides() -> Map<String, Value> {
    let mut overrides = Map::new();
    overrides.insert(
        "wall_generator".to_owned(),
        Value::String("classic".to_owned()),
    );
    overrides.insert("detect_thin_wall".to_owned(), Value::String("0".to_owned()));
    overrides.insert(
        "bed_exclude_area".to_owned(),
        Value::Array(vec![Value::String("0x0".to_owned())]),
    );
    overrides.insert("post_process".to_owned(), Value::Array(Vec::new()));
    overrides
}

pub(crate) fn normalize_filament_defaults(filaments: &mut [Map<String, Value>]) {
    for filament in filaments {
        let Some(value) = filament.get_mut("filament_flush_temp") else {
            continue;
        };
        if let Value::Array(values) = value {
            for value in values {
                normalize_flush_value(value);
            }
        } else {
            normalize_flush_value(value);
        }
    }
}

fn normalize_flush_value(value: &mut Value) {
    if value.is_null() || value.as_str() == Some("nil") {
        *value = Value::String("0".to_owned());
    }
}

pub(crate) fn normalize_process_defaults(
    machine: &Map<String, Value>,
    process: &mut Map<String, Value>,
) {
    // Classic fuzzy noise uses random_device and cannot be replayed across
    // independent slicer processes. Ripple is the deterministic source mode;
    // option coverage overwrites this field when testing its own enum domain.
    process.insert(
        "fuzzy_skin_noise_type".to_owned(),
        Value::String("ripple".to_owned()),
    );
    let nozzle = machine
        .get("nozzle_diameter")
        .and_then(first_number)
        .unwrap_or(0.4);
    let layer_height = process
        .get("layer_height")
        .or_else(|| machine.get("layer_height"))
        .and_then(first_number)
        .unwrap_or(0.2);
    if nozzle <= layer_height {
        for key in ["skin_infill_line_width", "skeleton_infill_line_width"] {
            process.insert(key.to_owned(), Value::String("0".to_owned()));
        }
    }
}

pub(crate) fn smoke_case_overrides(
    machine: &Map<String, Value>,
    process: &Map<String, Value>,
) -> Map<String, Value> {
    let mut overrides = smoke_overrides();
    let relative_e = machine
        .get("use_relative_e_distances")
        .is_none_or(option_true);
    let before = process
        .get("before_layer_change_gcode")
        .or_else(|| machine.get("before_layer_change_gcode"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    let layer = process
        .get("layer_change_gcode")
        .or_else(|| machine.get("layer_change_gcode"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    if relative_e && !contains_active_reset(before) && !contains_active_reset(layer) {
        let separator = if before.is_empty() || before.ends_with('\n') {
            ""
        } else {
            "\n"
        };
        overrides.insert(
            "before_layer_change_gcode".to_owned(),
            Value::String(format!("{before}{separator}G92 E0")),
        );
    } else if !relative_e {
        if let Some(normalized) = without_active_reset(before) {
            overrides.insert(
                "before_layer_change_gcode".to_owned(),
                Value::String(normalized),
            );
        }
        if let Some(normalized) = without_active_reset(layer) {
            overrides.insert("layer_change_gcode".to_owned(), Value::String(normalized));
        }
    }
    if let Some(value) = clamp(machine, "retraction_distances_when_cut", 10.0, 18.0) {
        overrides.insert("retraction_distances_when_cut".to_owned(), value);
    }
    if let Some(value) = clamp(machine, "extruder_printable_height", 0.0, 1_000.0) {
        overrides.insert("extruder_printable_height".to_owned(), value);
    }
    if machine
        .get("use_firmware_retraction")
        .is_some_and(option_true)
        && process
            .get("wipe")
            .or_else(|| machine.get("wipe"))
            .is_some_and(option_true)
    {
        overrides.insert(
            "use_firmware_retraction".to_owned(),
            Value::String("0".to_owned()),
        );
    }
    let nozzle = machine
        .get("nozzle_diameter")
        .and_then(first_number)
        .unwrap_or(0.4);
    if process
        .get("bridge_line_width")
        .and_then(first_number)
        .is_some_and(|width| width > nozzle)
    {
        overrides.insert(
            "bridge_line_width".to_owned(),
            Value::String(nozzle.to_string()),
        );
    }
    let layer_height = process
        .get("layer_height")
        .or_else(|| machine.get("layer_height"))
        .and_then(first_number)
        .unwrap_or(0.2);
    for key in [
        "line_width",
        "bridge_line_width",
        "support_line_width",
        "inner_wall_line_width",
        "outer_wall_line_width",
        "sparse_infill_line_width",
        "internal_solid_infill_line_width",
        "top_surface_line_width",
        "skin_infill_line_width",
        "skeleton_infill_line_width",
    ] {
        if process
            .get(key)
            .or_else(|| machine.get(key))
            .and_then(|value| absolute_number(value, nozzle))
            .is_some_and(|width| width > 0.0 && width <= layer_height)
        {
            overrides.insert(key.to_owned(), Value::String("0".to_owned()));
        }
    }
    if nozzle <= layer_height {
        for key in [
            "bridge_line_width",
            "skin_infill_line_width",
            "skeleton_infill_line_width",
        ] {
            overrides.insert(key.to_owned(), Value::String("0".to_owned()));
        }
    }
    if let Some(source) = machine.get("machine_start_gcode").and_then(Value::as_str) {
        let mut normalized = source.replace("[output_filename_format]", "[input_filename_base]");
        for placeholder in [
            "extruder_rotation_volume[0]",
            "mixing_stepper_rotation_volume[0]",
            "multi_zone_1_initial_layer[0]",
            "multi_zone_2_initial_layer[0]",
            "multi_zone_3_initial_layer[0]",
        ] {
            normalized = normalized.replace(&format!("{{{placeholder}}}"), "0");
        }
        normalized = normalize_constant_random_calls(&normalized);
        if normalized != source {
            overrides.insert("machine_start_gcode".to_owned(), Value::String(normalized));
        }
    }
    overrides
}

fn contains_active_reset(template: &str) -> bool {
    template.lines().any(|line| {
        let mut words = line.split(';').next().unwrap_or_default().split_whitespace();
        matches!((words.next(), words.next(), words.next()), (Some("G92"), Some(value), None) if value.strip_prefix('E').and_then(|value| value.parse::<f64>().ok()) == Some(0.0))
    })
}

fn without_active_reset(template: &str) -> Option<String> {
    contains_active_reset(template).then(|| {
        template
            .lines()
            .filter(|line| !contains_active_reset(line))
            .collect::<Vec<_>>()
            .join("\n")
    })
}

fn normalize_constant_random_calls(source: &str) -> String {
    let pattern =
        regex::Regex::new(r"random\(\s*(-?\d+(?:\.\d+)?)\s*,\s*-?\d+(?:\.\d+)?\s*\)").unwrap();
    pattern.replace_all(source, "$1").into_owned()
}

fn option_true(value: &Value) -> bool {
    match value {
        Value::Array(values) => values.first().is_some_and(option_true),
        Value::Bool(value) => *value,
        Value::String(value) => value == "1" || value == "true",
        Value::Number(value) => value.as_i64() == Some(1),
        Value::Null | Value::Object(_) => false,
    }
}

fn absolute_number(value: &Value, basis: f64) -> Option<f64> {
    match value {
        Value::Array(values) => values
            .first()
            .and_then(|value| absolute_number(value, basis)),
        Value::String(value) if value.ends_with('%') => value
            .trim_end_matches('%')
            .parse::<f64>()
            .ok()
            .map(|percent| percent * basis / 100.0),
        _ => first_number(value),
    }
}

fn first_number(value: &Value) -> Option<f64> {
    match value {
        Value::Array(values) => values.first().and_then(first_number),
        Value::String(value) if !value.ends_with('%') => value.parse().ok(),
        Value::Number(value) => value.as_f64(),
        _ => None,
    }
}

fn clamp(fields: &Map<String, Value>, key: &str, minimum: f64, maximum: f64) -> Option<Value> {
    let value = fields.get(key)?;
    match value {
        Value::Array(values) => {
            let mut changed = false;
            let values = values
                .iter()
                .map(|value| clamp_number(value, minimum, maximum, &mut changed))
                .collect();
            changed.then_some(Value::Array(values))
        }
        value => {
            let mut changed = false;
            let value = clamp_number(value, minimum, maximum, &mut changed);
            changed.then_some(value)
        }
    }
}

fn clamp_number(value: &Value, minimum: f64, maximum: f64, changed: &mut bool) -> Value {
    let Some(number) = first_number(value) else {
        return value.clone();
    };
    let clamped = number.clamp(minimum, maximum);
    *changed |= clamped != number;
    Value::String(if clamped.fract() == 0.0 {
        format!("{clamped:.0}")
    } else {
        clamped.to_string()
    })
}
