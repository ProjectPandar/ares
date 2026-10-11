use serde_json::{Value, json};
use std::collections::BTreeSet;

// Admission requires persistent logical tool state, real positive XY wall
// extrusion, both slots and a handoff distinct from the upcoming tool.
pub(super) fn observe(gcode: &str) -> Value {
    let mut active = None;
    let mut role = "";
    let mut z = None;
    let mut config = false;
    let mut physical_layer = 0;
    let mut hooks = Vec::<Value>::new();
    let mut walls = BTreeSet::new();
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
            physical_layer += 1;
        }
        if let Some(id) = line.strip_prefix("; G4_SEQUENCE F") {
            active = Some(id.parse::<usize>().unwrap());
            role = "";
        }
        if let Some(marker) = line.strip_prefix("; LCTX ") {
            let fields = marker.split_ascii_whitespace().collect::<Vec<_>>();
            hooks.push(json!({"hook":fields[0], "physical_layer":physical_layer, "layer":fields[1].parse::<usize>().unwrap(),
                "scalar":fields[2].parse::<usize>().unwrap(), "layer_z":fields[3].parse::<f64>().unwrap(),
                "writer":active.unwrap(), "writer_z":z, "temperature":null, "explicit":null, "next_wall_tool":null}));
        }
        if line.starts_with("M104 S")
            && let Some(hook) = hooks.last_mut()
            && hook["temperature"].is_null()
        {
            hook["temperature"] = json!(
                line.split_ascii_whitespace().nth(1).unwrap()[1..]
                    .parse::<usize>()
                    .unwrap()
            );
        }
        if let Some(values) = line.strip_prefix("; LCTX_INDEX ") {
            hooks.last_mut().unwrap()["explicit"] = json!(
                values
                    .split_ascii_whitespace()
                    .map(|value| value.parse::<usize>().unwrap())
                    .collect::<Vec<_>>()
            );
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
            for hook in hooks
                .iter_mut()
                .rev()
                .take_while(|hook| hook["next_wall_tool"].is_null())
            {
                hook["next_wall_tool"] = json!(tool);
            }
        }
    }
    json!({"hooks":hooks,"wall_tools":walls})
}

pub(super) fn verify(observed: &Value, enabled: bool, paired: bool, handoff: bool, explicit: bool) {
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
    assert!(!hooks.is_empty(), "layer hook not consumed");
    let mut writers = BTreeSet::new();
    for hook in hooks {
        let writer = hook["writer"].as_u64().unwrap();
        writers.insert(writer);
        assert_eq!(hook["scalar"], json!(writer), "writer/scalar identity");
        assert_eq!(
            hook["temperature"],
            json!(if explicit || writer == 0 { 211 } else { 223 }),
            "M104 filament context: {hook}"
        );
        assert_eq!(
            hook["explicit"],
            json!([211, 223]),
            "explicit indices: {hook}"
        );
        assert!(
            !hook["next_wall_tool"].is_null(),
            "no following wall: {hook}"
        );
    }
    if paired {
        assert_eq!(writers, BTreeSet::from([0, 1]));
    }
    if handoff {
        assert!(
            hooks
                .iter()
                .any(|hook| hook["writer"] != hook["next_wall_tool"]),
            "no admitted current/upcoming distinction"
        );
    }
}
