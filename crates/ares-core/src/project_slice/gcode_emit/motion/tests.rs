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
