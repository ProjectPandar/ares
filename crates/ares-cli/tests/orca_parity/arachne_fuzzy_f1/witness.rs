//! Public emitted XY/E/width/role witnesses; never substitutes for strict bytes.
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn capture(bytes: &[u8]) -> Value {
    let mut position = [0.0; 2];
    let mut extrusion = 0.0;
    let mut relative = false;
    let mut role = "";
    let mut layer = 0usize;
    let mut layer_started = false;
    let mut width = "";
    let mut counts = BTreeMap::<String, usize>::new();
    let mut widths = BTreeMap::<String, BTreeSet<String>>::new();
    let mut diagonal = 0;
    let mut wipe = false;
    for line in std::str::from_utf8(bytes).unwrap().lines() {
        if line == ";LAYER_CHANGE" {
            layer += usize::from(layer_started);
            layer_started = true;
        }
        if let Some(value) = line.strip_prefix(";TYPE:") {
            role = value;
        }
        if let Some(value) = line.strip_prefix(";WIDTH:") {
            width = value;
        }
        if line == ";WIPE_START" {
            wipe = true;
        }
        if line == ";WIPE_END" {
            wipe = false;
        }
        let mut words = line.split(';').next().unwrap().split_whitespace();
        let Some(command) = words.next() else {
            continue;
        };
        if command == "M83" {
            relative = true;
        }
        if command == "M82" {
            relative = false;
        }
        if !matches!(command, "G0" | "G1" | "G92") {
            continue;
        }
        let start = position;
        let mut e = None;
        for word in words {
            let (axis, text) = word.split_at(1);
            let Ok(value) = text.parse::<f64>() else {
                continue;
            };
            match axis {
                "X" => position[0] = value,
                "Y" => position[1] = value,
                "E" => e = Some(value),
                _ => {}
            }
        }
        let delta = match e {
            Some(value) if relative => value,
            Some(value) => value - extrusion,
            None => 0.0,
        };
        if let Some(value) = e {
            extrusion = value;
        }
        if command == "G1"
            && delta > 0.0
            && position != start
            && !wipe
            && matches!(role, "Outer wall" | "Inner wall")
        {
            let key = format!("layer{layer}/{role}");
            *counts.entry(key.clone()).or_default() += 1;
            widths.entry(key).or_default().insert(width.to_owned());
            if role == "Outer wall" && position[0] != start[0] && position[1] != start[1] {
                diagonal += 1;
            }
        }
    }
    json!({"layer_blocks":if layer_started {layer+1} else {0},
        "positive_wall_segments":counts,"widths_by_layer_role":widths,
        "diagonal_outer_segments":diagonal})
}

pub(super) fn verify_diagnostic(witness: &Value, case: &Value) {
    assert!(
        witness["positive_wall_segments"]
            .as_object()
            .unwrap()
            .values()
            .any(|count| count.as_u64().unwrap() > 0),
        "active stochastic input has emitted walls"
    );
    // Padding has its own strict seeded case; short walls may degenerate.
    if case["settings"]["fuzzy_skin_point_distance"] == "2" {
        return;
    }
    let mode = case["settings"]["fuzzy_skin_mode"].as_str().unwrap();
    if mode != "extrusion" {
        assert!(
            witness["diagonal_outer_segments"].as_u64().unwrap() > 0,
            "displacement/combined must change the straight prism wall motion"
        );
    }
    if mode != "displacement" {
        assert!(
            witness["widths_by_layer_role"]
                .as_object()
                .unwrap()
                .iter()
                .any(|(role, widths)| role.ends_with("Outer wall")
                    && widths.as_array().unwrap().len() > 1),
            "extrusion/combined must modulate width within an outer wall layer"
        );
    }
}
