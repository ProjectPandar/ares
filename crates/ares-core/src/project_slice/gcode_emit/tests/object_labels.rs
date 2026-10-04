#[tokio::test]
async fn multi_object_project_uses_each_objects_identity() {
    let archive = crate::project_slice::tests::prepare_infill::bridge_over_infill::multi_object::two_object_archive();

    let output = crate::slice_project(
        &archive.bytes(),
        crate::project_slice::tests::support::metadata(),
        crate::SlicerContext::new(0),
    )
    .await
    .unwrap();
    let output = std::str::from_utf8(&output).unwrap();

    assert!(output.contains("; printing object ksr_fdmtest_v4.drc id:0 copy 0\n"));
    // `set_object_info` never runs for BBL printers (`GCode.cpp:8075-8077`), so
    // `PrintObject::m_id` stays 0 for every object (`GCode.cpp:5349-5352`).
    assert!(output.contains("; printing object ksr_fdmtest_v4-copy.drc id:0 copy 0\n"));
}

#[tokio::test]
async fn nonprinting_prefix_does_not_supply_the_initial_emission_origin() {
    let mut archive = crate::project_slice::tests::prepare_infill::bridge_over_infill::multi_object::two_object_archive();
    let model = archive.entry_text("3D/3dmodel.model");
    archive.insert_text(
        "3D/3dmodel.model",
        &model.replacen("printable=\"1\"", "printable=\"0\"", 1),
    );
    archive.replace_unique(
        "Metadata/model_settings.config",
        "<part id=\"1\" subtype=\"normal_part\">",
        "<part id=\"1\" subtype=\"support_blocker\">",
    );
    let mut settings: serde_json::Value =
        serde_json::from_str(&archive.entry_text("Metadata/project_settings.config")).unwrap();
    settings["machine_start_gcode"] = serde_json::json!("");
    archive.insert_text(
        "Metadata/project_settings.config",
        &serde_json::to_string(&settings).unwrap(),
    );
    let output = crate::slice_project(
        &archive.bytes(),
        crate::project_slice::tests::support::metadata(),
        crate::SlicerContext::new(0),
    )
    .await
    .unwrap();
    let text = std::str::from_utf8(&output).unwrap();
    assert!(text.contains("G1 X144.504 Y100.092 E.63582\n"));
}

#[tokio::test]
async fn gcode_label_objects_false_suppresses_comments_and_bbl_labels() {
    let mut archive = crate::project_slice::tests::support::KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"gcode_label_objects\": \"1\"",
        "\"gcode_label_objects\": \"0\"",
    );

    let output = crate::slice_project(
        &archive.bytes(),
        crate::project_slice::tests::support::metadata(),
        crate::SlicerContext::new(0),
    )
    .await
    .unwrap();
    let output = std::str::from_utf8(&output).unwrap();

    // gcode_label_objects only suppresses the comment markers
    // (`GCode.cpp:5348-5352`); the exclusion markers follow exclude_object
    // independently (`GCode.cpp:5354-5372`). The KSR printer is BBL-based,
    // so the M624 labeling stays armed (`GCode.cpp:2583-2585`).
    assert!(!output.contains("; printing object "));
    assert!(!output.contains("; stop printing object "));
    assert!(output.contains("; start printing object, unique label id:"));
    assert!(output.contains("; stop printing object, unique label id:"));
}
