//! Admitted ParameterUtils.cpp:6-29 / ToolOrdering.cpp:1320-1362 count metadata.
use super::{OptionCase, Value};
use serde_json::json;

pub(super) fn generate(key: &str) -> Option<(Vec<OptionCase>, Value)> {
    (key == "other_layers_print_sequence_nums").then(|| {
        let cases = (0..=2).map(|count| OptionCase {
            label: format!("count{count}"),
            value: Some(json!(count.to_string())),
        }).collect();
        (cases, json!({
            "domain":"G4 bounded other-layer sequence count",
            "definition":"PrintConfig.cpp:1115-1123",
            "consumer":"ParameterUtils.cpp:6-29; ToolOrdering.cpp:1320-1362",
            "input":"tool_sequences::context::prepare",
            "context":"Ender-3 0.4; one Standard nozzle; two PLA; Manual map [1,1]; by layer",
            "first_layer_print_sequence":[0,1],
            "paired_vectors":[[2,5,2,1],[2,5,2,1],[2,5,1,2,4,6,2,1]],
            "canonical_count":3,"controls":["empty-vector","reverse-row-priority"],
            "spec":"docs/architecture/option-g4-other-layer-sequence-count.md",
            "deferred":["unbounded counts","malformed vectors","by-object","other G4 consumers"]
        }))
    })
}
