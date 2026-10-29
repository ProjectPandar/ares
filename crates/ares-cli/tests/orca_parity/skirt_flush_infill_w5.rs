//! GCode.cpp:5330-5333 wiping-pass extrusions x Print.cpp:2784-2996
//! per-object skirt: the SEMM fixture (R1 base, skirt_type=perobject,
//! prime tower on, flush_into_infill=1) — Native extrudes infill
//! during the wiping pass and groups per-object skirts with the tower
//! obstacle (GCode.cpp:4465-4485); the leg keeps the combination
//! byte-identical and asserts the flush is ACTIVE (infill inside the
//! wiping windows). flush_into_objects and flush_into_support stay
//! guarded (tool-change/print ordering and support-skirt placement
//! residuals — their own slices).
use std::fs;

use serde_json::json;
use sha2::{Digest, Sha256};

use crate::runner::{OrcaRunner, stages};

const INPUT: &[u8] =
    include_bytes!("../../../../tests/parity/validation/w5-skirt/skirt-flush-infill-w5.3mf");
const INPUT_SHA256: &str = "b029a0f96daf20f2e4a5f677a02af8f82e606da5affd584608631304951ccefc";

#[test]
fn skirt_flush_infill_w5_matches_native() {
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
        .prefix("skirt-flush-infill-w5-")
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
    // The flush option must be ACTIVE, not merely configured: infill
    // must extrude during the wiping pass — in a window between
    // `; PURGING FINISHED` and the next `;LAYER_CHANGE`.
    let mut in_wiping_window = false;
    let mut wiping_infill_windows = 0usize;
    for line in reference.lines() {
        if line == "; PURGING FINISHED" {
            in_wiping_window = true;
        } else if line == ";LAYER_CHANGE" {
            in_wiping_window = false;
        } else if in_wiping_window && line == ";TYPE:Sparse infill" {
            wiping_infill_windows += 1;
        }
    }
    assert!(
        wiping_infill_windows >= 10,
        "Native flushing into infill active: {wiping_infill_windows} wiping windows with infill"
    );
    assert!(
        reference.contains(";TYPE:Skirt\n"),
        "Native per-object skirt emission"
    );
    let outcome = crate::artifacts::compare(
        &root,
        "skirt-flush-infill-w5",
        Ok(INPUT),
        Ok(reference.as_bytes()),
        json!({
            "kind":"skirt_per_object_tower_native",
            "input_sha256":hash,
            "native_command_directory":directory,
            "source":"Print.cpp:2784-2996,2821-2823,2865-2899;GCode.cpp:4465-4485,5330-5333",
        }),
    )
    .unwrap();
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}
