//! PrintConfig.cpp:2489; ToolOrdering.cpp:1275-1312: bounded Manual maps.
use super::{OptionCase, Value};

pub(super) fn generate(key: &str) -> Option<Vec<OptionCase>> {
    (key == "filament_map").then(|| {
        [["1", "1"], ["1", "2"], ["2", "1"], ["2", "2"]]
            .into_iter()
            .map(|map| OptionCase {
                label: format!("manual-{}-{}", map[0], map[1]),
                value: Some(Value::from(map.to_vec())),
            })
            .collect()
    })
}
