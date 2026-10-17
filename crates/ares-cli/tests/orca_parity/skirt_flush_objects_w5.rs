//! ToolOrdering.cpp:660-760 collect_extruders x WipingExtrusions
//! is_overriddable_and_mark (:686-698, :718-733): the SEMM fixture
//! (R1 base, prime tower, 2 filaments, by-layer, skirt_type=perobject,
//! flush_into_objects=1) — every wall/fill of the tool-2 object is
//! overriddable, so the region adds no extruder to LayerTools and
//! Native collapses the print to a single tool (`; filament: 1`, the
//! second extruder's used-filament entry empty, no toolchange, no
//! tower) while still emitting both objects and their per-object
//! skirts; Ares rejects the input via the flush_into_objects guard
//! disjunct — the RED must FAIL on the strict compare before the
//! GREEN gates collect.rs's extruder pushes on the overridable
//! marking and retires that last guard term.
use std::fs;

use serde_json::json;
use sha2::{Digest, Sha256};

use crate::runner::{OrcaRunner, stages};

const INPUT: &[u8] =
    include_bytes!("../../../../tests/parity/validation/w5-skirt/skirt-flush-objects-w5.3mf");
const INPUT_SHA256: &str = "7fc25540dc506a126c618d14088e3b4dbd34b7d0b94851ec908ae0c441da3ce2";

#[test]
fn skirt_flush_objects_w5_matches_native() {
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
        .prefix("skirt-flush-objects-w5-")
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
    // The flush option must be ACTIVE, not merely configured: the
    // tool-2 object must be fully overridden — the header lists one
    // filament and the second extruder's used-filament entry is
    // empty (collect_extruders added no extruder for it).
    let filament_header = reference
        .lines()
        .find(|line| line.starts_with("; filament: "))
        .unwrap();
    assert_eq!(
        filament_header, "; filament: 1",
        "Native single-tool collapse"
    );
    let used = reference
        .lines()
        .find(|line| line.starts_with("; filament used [mm] = "))
        .unwrap();
    let second = used
        .rsplit(',')
        .next()
        .unwrap()
        .trim()
        .parse::<f64>()
        .unwrap();
    assert_eq!(second, 0.0, "Native second extruder unused: {used}");
    assert!(
        reference.contains(";TYPE:Skirt\n"),
        "Native per-object skirt emission"
    );
    let outcome = crate::artifacts::compare(
        &root,
        "skirt-flush-objects-w5",
        Ok(INPUT),
        Ok(reference.as_bytes()),
        json!({
            "kind":"skirt_per_object_tower_native",
            "input_sha256":hash,
            "native_command_directory":directory,
            "source":"ToolOrdering.cpp:660-760,686-698,718-733;GCode.cpp:5336-5338",
        }),
    )
    .unwrap();
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}
