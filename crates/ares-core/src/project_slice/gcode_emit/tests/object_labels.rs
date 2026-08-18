#[tokio::test]
async fn ksr_first_object_travel_uses_configured_acceleration_lift_and_deretraction() {
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
    let label = lines
        .iter()
        .position(|line| *line == "M624 AQAAAAAAAAA=")
        .unwrap();

    assert_eq!(lines[label - 4], "M204 S500");
    assert!(lines[label - 3].starts_with("; printing object "));
    assert_eq!(lines[label - 2], "M204 S6000");
    assert_eq!(
        lines[label - 1],
        "; start printing object, unique label id: 133"
    );
    assert!(lines[label + 1].starts_with("G1 X"));
    assert!(lines[label + 1].ends_with(" F60000"));
    assert_eq!(lines[label + 2], "G1 Z.6");
    assert_eq!(lines[label + 3], "G1 Z.2");
    assert_eq!(lines[label + 4], "G1 E.4 F1800");
}

#[tokio::test]
async fn ksr_project_emits_3mf_object_labels_per_layer() {
    let output = crate::slice_project(
        crate::project_slice::tests::support::ksr_project(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let output = std::str::from_utf8(&output).unwrap();

    assert_eq!(
        output
            .matches("; printing object ksr_fdmtest_v4.drc id:2 copy 0\n")
            .count(),
        460
    );
    assert_eq!(
        output
            .matches("; start printing object, unique label id: 133\nM624 AQAAAAAAAAA=\n")
            .count(),
        460
    );
    assert_eq!(
        output
            .matches("; stop printing object, unique label id: 133\nM625\n")
            .count(),
        460
    );
}

#[tokio::test]
async fn multi_object_project_uses_each_objects_identity() {
    let archive = crate::project_slice::tests::prepare_infill::bridge_over_infill::multi_object::two_object_archive();

    let output = crate::slice_project(
        &archive.bytes(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let output = std::str::from_utf8(&output).unwrap();

    assert!(output.contains("; printing object ksr_fdmtest_v4.drc id:2 copy 0\n"));
    assert!(output.contains("; printing object ksr_fdmtest_v4-copy.drc id:3 copy 0\n"));
}
#[tokio::test]
async fn gcode_label_objects_false_suppresses_project_object_labels() {
    let mut archive = crate::project_slice::tests::support::KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"gcode_label_objects\": \"1\"",
        "\"gcode_label_objects\": \"0\"",
    );

    let output = crate::slice_project(
        &archive.bytes(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let output = std::str::from_utf8(&output).unwrap();

    assert!(!output.contains("; printing object "));
    assert!(!output.contains("; start printing object, unique label id:"));
    assert!(!output.contains("; stop printing object "));
    assert!(!output.contains("; stop printing object, unique label id:"));
}
