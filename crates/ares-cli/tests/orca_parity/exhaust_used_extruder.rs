//! GCode.cpp3181–3196/3501–3516 at8500fcdcc: writer-used exhaust fan reduction.
//! Success-required public RED; failure inventory: exhaust-used-extruder-spec.md.

use crate::{
    paint_color_activity as activity,
    runner::{
        native_api_cli::{NativeCase, sha},
        native_stream_case,
    },
};
use serde_json::json;
use std::fs;

const UPSTREAM: &[&str] = &[
    "GCode.cpp:3181-3196,3501-3516,2760,2791,5646-5648",
    "GCodeWriter.cpp:63-80,1157-1171;Print.cpp:522-547",
    "PrintConfig.cpp:1886-1921,3899-3904;Tab.cpp:4108-4123,4304-4312,4577",
];

struct Witness {
    label: &'static str,
    sha: &'static str,
}

const CASES: [Witness; 4] = [
    Witness {
        label: "unused-high",
        sha: "335014559fc941ce8519926d2ae50c00ef9b8c3df5f67fa2b5359be3b6b9d297",
    },
    Witness {
        label: "equal",
        sha: "94c7e72870e46fbb42b481daf45497780434e02dfb929fec0aa0503dcf8635e6",
    },
    Witness {
        label: "unused-disabled",
        sha: "4663aa8d224711c8c025db0de5851ca6cedfc7faf3e2fb5d7f3d744850c5ce3a",
    },
    Witness {
        label: "capability-off",
        sha: "4da717b2ae507a446782e5265a4803a9aae687e096fde757cedde6adf7f9c1f5",
    },
];

#[test]
fn orca_parity_exhaust_unused_high_speed() {
    check(&CASES[0]);
}

#[test]
fn orca_parity_exhaust_equal_speed() {
    check(&CASES[1]);
}

#[test]
fn orca_parity_exhaust_unused_disabled() {
    check(&CASES[2]);
}

#[test]
fn orca_parity_exhaust_capability_off() {
    check(&CASES[3]);
}

#[test]
fn orca_parity_exhaust_native_preflight() {
    let mut qualified = Vec::new();
    for case in &CASES {
        let project = input(case);
        let (reference, replay) = native_stream_case::qualify(
            ("exhaust-used-extruder-preflight", UPSTREAM),
            case.label,
            &project,
            case.sha,
            false,
        );
        assert_native(&reference, &replay);
        qualified.push(json!({"case":case.label,"input_sha256":sha(&project),
            "reference":reference.root,"replay":replay.root}));
    }
    fs::write(
        crate::artifacts::root_from_env()
            .unwrap()
            .join("exhaust-used-extruder-preflight.json"),
        serde_json::to_vec_pretty(&qualified).unwrap(),
    )
    .unwrap();
}

fn check(case: &Witness) {
    native_stream_case::check(
        ("exhaust-used-extruder", UPSTREAM),
        case.label,
        &input(case),
        case.sha,
        (false, assert_native),
    );
}

fn input(case: &Witness) -> Vec<u8> {
    fs::read(
        crate::runner::repo_root()
            .join("tests/parity/exhaust-used-extruder")
            .join(format!("{}.3mf", case.label)),
    )
    .unwrap()
}

fn assert_native(reference: &NativeCase, replay: &NativeCase) {
    let mut activities = Vec::new();
    for native in [reference, replay] {
        let observed = activity::activity(native.native.as_bytes());
        let model = observed["positive_model_XYE_by_tool"].as_object().unwrap();
        assert!(model["0"].as_u64().unwrap() > 0);
        assert!(model.keys().all(|tool| tool == "0"));
        fs::write(
            native.root.join("native-activity.json"),
            serde_json::to_vec_pretty(&observed).unwrap(),
        )
        .unwrap();
        activities.push(observed);
    }
    assert_eq!(
        activities[0], activities[1],
        "Native T0 model activity replay"
    );
}
