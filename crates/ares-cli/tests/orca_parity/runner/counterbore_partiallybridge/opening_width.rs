//! `PerimeterGenerator::process_no_bridge` partial counterbore `opening_ex` width parity.

use super::super::{OrcaRunner, application, cache_identity, stages};
use serde_json::json;
use std::fs;

const ROT15_SHA256: &str = "617a41ed31f52ec6dc6731b41ea9c734794e24cfaa6ca97e588940c9b1a79f36";
const ROT37_SHA256: &str = "b2f2ae8234feeff2b9f5b46937264a700ea65544d8eb20626ec9c46f433c811b";
const ROT6P003_SHA256: &str = "1502c15f61d83d5204c9b5b12c1b126c97f7f3235487a432dc77769eaaea9910";

fn assert_rotated_slot_opening_width_matches_orca(
    fixture_name: &str,
    project: &[u8],
    expected_sha256: &str,
) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    assert_eq!(cache_identity::raw_digest(project), expected_sha256);

    let input = runner.work.join("input.3mf");
    let reference_dir = runner.work.join("reference");
    fs::write(&input, project).unwrap();
    fs::create_dir_all(&reference_dir).unwrap();
    stages::run_live(
        runner.bin(),
        [
            "--slice".to_owned(),
            "1".to_owned(),
            "--outputdir".to_owned(),
            reference_dir.to_str().unwrap().to_owned(),
            input.to_str().unwrap().to_owned(),
        ],
        stages::Stage::Slice,
        &runner.work,
    )
    .unwrap();
    let reference = fs::read(reference_dir.join("plate_1.gcode")).unwrap();

    let actual = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(application::slice_project_plate_validated(
            project,
            ares_core::GenerationMetadata::deterministic(2026, 11, 26, 0, 0, 0),
            Some(1),
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    fs::write(runner.work.join("ares.gcode"), &actual).unwrap();

    let compared = crate::golden::compare_ordered_bytes_generator_only(&reference, &actual);
    fs::write(
        runner.work.join("opening-width-result.json"),
        serde_json::to_vec_pretty(&json!({
            "fixture": fixture_name,
            "input_sha256": expected_sha256,
            "reference_sha256": cache_identity::raw_digest(&reference),
            "ares_sha256": cache_identity::raw_digest(&actual),
            "match": compared.is_ok(),
            "detail": compared.as_ref().err(),
            "comparison": "complete bytes; generated-by line only",
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(
        compared.is_ok(),
        "counterbore opening width output differs ({fixture_name}): {:?}; {:?}",
        compared.err(),
        runner.work
    );
}

#[test]
fn counterbore_partiallybridge_rotated_slot_15_opening_width_matches_orca() {
    assert_rotated_slot_opening_width_matches_orca(
        "rotated-slot-15.3mf",
        include_bytes!(
            "../../../../../../tests/parity/counterbore-partiallybridge/rotated-slot-15.3mf"
        ),
        ROT15_SHA256,
    );
}

#[test]
fn counterbore_partiallybridge_rotated_slot_37_opening_width_matches_orca() {
    assert_rotated_slot_opening_width_matches_orca(
        "rotated-slot-37.3mf",
        include_bytes!(
            "../../../../../../tests/parity/counterbore-partiallybridge/rotated-slot-37.3mf"
        ),
        ROT37_SHA256,
    );
}

#[test]
fn counterbore_partiallybridge_rotated_slot_6p003_opening_width_matches_orca() {
    assert_rotated_slot_opening_width_matches_orca(
        "rotated-slot-6p003.3mf",
        include_bytes!(
            "../../../../../../tests/parity/counterbore-partiallybridge/rotated-slot-6p003.3mf"
        ),
        ROT6P003_SHA256,
    );
}
