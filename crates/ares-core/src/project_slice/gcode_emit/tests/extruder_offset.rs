#[tokio::test]
async fn active_extruder_offset_translates_emitted_motion() {
    let mut archive = crate::project_slice::tests::support::KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"extruder_offset\": [\r\n\t\t\"0x0\",\r\n\t\t\"0x0\"\r\n\t]",
        "\"extruder_offset\": [\r\n\t\t\"0x0\",\r\n\t\t\"0x2\"\r\n\t]",
    );

    let output = crate::slice_project(
        archive.bytes(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let output = std::str::from_utf8(&output).unwrap();

    assert!(output.contains("G1 X144.504 Y98.092 E.63582\n"));
    assert!(!output.contains("G1 X144.504 Y100.092 E.63582\n"));
}
