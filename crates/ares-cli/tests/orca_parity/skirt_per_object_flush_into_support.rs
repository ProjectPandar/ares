//! GCode.cpp:5318-5360 process_layer print_wipe_extrusions two-loop
//! order x Print.cpp:2865-2899 support footprints: the SEMM fixture
//! (overhang object, tree support, prime tower, 2 filaments,
//! skirt_type=perobject, flush_into_support=1) — Native extrudes
//! Support inside the wiping pass and emits per-object skirts in the
//! first print_wipe_extrusions loop (generate_object_skirt_group,
//! GCode.cpp:5336-5338) ahead of the printing/stop labels, with
//! "; PURGING FINISHED" opening the normal loop (:5327-5328); the leg
//! keeps the whole layer sequence byte-identical and asserts the
//! flush is ACTIVE (support inside the wiping windows).
//! flush_into_objects stays guarded (tool-change/print ordering).
use std::fs;

use serde_json::json;
use sha2::{Digest, Sha256};

use crate::runner::{OrcaRunner, stages};

const INPUT: &[u8] = include_bytes!(
    "../../../../tests/parity/validation/skirt_wipe_tower_flush/skirt-flush-support.3mf"
);
const INPUT_SHA256: &str = "2ac30225e66793852adc01d9abd9dfe6e0c068e09311b8632f4c050b88dca85c";

#[test]
fn skirt_per_object_flush_into_support_matches_native() {
    let hash = Sha256::digest(INPUT)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(hash, INPUT_SHA256);

    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let directory = tempfile::Builder::new()
        .prefix("skirt-flush-support-")
        .tempdir_in(&root)
        .unwrap()
        .keep();
    let path = directory.join("input.3mf");
    let output = directory.join("orca");
    fs::write(&path, INPUT).unwrap();
    fs::create_dir(&output).unwrap();
    let args = [
        "--slice".to_owned(),
        "0".to_owned(),
        "--outputdir".to_owned(),
        output.to_str().unwrap().to_owned(),
        path.to_str().unwrap().to_owned(),
    ];
    stages::run(
        runner.bin(),
        args.iter().cloned(),
        stages::Stage::Slice,
        &directory,
    )
    .unwrap();
    let reference = fs::read_to_string(output.join("plate_1.gcode")).unwrap();
    // The flush option must be ACTIVE, not merely configured: support
    // must extrude during the wiping pass — in a window between
    // `; PURGING FINISHED` and the next `;LAYER_CHANGE`.
    let mut in_wiping_window = false;
    let mut wiping_support_windows = 0usize;
    for line in reference.lines() {
        if line == "; PURGING FINISHED" {
            in_wiping_window = true;
        } else if line == ";LAYER_CHANGE" {
            in_wiping_window = false;
        } else if in_wiping_window && (line == ";TYPE:Support" || line == ";TYPE:Support interface")
        {
            wiping_support_windows += 1;
        }
    }
    assert!(
        wiping_support_windows >= 10,
        "Native flushing into support active: {wiping_support_windows} wiping windows with support"
    );
    assert!(
        reference.contains(";TYPE:Skirt\n"),
        "Native per-object skirt emission"
    );
    let outcome = crate::artifacts::compare(
        &root,
        "skirt-flush-support",
        Ok(INPUT),
        Ok(reference.as_bytes()),
        json!({
            "kind":"skirt_per_object_tower_native",
            "input_sha256":hash,
            "native_command_directory":directory,
            "source":"GCode.cpp:5318-5360,5327-5328,5336-5338,5349,5471-5475,4465-4485;Print.cpp:2865-2899",
        }),
    )
    .unwrap();
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}
