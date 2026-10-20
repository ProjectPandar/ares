use crate::{
    OrcaRunner, golden,
    runner::native_api_cli::{NativeCase, sha},
};

#[test]
fn detect_off_requires_native_overhang_seam_choice() {
    check(
        "detect-off-extra1",
        include_bytes!("../../../../tests/parity/classic-seam-overhang/detect-off-extra1.3mf"),
        "b620c336d9c931cc008e8e5762c1b7b3592ca589536c760126f9e8c5e1d1cf3c",
    );
}

#[test]
fn detect_on_single_variable_control() {
    check(
        "detect-on-extra1",
        include_bytes!("../../../../tests/parity/classic-seam-overhang/detect-on-extra1.3mf"),
        "85e93cb93363cd43ce7d6007db39162871d6e9d6539679bb974926617c414fce",
    );
}

fn check(label: &str, project: &[u8], expected_sha: &str) {
    assert_eq!(sha(project), expected_sha, "frozen input SHA");
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let case = NativeCase::slice(&runner, project, &format!("classic-seam-overhang-{label}-"));
    let replay = NativeCase::slice(
        &runner,
        project,
        &format!("classic-seam-overhang-{label}-replay-"),
    );
    let first =
        golden::normalize_one_generator_line(case.native.as_bytes(), golden::GeneratorKind::Orca)
            .unwrap();
    let second =
        golden::normalize_one_generator_line(replay.native.as_bytes(), golden::GeneratorKind::Orca)
            .unwrap();
    let stable = golden::first_difference(&first, &second)
        .map_or(Ok(()), |difference| Err(difference.to_string()));
    std::fs::write(
        case.root.join("native-replay.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "case": label,
            "input_sha256": expected_sha,
            "reference_root": case.root,
            "replay_root": replay.root,
            "native_replay_comparison": stable,
            "upstream": ["GCode/SeamPlacer.cpp:1058-1093", "Line.hpp:251-266", "AABBTreeLines.hpp:329-350"]
        }))
        .unwrap(),
    )
    .unwrap();
    stable.unwrap_or_else(|error| panic!("Native nondeterminism at {:?}: {error}", case.root));
    case.compare(ares_core::GenerationMetadata::deterministic(
        2026, 8, 27, 0, 0, 0,
    ));
}
