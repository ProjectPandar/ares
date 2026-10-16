//! Print.cpp:2784-2996 per-object skirt grouping x wipe tower: the SEMM
//! two-object/two-filament fixture (extruders 1/2, prime tower on) with
//! skirt_type=perobject. Native groups the objects with the tower
//! obstacle (:2821-2823, :2865-2899) and emits per-group skirts
//! (GCode.cpp:4465-4485); Ares rejects `skirt_type per-object`
//! (gcode_emit/skirt/generate.rs guard) — the RED must FAIL before
//! the GREEN slice removes the tower/multi-filament/prime-tower
//! disjuncts.
use std::fs;

use serde_json::json;
use sha2::{Digest, Sha256};

use crate::runner::{OrcaRunner, stages};

const INPUT: &[u8] =
    include_bytes!("../../../../tests/parity/validation/w5-skirt/per-object-tower-w5.3mf");
const INPUT_SHA256: &str = "d9ee50b057550861a5bcf7ef9fa62a90e08c54cb3c197ebf5255b6c4122e145b";

#[test]
fn skirt_per_object_tower_w5_matches_native() {
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
        .prefix("skirt-per-object-tower-w5-")
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
    let reference = fs::read(output.join("plate_1.gcode")).unwrap();
    assert!(
        std::str::from_utf8(&reference)
            .unwrap()
            .contains(";TYPE:Skirt\n"),
        "Native skirt activation"
    );
    let outcome = crate::artifacts::compare(
        &root,
        "skirt-per-object-tower-w5",
        Ok(INPUT),
        Ok(&reference),
        json!({
            "kind":"skirt_per_object_tower_native",
            "input_sha256":hash,
            "native_command_directory":directory,
            "source":"Print.cpp:2784-2996,2821-2823,2865-2899;GCode.cpp:4465-4485",
        }),
    )
    .unwrap();
    assert_eq!(
        outcome.status, "PASS",
        "{}; {:?}",
        outcome.detail, outcome.artifacts
    );
}
