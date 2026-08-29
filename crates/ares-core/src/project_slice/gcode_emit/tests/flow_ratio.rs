#[tokio::test]
async fn project_print_flow_ratio_scales_extrusion() {
    let mut archive = crate::project_slice::tests::support::KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"print_flow_ratio\": \"1\"",
        "\"print_flow_ratio\": \"0.5\"",
    );

    let output = crate::slice_project(
        &archive.bytes(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let output = std::str::from_utf8(&output).unwrap();

    let start = output.find("; FEATURE: Inner wall\n").unwrap();
    let block = output[start..].lines().take(6).collect::<Vec<_>>();
    assert_eq!(
        block,
        [
            "; FEATURE: Inner wall",
            "; LINE_WIDTH: 0.5",
            "G1 F3000",
            "G1 X139.876 Y103.477 E.0137",
            "G1 X139.697 Y104.225 E.01433",
            "G1 X139.639 Y104.957 E.01367",
        ]
    );
}
