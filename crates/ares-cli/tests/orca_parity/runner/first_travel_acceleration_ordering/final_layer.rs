//! `GCodeWriter.cpp:1096–1105`: same-Z travel precedes the unclear-position split.

pub(super) fn assert_prefix(directory: &std::path::Path, native: &[&str], ares: &[&str]) {
    let target = native.iter().position(|line| *line == "G1 F4182").unwrap();
    let capture = native
        .iter()
        .rposition(|line| *line == "M971 S11 C10 O0")
        .unwrap();
    let start = native[..capture]
        .iter()
        .rposition(|line| *line == "M622.1 S1")
        .unwrap();
    let close = capture
        + native[capture..]
            .iter()
            .position(|line| *line == "M623")
            .unwrap();
    assert!(target < start);
    let end = close + 6;
    assert_eq!(
        &native[close + 1..end],
        &[
            "M204 S10000",
            "G1 X131.91 Y130.677 F30000",
            "M204 S2000",
            "G1 F4182",
            "G1 X132.677 Y129.91 E.03356",
        ]
    );
    let feedrate_prefix_passed = ares.get(..target + 1) == Some(&native[..target + 1]);
    let approach_section_passed = ares.get(start..end) == Some(&native[start..end]);
    let first_difference = native
        .iter()
        .zip(ares)
        .position(|(expected, actual)| expected != actual)
        .unwrap_or_else(|| native.len().min(ares.len()));
    std::fs::write(
        directory.join("final-layer-feedrate-section.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "oracle": "final-layer feedrate prefix and same-Z photo approach",
            "passed": feedrate_prefix_passed && approach_section_passed,
            "feedrate_prefix_passed": feedrate_prefix_passed,
            "approach_section_passed": approach_section_passed,
            "feedrate_command_index": target,
            "expected_feedrate": native[target],
            "actual_feedrate": ares.get(target),
            "section_command_index": start,
            "expected_section": &native[start..end],
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
        feedrate_prefix_passed && approach_section_passed,
        "final-layer oracle failed at command {first_difference}; Native={:?}; Ares={:?}; {directory:?}",
        native.get(first_difference),
        ares.get(first_difference),
    );
}
