#[tokio::test]
async fn dynamic_outer_wall_travels_to_the_unprocessed_first_point() {
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
    let wipe_end = lines
        .windows(4)
        .position(|lines| {
            lines[0] == "; WIPE_END" && lines[1] == "G17" && lines[3] == "G1 X133.539 Y89.629 Z1.2"
        })
        .unwrap();

    assert_eq!(lines[wipe_end + 2], "G3 Z1.2 I-.045 J1.216 P1  F60000");
    assert_eq!(lines[wipe_end + 3], "G1 X133.539 Y89.629 Z1.2");
}
