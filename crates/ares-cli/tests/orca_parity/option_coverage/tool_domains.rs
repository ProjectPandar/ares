//! PrintConfig.cpp:2489; ToolOrdering.cpp:1275-1312: bounded Manual maps.
use super::{OptionCase, Value};

use serde_json::json;

pub(super) fn generate(key: &str) -> Option<(Vec<OptionCase>, Value)> {
    (key == "filament_map").then(|| {
        let cases = [["1", "1"], ["1", "2"], ["2", "1"], ["2", "2"]]
            .into_iter()
            .map(|map| OptionCase {
                label: format!("manual-{}-{}", map[0], map[1]),
                value: Some(Value::from(map.to_vec())),
            })
            .collect();
        (
            cases,
            json!({
                "domain":"G4 bounded Manual filament maps",
                "definition":"PrintConfig.cpp:2489",
                "consumer":"ToolOrdering.cpp:1275-1312",
                "input":"tool_context::mapping",
                "mode":"Manual",
                "nozzle_diameter":["0.4","0.4"],
                "nozzle_volume_type":["Standard","Standard"],
                "wall_filament":1,"sparse_infill_filament":2,
                "deferred":["automatic grouping","dynamic mapping","all vector shapes"]
            }),
        )
    })
}
