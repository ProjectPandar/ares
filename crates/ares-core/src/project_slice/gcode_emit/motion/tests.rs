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
async fn first_layer_seam_and_island_order_match_project_slice() {
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
    let lines = output.lines().collect::<Vec<_>>();
    let first_helical_travel = lines
        .iter()
        .position(|line| line.starts_with("G3 Z.6 ") && line.ends_with(" F60000"))
        .unwrap();
    assert_eq!(lines[first_helical_travel + 1], "G1 X145.539 Y95.848 Z.6");
}

#[tokio::test]
async fn first_layer_inner_perimeter_uses_source_aligned_seam() {
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
    let layer_changes = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| **line == "; CHANGE_LAYER")
        .map(|(index, _)| index)
        .take(2)
        .collect::<Vec<_>>();
    let destination = lines[layer_changes[0]..layer_changes[1]]
        .iter()
        .enumerate()
        .filter(|(_, line)| line.starts_with("G3 Z") && line.ends_with(" F60000"))
        .nth(9)
        .map(|(index, _)| lines[layer_changes[0] + index + 1]);

    assert_eq!(destination, Some("G1 X151.343 Y94.919 Z.6"));
}

#[tokio::test]
async fn first_layer_second_inner_perimeter_uses_fitted_aligned_seam() {
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
    let layer_changes = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| **line == "; CHANGE_LAYER")
        .map(|(index, _)| index)
        .take(2)
        .collect::<Vec<_>>();
    let destination = lines[layer_changes[0]..layer_changes[1]]
        .iter()
        .enumerate()
        .filter(|(_, line)| line.starts_with("G3 Z") && line.ends_with(" F60000"))
        .nth(11)
        .map(|(index, _)| lines[layer_changes[0] + index + 1]);

    assert_eq!(destination, Some("G1 X140.545 Y90.801 Z.6"));
}

#[tokio::test]
async fn first_layer_linear_extrusions_use_project_geometry_lengths() {
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

    assert!(lines.windows(3).any(|window| {
        window
            == [
                "G1 X141.657 Y101.026 E.02865",
                "G1 X141.072 Y101.525 E.02865",
                "G1 X140.573 Y102.11 E.02866",
            ]
    }));
    assert!(lines.windows(3).any(|window| {
        window
            == [
                "G1 X129.699 Y83.135 E.31872",
                "G1 X129.052 Y83.135 E.02413",
                "G1 X133.577 Y87.66 E.23863",
            ]
    }));
}

#[tokio::test]
async fn extrusion_height_processor_state_tracks_layers_and_paths() {
    let output = crate::slice_project(
        crate::project_slice::tests::support::ksr_project(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let output = std::str::from_utf8(&output).unwrap();

    assert_eq!(output.matches("; LAYER_HEIGHT:").count(), 515);
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

#[tokio::test]
async fn task22o234_variable_overhang_segments_switch_the_configured_fan() {
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

    assert!(lines.windows(3).enumerate().any(|(index, window)| {
        index > 20_000
            && window[0] == "M106 S255"
            && window[1] == "G1 F3000"
            && window[2].starts_with("G1 X")
    }));
}
