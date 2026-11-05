//! `GCode.cpp:7386–7392` layer-start gate and first-travel command ordering.

#[path = "first_travel_acceleration_ordering/final_layer.rs"]
mod final_layer;
#[path = "first_travel_acceleration_ordering/inline_snapshot.rs"]
mod inline_snapshot;
#[path = "first_travel_acceleration_ordering/multihead_snapshot.rs"]
mod multihead_snapshot;

pub(crate) fn assert_multi_nozzle_prefix(
    directory: &std::path::Path,
    case: &str,
    native: &[u8],
    ares: &[u8],
    photo: &str,
) {
    multihead_snapshot::assert_prefix(directory, case, &commands(native), &commands(ares), photo);
}

pub(crate) fn assert_final_layer(directory: &std::path::Path, native: &[u8], ares: &[u8]) {
    final_layer::assert_prefix(directory, &commands(native), &commands(ares));
}

pub(crate) fn assert_first_travel_prefix(
    directory: &std::path::Path,
    case: &str,
    native: &[u8],
    ares: &[u8],
) {
    let native = commands(native);
    let ares = commands(ares);
    let target = native
        .iter()
        .position(|line| *line == "M204 S6000")
        .expect("Native first-travel acceleration command");
    let end = target + 5;
    let expected = &native[..end];
    let actual = ares.get(..end);
    let passed = Some(expected) == actual;
    let first_difference = native
        .iter()
        .zip(&ares)
        .position(|(expected, actual)| expected != actual)
        .unwrap_or_else(|| native.len().min(ares.len()));
    std::fs::write(
        directory.join("first-difference-section.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "case": case,
            "oracle": "first-difference section",
            "passed": passed,
            "section_command_index": target,
            "expected_section": &native[target..end],
            "actual_section": ares.get(target..end),
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
        "{case}: first-difference section oracle failed at command {first_difference}; Native={:?}; Ares={:?}; {directory:?}",
        native.get(first_difference),
        ares.get(first_difference),
    );
    inline_snapshot::assert_prefix(directory, case, &native, &ares);
}

fn commands(bytes: &[u8]) -> Vec<&str> {
    std::str::from_utf8(bytes)
        .unwrap()
        .lines()
        .filter_map(|line| {
            let line = line.split_once(';').map_or(line, |(command, _)| command);
            let line = line.trim();
            let command = line.split_whitespace().next()?;
            (command != "M73" && !command.starts_with("M73.")).then_some(line)
        })
        .collect()
}
