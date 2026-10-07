use std::path::Path;

use serde_json::{Value, json};

use super::bounds::{definition_block, numeric_bound, range_type};

pub(in super::super) fn inventory(repo: &Path) -> Vec<Value> {
    let source =
        std::fs::read_to_string(repo.join("OrcaSlicer/src/libslic3r/PrintConfig.cpp")).unwrap();
    let lines = source.lines().collect::<Vec<_>>();
    let options: Value = serde_json::from_str(
        &std::fs::read_to_string(repo.join("tests/ksr_fdmtest_v4/options-v242.json")).unwrap(),
    )
    .unwrap();
    options
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|entry| {
            let kind = entry["option_type"].as_str().unwrap();
            if !range_type(kind) {
                return None;
            }
            let key = entry["key"].as_str().unwrap();
            let definition = &entry["upstream_definition"];
            let line = definition["line"].as_u64().unwrap() as usize;
            let block = definition_block(&lines, key, line);
            let resolved_line = lines
                .iter()
                .position(|candidate| *candidate == block.lines().next().unwrap())
                .unwrap()
                + 1;
            let side = |name| match numeric_bound(&block, name) {
                None => json!({"kind":"UNBOUNDED"}),
                Some(Ok(value)) => json!({"kind":"DECLARED", "value":value}),
                Some(Err(())) => json!({"kind":"UNRESOLVED"}),
            };
            Some(json!({"key":key, "type":kind,
            "source":format!("src/libslic3r/PrintConfig.cpp:{resolved_line}"),
                "inventory_source":format!("{}:{line}", definition["path"].as_str().unwrap()),
            "minimum":side("min"), "maximum":side("max")}))
        })
        .collect()
}
