//! PrintObjectSlice.cpp1207–1243 / MultiMaterialSegmentation.cpp2169–2207 at8500fcdcc.
//! Success-required public RED; failure inventory: painted-beam-call-order-spec.md.

use crate::{
    paint_color_activity as activity,
    runner::{
        native_api_cli::{NativeCase, sha},
        native_stream_case,
    },
};
use serde_json::{Value, json};
use std::fs;

const UPSTREAM: &[&str] = &[
    "PrintObjectSlice.cpp:874-1030,1207-1254",
    "MultiMaterialSegmentation.cpp:1955-2207",
    "Feature/Interlocking/InterlockingGenerator.cpp:25-65",
    "GUI/Tab.cpp:2691-2697;GUI/ConfigManipulation.cpp:987-991;PrintConfig.cpp:4226-4277",
];
const CASES: [(&str, &str, &str); 2] = [
    (
        "painted-split-beam-off",
        "011b786ebc55a3208c554e4f87c50c35d2473a8cb737485f1b35001f728eceb2",
        "0",
    ),
    (
        "painted-split-beam-on",
        "703e1d4cf4ade1e7aa93beb0a5ae7950e9633ad76fa0a610f06548ee36963bf8",
        "1",
    ),
];

fn input(label: &str) -> Vec<u8> {
    fs::read(
        crate::runner::repo_root()
            .join("tests/parity/painted-beam")
            .join(format!("{label}.3mf")),
    )
    .unwrap()
}

fn assert_native(reference: &NativeCase, replay: &NativeCase, beam: &str) -> Value {
    let mut observed = Vec::new();
    for native in [reference, replay] {
        let model = activity::activity(native.native.as_bytes());
        for tool in ["0", "1"] {
            assert!(model["positive_model_XYE_by_tool"][tool].as_u64().unwrap() > 0);
        }
        for (key, value) in [
            ("interlocking_beam", beam),
            ("interlocking_beam_width", "0.8"),
            ("interlocking_orientation", "22.5"),
            ("interlocking_beam_layer_count", "2"),
            ("interlocking_depth", "2"),
            ("interlocking_boundary_avoidance", "0"),
            ("mmu_segmented_region_max_width", "0"),
            ("mmu_segmented_region_interlocking_depth", "0"),
            ("enable_prime_tower", "0"),
        ] {
            assert!(native.native.contains(&format!("; {key} = {value}\n")));
        }
        fs::write(
            native.root.join("native-activity.json"),
            serde_json::to_vec_pretty(&model).unwrap(),
        )
        .unwrap();
        observed.push(model);
    }
    assert_eq!(observed[0], observed[1], "Native model activity replay");
    observed.remove(0)
}

#[test]
fn painted_beam_native_preflight() {
    let mut qualified = Vec::new();
    let mut activities = Vec::new();
    for (label, digest, beam) in CASES {
        let project = input(label);
        let (reference, replay) = native_stream_case::qualify(
            ("painted-beam-preflight", UPSTREAM),
            label,
            &project,
            digest,
            true,
        );
        activities.push(assert_native(&reference, &replay, beam));
        qualified.push(json!({"case":label,"input_sha256":sha(&project),
            "reference":reference.root,"replay":replay.root}));
    }
    assert_ne!(
        activities[0], activities[1],
        "beam must change model activity"
    );
    fs::write(
        crate::artifacts::root_from_env()
            .unwrap()
            .join("painted-beam-preflight.json"),
        serde_json::to_vec_pretty(&qualified).unwrap(),
    )
    .unwrap();
}

fn check(index: usize) {
    let (label, digest, beam) = CASES[index];
    native_stream_case::check(
        ("painted-beam", UPSTREAM),
        label,
        &input(label),
        digest,
        (true, |reference, replay| {
            assert_native(reference, replay, beam);
        }),
    );
}

#[test]
fn painted_beam_off_control() {
    check(0);
}

#[test]
fn painted_beam_on() {
    check(1);
}
