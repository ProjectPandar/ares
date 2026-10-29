use serde_json::{Value, json};
use std::collections::BTreeMap;

pub(super) fn activity(bytes: &[u8]) -> Value {
    let mut tool = None;
    let mut relative_e = true;
    let mut e_position = BTreeMap::<Option<usize>, f64>::new();
    let mut layers = Vec::<BTreeMap<usize, usize>>::new();
    let mut counts = BTreeMap::<usize, usize>::new();
    let mut feature = "";
    for line in std::str::from_utf8(bytes).unwrap().lines() {
        if line == ";LAYER_CHANGE" {
            layers.push(BTreeMap::new());
        }
        if let Some(value) = line.strip_prefix(";TYPE:") {
            feature = value;
        }
        if let Some(value) = line.strip_prefix('T')
            && let Ok(value) = value.parse::<usize>()
        {
            tool = Some(value);
        }
        let mut words = line.split(';').next().unwrap().split_whitespace();
        let command = words.next();
        match command {
            Some("M82") => relative_e = false,
            Some("M83") => relative_e = true,
            _ => {}
        }
        if !matches!(command, Some("G0" | "G1" | "G92")) {
            continue;
        }
        let fields: Vec<_> = words.collect();
        let e = fields
            .iter()
            .find_map(|word| word.strip_prefix('E'))
            .map(|value| value.parse::<f64>().unwrap());
        let Some(e) = e else { continue };
        if command == Some("G92") {
            e_position.insert(tool, e);
            continue;
        }
        if !matches!(command, Some("G0" | "G1")) {
            continue;
        }
        let previous = e_position.entry(tool).or_default();
        let delta = if relative_e { e } else { e - *previous };
        *previous = if relative_e { *previous + e } else { e };
        if delta > 0.0
            && !layers.is_empty()
            && fields.iter().any(|word| word.starts_with(['X', 'Y']))
            && matches!(
                feature,
                "Outer wall"
                    | "Inner wall"
                    | "Sparse infill"
                    | "Internal solid infill"
                    | "Top surface"
                    | "Bottom surface"
                    | "Gap infill"
            )
        {
            let tool = tool.unwrap();
            *counts.entry(tool).or_default() += 1;
            *layers.last_mut().unwrap().entry(tool).or_default() += 1;
        }
    }
    json!({"positive_model_XYE_by_tool":counts,"positive_model_XYE_by_layer":layers})
}
