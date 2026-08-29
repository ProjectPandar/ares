#[tokio::test]
async fn before_layer_change_layer_num_is_zero_based() {
    let mut archive = crate::project_slice::tests::support::KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"before_layer_change_gcode\": \"\"",
        "\"before_layer_change_gcode\": \";BEFORE {layer_num} {layer_z}\"",
    );

    let output = crate::slice_project(
        &archive.bytes(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let lines = std::str::from_utf8(&output)
        .unwrap()
        .lines()
        .filter(|line| line.starts_with(";BEFORE "))
        .take(2)
        .collect::<Vec<_>>();

    assert_eq!(lines, [";BEFORE 0 0.2", ";BEFORE 1 0.4"]);
}

#[tokio::test]
async fn machine_end_layer_num_is_the_final_zero_based_index() {
    const SETTINGS: &str = "Metadata/project_settings.config";
    let mut archive = crate::project_slice::tests::support::KsrArchive::new();
    let mut settings: serde_json::Value =
        serde_json::from_str(&archive.entry_text(SETTINGS)).unwrap();
    settings["machine_end_gcode"] = serde_json::json!(";END_LAYER {layer_num}");
    archive.insert_text(SETTINGS, &serde_json::to_string(&settings).unwrap());

    let output = crate::slice_project(
        &archive.bytes(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let output = std::str::from_utf8(&output).unwrap();

    assert!(output.contains(";END_LAYER 459\n"));
}
