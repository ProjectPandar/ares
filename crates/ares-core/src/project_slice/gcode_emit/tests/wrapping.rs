//! `{max_layer_z}` wrapping-detection placeholder on the project emitter:
//! upstream's `m_max_layer_z` is the running FLOAT-domain max over the
//! RAW print z (`GCode.cpp:4629-4630`), updated before the template
//! renders (`GCode.cpp:5221`) — `z_offset` never enters it.

#[tokio::test]
async fn wrapping_max_layer_z_is_current_layer_raw_float_z() {
    const SETTINGS: &str = "Metadata/project_settings.config";
    let mut archive = crate::project_slice::tests::support::KsrArchive::new();
    let mut settings: serde_json::Value =
        serde_json::from_str(&archive.entry_text(SETTINGS)).unwrap();
    settings["enable_wrapping_detection"] = serde_json::json!("1");
    settings["wrapping_exclude_area"] = serde_json::json!(["0x0", "10x0", "10x10", "0x10"]);
    settings["wrapping_detection_gcode"] =
        serde_json::json!(";WRAP {layer_num} {layer_z} {max_layer_z}");
    archive.insert_text(SETTINGS, &serde_json::to_string(&settings).unwrap());

    let output = crate::slice_project(
        &archive.bytes(),
        crate::project_slice::tests::support::metadata(),
        crate::SlicerContext::new(0),
    )
    .await
    .unwrap();
    let lines = std::str::from_utf8(&output)
        .unwrap()
        .lines()
        .filter(|line| line.starts_with(";WRAP "))
        .collect::<Vec<_>>();

    // Each layer's wrap block must report the CURRENT layer as the max —
    // a stale one-layer-behind maximum would print `0.2` on layer 2.
    assert_eq!(lines.first().copied(), Some(";WRAP 0 0.2 0.2"));
    assert_eq!(lines.get(1).copied(), Some(";WRAP 1 0.4 0.4"));
    assert!(lines.len() >= 2);
}

#[tokio::test]
async fn wrapping_max_layer_z_excludes_z_offset() {
    const SETTINGS: &str = "Metadata/project_settings.config";
    let mut archive = crate::project_slice::tests::support::KsrArchive::new();
    let mut settings: serde_json::Value =
        serde_json::from_str(&archive.entry_text(SETTINGS)).unwrap();
    settings["enable_wrapping_detection"] = serde_json::json!("1");
    settings["z_offset"] = serde_json::json!("0.5");
    settings["wrapping_exclude_area"] = serde_json::json!(["0x0", "10x0", "10x10", "0x10"]);
    settings["wrapping_detection_gcode"] = serde_json::json!(";WRAP {layer_num} {max_layer_z}");
    archive.insert_text(SETTINGS, &serde_json::to_string(&settings).unwrap());

    let output = crate::slice_project(
        &archive.bytes(),
        crate::project_slice::tests::support::metadata(),
        crate::SlicerContext::new(0),
    )
    .await
    .unwrap();
    let lines = std::str::from_utf8(&output)
        .unwrap()
        .lines()
        .filter(|line| line.starts_with(";WRAP "))
        .collect::<Vec<_>>();

    // `z_offset` shifts the writer, not the placeholder
    // (`GCode.cpp:5221` inserts the raw `m_max_layer_z`).
    assert_eq!(lines.first().copied(), Some(";WRAP 0 0.2"));
}
