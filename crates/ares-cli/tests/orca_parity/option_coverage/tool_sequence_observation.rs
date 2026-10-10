//! Observe rendered logical identity only when real role XY extrusion occurs.
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

// Failure inventory: config literal markers, retractions, startup moves and
// stale role tags must not count as filament usage; active identity spans layers.
pub(super) fn observe(gcode: &str) -> Value {
    let mut config = false;
    let mut layer = 0;
    let mut active = None;
    let mut role = "";
    let mut order = BTreeMap::<usize, Vec<usize>>::new();
    let mut roles = BTreeMap::<usize, BTreeMap<usize, BTreeSet<&str>>>::new();
    for line in gcode.lines() {
        match line {
            "; CONFIG_BLOCK_START" => {
                config = true;
                continue;
            }
            "; CONFIG_BLOCK_END" => {
                config = false;
                continue;
            }
            _ if config => continue,
            ";LAYER_CHANGE" | "; CHANGE_LAYER" => {
                layer += 1;
                // GCode.cpp:6788-6791 emits TYPE only when the role changes;
                // the last role therefore remains active across layer boundaries.
            }
            _ => {}
        }
        if let Some(id) = line.strip_prefix("; G4_SEQUENCE F") {
            active = Some(id.parse::<usize>().unwrap());
            role = "";
        }
        if let Some(value) = line
            .strip_prefix("; FEATURE: ")
            .or_else(|| line.strip_prefix(";TYPE:"))
        {
            role = value;
        }
        if layer == 0
            || !line.starts_with("G1 ")
            || !matches!(
                role,
                "Outer wall"
                    | "Inner wall"
                    | "Sparse infill"
                    | "Internal solid infill"
                    | "Top surface"
                    | "Bottom surface"
            )
        {
            continue;
        }
        let xy = line
            .split_ascii_whitespace()
            .any(|field| field.starts_with('X') || field.starts_with('Y'));
        let extruding = line
            .split_ascii_whitespace()
            .filter_map(|field| field.strip_prefix('E'))
            .any(|value| value.parse::<f64>().unwrap() > 0.0);
        if xy && extruding {
            let tool =
                active.expect("positive role extrusion without rendered logical tool identity");
            let tools = order.entry(layer).or_default();
            if tools.last() != Some(&tool) {
                tools.push(tool);
            }
            roles
                .entry(layer)
                .or_default()
                .entry(tool)
                .or_default()
                .insert(role);
        }
    }
    json!({"ordered_extrusion_runs":order,"roles":roles})
}

pub(super) fn verify(observation: &Value, label: &str, expected: &[(usize, [usize; 2])]) {
    let runs = observation["ordered_extrusion_runs"].as_object().unwrap();
    for layer in 1..=7 {
        let order = runs[&layer.to_string()].as_array().unwrap();
        assert_eq!(
            order.len(),
            2,
            "{label}/layer{layer}: both real extrusion groups required"
        );
        assert_eq!(
            order
                .iter()
                .map(|id| id.as_u64().unwrap())
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([0, 1])
        );
        let roles = &observation["roles"][layer.to_string()];
        assert!(
            roles["0"]
                .as_array()
                .unwrap()
                .iter()
                .any(|role| matches!(role.as_str(), Some("Outer wall" | "Inner wall")))
        );
        assert!(roles["1"].as_array().unwrap().iter().any(|role| matches!(
            role.as_str(),
            Some("Sparse infill" | "Internal solid infill" | "Top surface" | "Bottom surface")
        )));
    }
    assert_eq!(
        observation["ordered_extrusion_runs"]["1"],
        json!([0, 1]),
        "{label}: settled first-layer order"
    );
    for (layer, order) in expected {
        assert_eq!(
            observation["ordered_extrusion_runs"][layer.to_string()],
            json!(order),
            "{label}/layer{layer}: inclusive/last-row priority"
        );
    }
}
