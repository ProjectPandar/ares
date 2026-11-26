//! `BridgeDetector::bridge_direction_candidates` deduplication tolerance parity.

use super::super::{OrcaRunner, application, cache_identity, stages};
use serde_json::json;
use std::fs;

const FIXTURE_SHA256: &str = "383cacd4f11bfda45a82c7525f40b00310c5684158848c644d34439a8a60b2dc";

#[test]
fn counterbore_partiallybridge_rotated_slot_candidate_direction_matches_orca() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project = include_bytes!(
        "../../../../../../tests/parity/counterbore-partiallybridge/rotated-slot-1p003.3mf"
    );
    assert_eq!(cache_identity::raw_digest(project), FIXTURE_SHA256);

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
        runner.work.join("candidate-direction-result.json"),
        serde_json::to_vec_pretty(&json!({
            "fixture": "rotated-slot-1p003.3mf",
            "input_sha256": FIXTURE_SHA256,
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
        "counterbore bridge candidate direction output differs: {:?}; {:?}",
        compared.err(),
        runner.work
    );
}
