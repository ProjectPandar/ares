//! Emitted positive wall XY/E crossings of y=110 witness independently authored
//! prism/step/island fixtures. Full ordered bytes remain the parity gate.
use serde_json::{Value, json};

pub(super) fn capture(bytes: &[u8]) -> Vec<Value> {
    let mut layers = Vec::new();
    let mut position = [0.0; 2];
    let mut extrusion = 0.0;
    let mut relative_e = false;
    let mut role = "";
    let mut crossings = Vec::new();
    let mut wall_segments = 0;
    let mut infill_segments = 0;
    for line in std::str::from_utf8(bytes).unwrap().lines() {
        if line == ";LAYER_CHANGE" {
            if !layers.is_empty() {
                *layers.last_mut().unwrap() = json!({"crossings":crossings,
                    "wall_segments":wall_segments,"infill_segments":infill_segments});
            }
            layers.push(Value::Null);
            crossings = Vec::new();
            wall_segments = 0;
            infill_segments = 0;
        }
        if let Some(value) = line.strip_prefix(";TYPE:") {
            role = value;
        }
        let mut words = line.split(';').next().unwrap().split_whitespace();
        let Some(command) = words.next() else {
            continue;
        };
        if command == "M83" {
            relative_e = true;
        }
        if command == "M82" {
            relative_e = false;
        }
        if !matches!(command, "G0" | "G1" | "G92") {
            continue;
        }
        let start = position;
        let mut e_value = None;
        for word in words {
            let (axis, number) = word.split_at(1);
            let Ok(value) = number.parse::<f64>() else {
                continue;
            };
            match axis {
                "X" => position[0] = value,
                "Y" => position[1] = value,
                "E" => e_value = Some(value),
                _ => {}
            }
        }
        let delta = match e_value {
            Some(value) if relative_e => value,
            Some(value) => value - extrusion,
            None => 0.0,
        };
        if let Some(value) = e_value {
            extrusion = value;
        }
        if command != "G1" || delta <= 0.0 || start == position || layers.is_empty() {
            continue;
        }
        if matches!(role, "Outer wall" | "Inner wall") {
            wall_segments += 1;
            if (start[1] > 110.0) != (position[1] > 110.0) {
                crossings.push(
                    start[0]
                        + (110.0 - start[1]) * (position[0] - start[0]) / (position[1] - start[1]),
                );
            }
        }
        if role == "Sparse infill" {
            infill_segments += 1;
        }
    }
    *layers.last_mut().unwrap() = json!({"crossings":crossings,
        "wall_segments":wall_segments,"infill_segments":infill_segments});
    layers
}

/// `PerimeterGenerator.cpp:2703-2706`: the configured walls, plus one on odd
/// layers when alternating over a nonzero integer density. The single-wall
/// overrides at `:2710-2716` need shell layers (`:618-631`, `:2681-2682`), and
/// every non-spiral fixture here has none, so they do not enter the count.
fn wall_count(settings: &Value, layer_id: usize) -> usize {
    let number = |key: &str| {
        settings[key]
            .as_str()
            .unwrap()
            .trim_end_matches('%')
            .parse::<f64>()
            .unwrap()
    };
    let alternate = settings["alternate_extra_wall"] == "1"
        && layer_id % 2 == 1
        && number("sparse_infill_density") as i32 > 0;
    number("wall_loops") as usize + usize::from(alternate)
}

pub(super) fn verify(label: &str, cases: &Value, off: &[Value], on: &[Value]) {
    if label == "spiral" {
        // SpiralVase.cpp:123,153 adds the final constant-Z transition-out.
        assert_eq!(off.len(), 7, "Native spiral transition-out layer");
        assert_eq!(on.len(), 7, "Native spiral transition-out layer");
        assert_eq!(off, on, "effective spiral control");
        assert!(
            off.iter()
                .all(|layer| layer["wall_segments"].as_u64().unwrap() > 0)
        );
        return;
    }
    assert_eq!(off.len(), 6, "Native off layer count: {label}");
    assert_eq!(on.len(), 6, "Native on layer count: {label}");
    // Two y=110 crossings per wall of each independent contour.
    let islands = if label == "two-islands" { 2 } else { 1 };
    for (mode, layers) in [("off", off), ("on", on)] {
        let settings = &cases[mode]["settings"];
        for (layer_id, layer) in layers.iter().enumerate() {
            assert_eq!(
                layer["crossings"].as_array().unwrap().len(),
                2 * islands * wall_count(settings, layer_id),
                "{label}/{mode} layer {layer_id}: {layer}"
            );
        }
    }
    if matches!(label, "prism" | "zero-walls" | "two-islands") {
        assert!(
            off.iter()
                .any(|layer| layer["infill_segments"].as_u64().unwrap() > 0),
            "positive sparse infill must actually emit: {label}"
        );
    }
}
