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

pub(super) fn verify(label: &str, off: &[Value], on: &[Value]) {
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
    // Worked fixture expectations: two y=110 crossings per enclosing wall,
    // except two-islands where each independent contour contributes its pair.
    let (expected_off, expected_on) = match label {
        "density-zero" | "density-half" => ([4; 6], [4; 6]),
        "single-top" => ([4, 4, 4, 4, 4, 2], [4, 6, 4, 6, 4, 2]),
        // Layer 4 is not topmost and its upper footprint is unchanged;
        // regeneration's no-top-surface branch restores the full count.
        "stepped-top" => ([4, 4, 4, 4, 4, 2], [4, 6, 4, 6, 4, 2]),
        "single-first" => ([2, 4, 4, 4, 4, 4], [2, 6, 4, 6, 4, 6]),
        "zero-walls" => ([0; 6], [0, 2, 0, 2, 0, 2]),
        "two-islands" => ([8; 6], [8, 12, 8, 12, 8, 12]),
        "prism" | "density-one" => ([4; 6], [4, 6, 4, 6, 4, 6]),
        _ => panic!("unknown fixture {label}"),
    };
    for (mode, layers, expected) in [("off", off, expected_off), ("on", on, expected_on)] {
        for (layer_id, (layer, count)) in layers.iter().zip(expected).enumerate() {
            assert_eq!(
                layer["crossings"].as_array().unwrap().len(),
                count,
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
