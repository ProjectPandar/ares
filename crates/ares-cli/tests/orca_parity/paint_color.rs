//! Success-required public painting parity; MMS1955–2207 / PrintObjectSlice874–1030.
//! Failure inventory and Native-first protocol: painted-projection-graph-spec §7–8.

use crate::runner::{
    native_api_cli::{NativeCase, sha},
    native_stream_case,
};
use serde_json::{Value, json};
use std::{env, fs, process::Command};

use crate::paint_color_activity as activity;

const UPSTREAM: &[&str] = &[
    "MultiMaterialSegmentation.cpp:491-956,1181-1472,1527-1870,1955-2207",
    "Point.hpp:506-659",
    "PrintObjectSlice.cpp:874-1030,1207-1224",
];

#[test]
fn paint_color_unpainted() {
    check("unpainted");
}

#[test]
fn paint_color_leaves() {
    check("painted-leaves");
}

#[test]
fn paint_color_split() {
    check("painted-split");
}

#[test]
fn paint_color_native_preflight() {
    let mut qualified = Vec::new();
    for label in ["unpainted", "painted-leaves", "painted-split"] {
        let (project, case) = prepare(label);
        let (reference, replay) = native_stream_case::qualify(
            ("paint-color-preflight", UPSTREAM),
            label,
            &project,
            case["archive_sha256"].as_str().unwrap(),
            label != "unpainted",
        );
        assert_activity(&reference, &replay, &case, label);
        qualified.push(json!({"case":label,"input_sha256":sha(&project),
            "reference":reference.root,"replay":replay.root}));
    }
    fs::write(
        crate::artifacts::root_from_env()
            .unwrap()
            .join("paint-color-preflight.json"),
        serde_json::to_vec_pretty(&qualified).unwrap(),
    )
    .unwrap();
}

fn check(label: &str) {
    let (project, case) = prepare(label);
    native_stream_case::check(
        ("paint-color", UPSTREAM),
        label,
        &project,
        case["archive_sha256"].as_str().unwrap(),
        (label != "unpainted", |reference, replay| {
            assert_activity(reference, replay, &case, label)
        }),
    );
}

fn prepare(label: &str) -> (Vec<u8>, Value) {
    let fixture = crate::runner::repo_root().join("tests/parity/paint-color");
    let manifest_bytes = fs::read(fixture.join("manifest.json")).unwrap();
    assert_eq!(
        sha(&manifest_bytes),
        "6e7a567431843ccd0024ad95823b179e240740d82bcf21e067125839e278652b"
    );
    let mut manifest: Value = serde_json::from_slice(&manifest_bytes).unwrap();
    let case = manifest["cases"][label].take();
    let root = tempfile::Builder::new()
        .prefix(&format!("paint-color-input-{label}-"))
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let output = Command::new(env::var_os("ARES_PYTHON_BIN").unwrap())
        .arg(fixture.join("prepare.py"))
        .arg(root.join("input"))
        .output()
        .unwrap();
    fs::write(root.join("prepare.stdout"), &output.stdout).unwrap();
    fs::write(root.join("prepare.stderr"), &output.stderr).unwrap();
    assert!(output.status.success(), "fixture preparation: {root:?}");
    let project = fs::read(root.join("input").join(format!("{label}.3mf"))).unwrap();
    assert_eq!(sha(&project), case["archive_sha256"]);
    let native = env::var_os("ARES_ORCA_NATIVE_BIN").unwrap();
    fs::write(
        root.join("launch.json"),
        serde_json::to_vec_pretty(&json!({"case":label,"input_sha256":sha(&project),
            "manifest_sha256":sha(&manifest_bytes),"native_binary":native,
            "native_sha256":sha(&fs::read(&native).unwrap()),
            "wrapper":env::var_os("ARES_ORCA_BIN"),
            "resources":env::var_os("ARES_ORCA_RESOURCES_DIR"),
            "replay_cpu":env::var_os("ORCA_REPLAY_CPU"),
            "seeded_reference":env::var_os("ARES_PARITY_SEEDED_REFERENCE"),
            "spec":"465a6f8194bfc28413b19387c3ac01f21593b496"}))
        .unwrap(),
    )
    .unwrap();
    (project, case)
}

fn assert_activity(reference: &NativeCase, replay: &NativeCase, case: &Value, label: &str) {
    let mut activities = Vec::new();
    for native in [reference, replay] {
        let activity = activity::activity(native.native.as_bytes());
        let layers = activity["positive_model_XYE_by_layer"].as_array().unwrap();
        assert_eq!(layers.len(), 50);
        assert!(
            layers
                .iter()
                .all(|layer| !layer.as_object().unwrap().is_empty())
        );
        fs::write(
            native.root.join("native-activity.json"),
            serde_json::to_vec_pretty(&activity).unwrap(),
        )
        .unwrap();
        if label != "unpainted" {
            for tool in ["0", "1"] {
                assert!(
                    activity["positive_model_XYE_by_tool"][tool]
                        .as_u64()
                        .unwrap()
                        > 0,
                    "tool {tool} must print"
                );
            }
            let both = layers
                .iter()
                .filter(|layer| {
                    let layer = layer.as_object().unwrap();
                    layer.contains_key("0") && layer.contains_key("1")
                })
                .count();
            assert!(both > 0, "both materials must share a layer");
        }
        for (key, value) in case["settings"].as_object().unwrap() {
            assert!(
                native
                    .native
                    .contains(&format!("; {key} = {}\n", value.as_str().unwrap()))
            );
        }
        activities.push(activity);
    }
    assert_eq!(activities[0], activities[1], "Native model activity replay");
}
