//! `GCode.cpp:9831–9853`: the inline photo precedes the next path travel.

pub(super) fn assert_prefix(
    directory: &std::path::Path,
    case: &str,
    native: &[&str],
    ares: &[&str],
) {
    let start = native.iter().position(|line| *line == "M622.1 S1").unwrap();
    let close = start
        + native[start..]
            .iter()
            .position(|line| *line == "M623")
            .unwrap();
    assert!(native[start..close].contains(&"M971 S11 C10 O0"));
    assert_eq!(native[close + 1], "M204 S6000");
    let end = close + 2;
    let passed = ares.get(..end) == Some(&native[..end]);
    let first_difference = native
        .iter()
        .zip(ares)
        .position(|(expected, actual)| expected != actual)
        .unwrap_or_else(|| native.len().min(ares.len()));
    std::fs::write(
        directory.join("inline-snapshot-section.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "case": case,
            "oracle": "inline farthest-point snapshot prefix",
            "passed": passed,
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
        passed,
        "{case}: inline snapshot oracle failed at command {first_difference}; Native={:?}; Ares={:?}; {directory:?}",
        native.get(first_difference),
        ares.get(first_difference),
    );
}
