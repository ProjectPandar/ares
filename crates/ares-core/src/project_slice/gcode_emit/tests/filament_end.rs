#[tokio::test]
async fn filament_end_template_trims_initial_whitespace() {
    const SETTINGS: &str = "Metadata/project_settings.config";
    let mut archive = crate::project_slice::tests::support::KsrArchive::new();
    let mut settings: serde_json::Value =
        serde_json::from_str(&archive.entry_text(SETTINGS)).unwrap();
    settings["filament_end_gcode"] =
        serde_json::json!([" ; filament end gcode", " ; filament end gcode"]);
    settings["machine_end_gcode"] = serde_json::json!("");
    archive.insert_text(SETTINGS, &serde_json::to_string(&settings).unwrap());

    let output = crate::slice_project(
        &archive.bytes(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let output = std::str::from_utf8(&output).unwrap();

    assert!(output.contains("\n; filament end gcode\n"));
    assert!(!output.contains("\n ; filament end gcode\n"));
}
