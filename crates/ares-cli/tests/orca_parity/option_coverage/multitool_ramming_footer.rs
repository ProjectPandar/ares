//! `PrintConfig.cpp:10211–10213,11862–11994`: public three-key gather witness.

use crate::{ParityOutcome, artifacts};
use serde_json::json;

const PREFIXES: [&str; 3] = [
    "; filament_multitool_ramming = ",
    "; filament_multitool_ramming_flow = ",
    "; filament_multitool_ramming_volume = ",
];
const EXPECTED: [&str; 3] = [
    "; filament_multitool_ramming = 0,0",
    "; filament_multitool_ramming_flow = 10,10",
    "; filament_multitool_ramming_volume = 10,10",
];

fn footer(output: &str) -> Vec<&str> {
    output
        .lines()
        .filter(|line| PREFIXES.iter().any(|prefix| line.starts_with(prefix)))
        .collect()
}

pub(crate) fn check(outcome: &ParityOutcome) {
    let directory = outcome.artifacts.as_ref().unwrap();
    let native = std::fs::read_to_string(directory.join("orca.gcode")).unwrap();
    let api = std::fs::read_to_string(directory.join("ares.gcode")).unwrap();
    let native_lines = footer(&native);
    let api_lines = footer(&api);
    artifacts::write(
        &directory.join("multitool-ramming-footer.json"),
        &serde_json::to_vec_pretty(&json!({
            "label": outcome.label,
            "expected_native": EXPECTED,
            "native": native_lines,
            "api": api_lines,
            "native_premise": native_lines == EXPECTED,
            "api_matches_native": api_lines == native_lines,
            "whole_stream_status": outcome.status,
            "whole_stream_detail": outcome.detail,
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(native_lines, EXPECTED, "Native three-key ramming premise");
    assert_eq!(
        api_lines, native_lines,
        "multitool ramming three-key footer mismatch; {directory:?}"
    );
}
