//! `GCode.cpp:6451–6455,9831–9853`: multi-nozzle photo-head inline placement.

pub(super) fn assert_prefix(
    directory: &std::path::Path,
    case: &str,
    native: &[&str],
    ares: &[&str],
    photo: &str,
) {
    let start = native.iter().position(|line| *line == "M622.1 S1").unwrap();
    let expected = [
        "M622.1 S1",
        "M1002 judge_flag timelapse_record_flag",
        "M622 J1",
        photo,
        "M623",
        "M204 S6000",
    ];
    let end = start + expected.len();
    let native_premise = native.get(start..end) == Some(expected.as_slice());
    let passed = native_premise && ares.get(..end) == native.get(..end);
    let first_difference = native
        .iter()
        .zip(ares)
        .position(|(expected, actual)| expected != actual)
        .unwrap_or_else(|| native.len().min(ares.len()));
    std::fs::write(
        directory.join("multihead-snapshot-section.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "case": case,
            "oracle": "multi-nozzle photo-head prefix",
            "native_premise": native_premise,
            "passed": passed,
            "section_command_index": start,
            "expected_section": expected,
            "native_section": native.get(start..end),
            "actual_section": ares.get(start..end),
            "first_difference": {
                "index": first_difference,
                "native": native.get(first_difference),
                "ares": ares.get(first_difference),
            },
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(
        native_premise,
        "{case}: Native inline-photo premise: {directory:?}"
    );
    assert!(
        passed,
        "{case}: multi-nozzle prefix failed at {first_difference}; Native={:?}; Ares={:?}; {directory:?}",
        native.get(first_difference),
        ares.get(first_difference),
    );
}
