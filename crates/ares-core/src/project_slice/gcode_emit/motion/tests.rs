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
