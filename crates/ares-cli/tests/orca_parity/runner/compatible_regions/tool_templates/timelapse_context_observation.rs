use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

// Observe executable blocks, persistent tool identity and positive XY/E walls;
// phase is relative to actual walls, not inferred from the marker's scalar.
pub(super) fn observe(gcode: &str) -> Value {
    let mut active = None;
    let mut role = "";
    let mut z = None;
    let mut config = false;
    let mut layer = 0;
    let mut wall_count = 0;
    let mut hooks = Vec::<Value>::new();
    let mut walls = BTreeSet::new();
    let mut in_hook = false;
    for line in gcode.lines() {
        match line {
            "; CONFIG_BLOCK_START" => config = true,
            "; CONFIG_BLOCK_END" => config = false,
            _ => {}
        }
        if config {
            continue;
        }
        if matches!(line, ";LAYER_CHANGE" | "; CHANGE_LAYER") {
            layer += 1;
            wall_count = 0;
        }
        if let Some(id) = line.strip_prefix("; G4_SEQUENCE F") {
            active = Some(id.parse::<usize>().unwrap());
            role = "";
        }
        if let Some(marker) = line.strip_prefix("; TLCTX ") {
            let fields = marker.split_ascii_whitespace().collect::<Vec<_>>();
            hooks.push(json!({"physical_layer":layer, "layer":fields[0].parse::<usize>().unwrap(),
                "scalar":fields[1].parse::<usize>().unwrap(), "layer_z":fields[2].parse::<f64>().unwrap(),
                "writer":active.unwrap(), "writer_z":z, "temperatures":[], "next_wall_tool":null,
                "phase":if wall_count == 0 { "direct" } else { "traditional" }, "preceding_wall_moves":wall_count}));
            in_hook = true;
        }
        if in_hook && let Some(value) = line.strip_prefix("M104 S") {
            hooks.last_mut().unwrap()["temperatures"]
                .as_array_mut()
                .unwrap()
                .push(json!(value.parse::<usize>().unwrap()));
        }
        if line == "; TLCTX_END" {
            in_hook = false;
        }
        if let Some(value) = line
            .strip_prefix("; FEATURE: ")
            .or_else(|| line.strip_prefix(";TYPE:"))
        {
            role = value;
        }
        if !line.starts_with("G1 ") {
            continue;
        }
        let fields = line.split_ascii_whitespace().collect::<Vec<_>>();
        if let Some(value) = fields.iter().find_map(|field| field.strip_prefix('Z')) {
            z = Some(value.parse::<f64>().unwrap());
        }
        let xy = fields
            .iter()
            .any(|field| field.starts_with('X') || field.starts_with('Y'));
        let positive = fields
            .iter()
            .filter_map(|field| field.strip_prefix('E'))
            .any(|value| value.parse::<f64>().unwrap() > 0.0);
        if role == "Outer wall" && xy && positive {
            let tool = active.unwrap();
            walls.insert(tool);
            wall_count += 1;
            for hook in hooks
                .iter_mut()
                .rev()
                .take_while(|hook| hook["next_wall_tool"].is_null())
            {
                hook["next_wall_tool"] = json!(tool);
            }
        }
    }
    json!({"hooks":hooks,"wall_tools":walls,"physical_layers":layer})
}

/// The phase `need_insert_timelapse_gcode_for_traditional` selects for a nonempty template
/// (`GCode.cpp:6689-6692`): the traditional inserts when true, the layer-change render
/// (`:6710`) otherwise. `enable_timelapse_print()` is `timelapse_type` Smooth.
pub(super) fn gate_phase(native: &str) -> &'static str {
    let value = |key: &str| {
        let prefix = format!("; {key} = ");
        native
            .lines()
            .find_map(|line| line.strip_prefix(prefix.as_str()))
            .unwrap()
    };
    let smooth_tower = value("enable_prime_tower") == "1" && value("timelapse_type") == "1";
    let i3 = value("printer_structure") == "i3" && value("spiral_mode") == "0";
    if !smooth_tower && (i3 || value("nozzle_diameter").contains(',')) {
        "traditional"
    } else {
        "direct"
    }
}

pub(super) fn verify(observed: &Value, enabled: bool, handoff: bool, explicit: bool, phase: &str) {
    assert_eq!(
        observed["wall_tools"],
        json!([0, 1]),
        "actual wall consumers"
    );
    let hooks = observed["hooks"].as_array().unwrap();
    if !enabled {
        assert!(hooks.is_empty());
        return;
    }
    let mut phases = BTreeMap::<u64, BTreeSet<&str>>::new();
    let mut writers = BTreeSet::new();
    for hook in hooks {
        let writer = hook["writer"].as_u64().unwrap();
        writers.insert(writer);
        phases
            .entry(hook["physical_layer"].as_u64().unwrap())
            .or_default()
            .insert(hook["phase"].as_str().unwrap());
        assert_eq!(hook["scalar"], json!(writer), "writer/scalar identity");
        assert_eq!(
            hook["temperatures"],
            if explicit {
                json!([211, 223])
            } else {
                json!([if writer == 0 { 211 } else { 223 }])
            },
            "executable M104 context: {hook}"
        );
        assert!(
            !hook["next_wall_tool"].is_null(),
            "following actual wall: {hook}"
        );
    }
    assert_eq!(writers, BTreeSet::from([0, 1]), "both current slots");
    assert_eq!(
        phases.len() as u64,
        observed["physical_layers"].as_u64().unwrap()
    );
    // `has_insert_timelapse_gcode` and the single layer-change site: one render per layer.
    assert_eq!(hooks.len(), phases.len(), "one timelapse render per layer");
    assert!(
        phases
            .values()
            .all(|value| *value == BTreeSet::from([phase])),
        "{phase} phase on every layer: {phases:?}"
    );
    if handoff {
        assert!(
            hooks
                .iter()
                .any(|hook| hook["writer"] != hook["next_wall_tool"]),
            "current/upcoming distinction"
        );
    }
}
