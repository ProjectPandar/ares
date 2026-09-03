#[tokio::test]
async fn non_bbl_non_traditional_timelapse_not_emitted() {
    // Upstream only inserts layer-change timelapse for BBL printers
    // (`GCode.cpp:5205-5210`) or via the traditional i3/multi-nozzle
    // path (`GCode.cpp:5264-5300`). A non-BBL, non-i3, single-nozzle
    // printer with `time_lapse_gcode` set emits no layer-change
    // timelapse at all (e.g. Artillery M1 Pro with
    // `printer_structure: undefine`).
    const SETTINGS: &str = "Metadata/project_settings.config";
    let mut archive = crate::project_slice::tests::support::KsrArchive::new();
    let mut settings: serde_json::Value =
        serde_json::from_str(&archive.entry_text(SETTINGS)).unwrap();
    settings["printer_model"] = serde_json::json!("Qidi Q1 Pro");
    settings["printer_structure"] = serde_json::json!("undefine");
    settings["nozzle_diameter"] = serde_json::json!(["0.4"]);
    settings["before_layer_change_gcode"] = serde_json::json!(";BEFORE");
    settings["layer_change_gcode"] = serde_json::json!(";AFTER");
    settings["time_lapse_gcode"] = serde_json::json!(";TIMELAPSE");
    archive.insert_text(SETTINGS, &serde_json::to_string(&settings).unwrap());

    let output = crate::slice_project(
        &archive.bytes(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let lines = std::str::from_utf8(&output)
        .unwrap()
        .lines()
        .collect::<Vec<_>>();
    let before = lines.iter().position(|line| *line == ";BEFORE").unwrap();
    let after = lines.iter().position(|line| *line == ";AFTER").unwrap();

    assert!(before < after);
    assert!(
        !lines.iter().any(|line| *line == ";TIMELAPSE"),
        "non-BBL, non-traditional layer-change timelapse must not emit"
    );
}

#[tokio::test]
async fn traditional_timelapse_runs_between_perimeters_and_infill() {
    const SETTINGS: &str = "Metadata/project_settings.config";
    let mut archive = crate::project_slice::tests::support::KsrArchive::new();
    let mut settings: serde_json::Value =
        serde_json::from_str(&archive.entry_text(SETTINGS)).unwrap();
    settings["printer_structure"] = serde_json::json!("i3");
    settings["spiral_mode"] = serde_json::json!("0");
    settings["time_lapse_gcode"] = serde_json::json!(";TIMELAPSE {max_layer_z}");
    archive.insert_text(SETTINGS, &serde_json::to_string(&settings).unwrap());

    let output = crate::slice_project(
        &archive.bytes(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let lines = std::str::from_utf8(&output)
        .unwrap()
        .lines()
        .collect::<Vec<_>>();
    let outer = lines
        .iter()
        .position(|line| *line == "; FEATURE: Outer wall")
        .unwrap();
    let timelapse = lines
        .iter()
        .position(|line| *line == ";TIMELAPSE 0.2")
        .unwrap();
    let bottom = lines
        .iter()
        .position(|line| *line == "; FEATURE: Bottom surface")
        .unwrap();

    assert!(outer < timelapse);
    assert!(timelapse < bottom);
    let second_layer = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| **line == "; CHANGE_LAYER")
        .nth(1)
        .unwrap()
        .0;
    let first_stop = lines.iter().position(|line| *line == "M625").unwrap();
    let notification = lines
        .iter()
        .position(|line| *line == "M991 S0 P1 ;notify layer change")
        .unwrap();
    assert!(second_layer < first_stop);
    assert!(first_stop < notification);
}
