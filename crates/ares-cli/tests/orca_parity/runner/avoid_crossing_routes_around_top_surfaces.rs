//! `AvoidCrossingPerimeters.cpp:1116–1130`: route around current Top fill surfaces.

use super::{ExportedCase, OrcaRunner, cache_identity};
use std::fs;

fn check(name: &str, input: &[u8], sha256: &str) {
    assert_eq!(cache_identity::raw_digest(input), sha256);
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let directory = tempfile::Builder::new()
        .prefix("j1-top-surface-lifecycle-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input_path = directory.join("input.3mf");
    let output_dir = directory.join("native");
    fs::write(&input_path, input).unwrap();
    fs::create_dir(&output_dir).unwrap();
    let native = runner
        .slice_case(&ExportedCase {
            label: format!("j1-top-surface-lifecycle/{name}"),
            project: input.to_vec(),
            project_path: input_path,
            output_dir,
        })
        .unwrap();
    let actual = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_project(
            input,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    fs::write(directory.join("ares.gcode"), &actual).unwrap();
    fs::write(directory.join("orca.gcode"), &native.reference).unwrap();
    let reference_travel = final_layer_travel(&native.reference);
    let actual_travel = final_layer_travel(&actual);
    fs::write(
        directory.join("travel-witness.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "input_sha256": sha256,
            "native_sha256": cache_identity::raw_digest(&native.reference),
            "ares_sha256": cache_identity::raw_digest(&actual),
            "native_travel": reference_travel,
            "ares_travel": actual_travel,
            "scope": "final-layer G0/G1 XY travels only; complete primary remains separately asserted"
        }))
        .unwrap(),
    )
    .unwrap();
    if name == "top0" {
        assert!(reference_travel.windows(3).any(|lines| {
            lines
                == [
                    "G1 X159.628 Y99.772",
                    "G1 X152.911 Y103.089",
                    "G1 X159.089 Y103.089",
                ]
        }));
    }
    assert_eq!(actual_travel, reference_travel, "{directory:?}");
}

fn final_layer_travel(bytes: &[u8]) -> Vec<&str> {
    let text = std::str::from_utf8(bytes).unwrap();
    let layer = text.split(";LAYER_CHANGE\n").nth(3).unwrap();
    layer
        .lines()
        .take_while(|line| !line.starts_with("; filament end gcode"))
        .filter(|line| {
            (line.starts_with("G0 X") || line.starts_with("G1 X"))
                && !line.split_whitespace().any(|word| word.starts_with('E'))
        })
        .collect()
}

#[test]
fn j1c_zero_top_layers_travel_has_no_stale_top_holes() {
    check(
        "top0",
        include_bytes!(
            "../../../../../tests/parity/j1-used-extruder-follows-emitted-tools/role2.3mf"
        ),
        "7d796432a448d46adc30e9ea91b4c729a6044058d3dd61bb4f5c13e64ea615f4",
    );
}

#[test]
fn j1c_nonzero_top_layers_travel_control() {
    check(
        "top1",
        include_bytes!(
            "../../../../../tests/parity/j1-used-extruder-follows-emitted-tools/top1-control.3mf"
        ),
        "83d0c1908087d796008602671bdc3a6ef43f12fdd7423b0d3c96db2d423af310",
    );
}
