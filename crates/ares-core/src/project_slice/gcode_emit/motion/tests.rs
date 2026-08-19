#[test]
fn internal_travel_retraction_matches_source_role_rules() {
    assert!(super::path::can_skip_retraction(
        true,
        Some("Sparse infill"),
        false,
        true,
    ));
    assert!(!super::path::can_skip_retraction(
        true,
        Some("Outer wall"),
        false,
        true,
    ));
    assert!(!super::path::can_skip_retraction(
        true,
        Some("Overhang wall"),
        false,
        true,
    ));
    assert!(!super::path::can_skip_retraction(
        true,
        Some("Sparse infill"),
        true,
        true,
    ));
}

#[tokio::test]
async fn task22o133_contiguous_same_speed_paths_do_not_repeat_feedrate() {
    let output = crate::slice_project(
        crate::project_slice::tests::support::ksr_project(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let lines = std::str::from_utf8(&output)
        .unwrap()
        .lines()
        .collect::<Vec<_>>();
    let object_start = lines
        .iter()
        .position(|line| *line == "M624 AQAAAAAAAAA=")
        .unwrap();
    let first_outer = lines[object_start..]
        .iter()
        .position(|line| *line == "; FEATURE: Outer wall")
        .map(|index| object_start + index)
        .unwrap();

    assert_eq!(
        lines[object_start..first_outer]
            .iter()
            .filter(|line| **line == "G1 F3000")
            .count(),
        1
    );
}

#[tokio::test]
async fn task22o134_overhang_role_uses_bridge_kinematics() {
    let output = crate::slice_project(
        crate::project_slice::tests::support::ksr_project(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let lines = std::str::from_utf8(&output)
        .unwrap()
        .lines()
        .collect::<Vec<_>>();
    let feature = lines
        .iter()
        .position(|line| *line == "; FEATURE: Overhang wall")
        .unwrap();

    assert_eq!(lines[feature - 1], "M204 S2500");
    assert_eq!(lines[feature + 2], "G1 F3000");
}

#[tokio::test]
async fn task22o135_overhang_overlap_bands_split_and_slow_wall_segments() {
    let output = crate::slice_project(
        crate::project_slice::tests::support::ksr_project(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let lines = std::str::from_utf8(&output)
        .unwrap()
        .lines()
        .collect::<Vec<_>>();
    let overhang = lines
        .iter()
        .position(|line| *line == "; FEATURE: Overhang wall")
        .unwrap();
    let following_inner = lines[overhang + 1..]
        .iter()
        .position(|line| *line == "; FEATURE: Inner wall")
        .map(|index| overhang + 1 + index)
        .unwrap();

    assert_eq!(
        &lines[overhang - 5..overhang],
        [
            "G1 F3000",
            "G1 X114.789 Y81.637 E.02836",
            "G1 F1980",
            "G1 X114.989 Y81.637 E.00663",
            "M204 S2500",
        ]
    );
    assert_eq!(lines[following_inner + 2], "G1 F1980");
}

#[tokio::test]
async fn task22o136_dynamic_segment_extrusion_uses_quantized_endpoints() {
    let output = crate::slice_project(
        crate::project_slice::tests::support::ksr_project(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let lines = std::str::from_utf8(&output)
        .unwrap()
        .lines()
        .collect::<Vec<_>>();
    let overhang = lines
        .iter()
        .position(|line| *line == "; FEATURE: Overhang wall")
        .unwrap();
    let following_inner = lines[overhang + 1..]
        .iter()
        .position(|line| *line == "; FEATURE: Inner wall")
        .map(|index| overhang + 1 + index)
        .unwrap();

    assert_eq!(lines[following_inner + 3], "G1 X116.989 Y81.637 E.06303");
}

#[tokio::test]
async fn task22o137_dynamic_overhang_rounds_original_speed() {
    let output = crate::slice_project(
        crate::project_slice::tests::support::ksr_project(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let lines = std::str::from_utf8(&output)
        .unwrap()
        .lines()
        .collect::<Vec<_>>();
    let overhang = lines
        .iter()
        .position(|line| *line == "; FEATURE: Overhang wall")
        .unwrap();
    let first_inner = lines[..overhang]
        .iter()
        .rposition(|line| *line == "; FEATURE: Inner wall")
        .unwrap();

    assert_eq!(lines[first_inner + 2], "G1 F15780");
    assert_eq!(lines[first_inner + 3], "G1 F15791.926");
}

#[tokio::test]
async fn task22o138_cooling_removes_redundant_feedrate_commands() {
    let output = crate::slice_project(
        crate::project_slice::tests::support::ksr_project(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let lines = std::str::from_utf8(&output)
        .unwrap()
        .lines()
        .collect::<Vec<_>>();
    let overhang = lines
        .iter()
        .position(|line| *line == "; FEATURE: Overhang wall")
        .unwrap();
    let first_inner = lines[..overhang]
        .iter()
        .rposition(|line| *line == "; FEATURE: Inner wall")
        .unwrap();

    assert_eq!(
        lines[first_inner + 2..overhang]
            .iter()
            .filter(|line| **line == "G1 F15791.926")
            .count(),
        1
    );
}

#[tokio::test]
async fn task22o141_mesh_simplification_matches_source_wall_vertices() {
    let output = crate::slice_project(
        crate::project_slice::tests::support::ksr_project(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let lines = std::str::from_utf8(&output)
        .unwrap()
        .lines()
        .collect::<Vec<_>>();
    let overhang = lines
        .iter()
        .position(|line| *line == "; FEATURE: Overhang wall")
        .unwrap();
    let first_inner = lines[..overhang]
        .iter()
        .rposition(|line| *line == "; FEATURE: Inner wall")
        .unwrap();

    assert!(lines[first_inner + 4].starts_with("G1 X132.523 Y100.347 E"));
    assert_eq!(
        &lines[first_inner + 5..first_inner + 11],
        [
            "G1 X132.28 Y100.359 E.00807",
            "G1 X132.023 Y100.397 E.00862",
            "G1 X131.706 Y100.484 E.0109",
            "G1 X131.348 Y100.632 E.01285",
            "G1 X131.069 Y100.792 E.01067",
            "G1 X130.67 Y101.119 E.01711",
        ]
    );
}

#[tokio::test]
async fn aligned_seam_uses_project_slice_embedding() {
    let output = crate::slice_project(
        crate::project_slice::tests::support::ksr_project(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let output = std::str::from_utf8(&output).unwrap();

    let mut lines = output
        .lines()
        .skip_while(|line| *line != "M624 AQAAAAAAAAA=");
    assert_eq!(lines.nth(1), Some("G1 X140.158 Y102.797 F60000"));
}

#[tokio::test]
async fn bottom_surface_travel_crossing_external_slice_retracts() {
    let output = crate::slice_project(
        crate::project_slice::tests::support::ksr_project(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let output = std::str::from_utf8(&output).unwrap();

    assert!(
        output
            .contains("G1 X101.68 Y139.896 E.50049\nM204 S6000\nG1 E-.11429 F1800\n; WIPE_START\n")
    );
}
