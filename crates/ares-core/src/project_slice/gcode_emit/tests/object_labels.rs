#[tokio::test]
async fn ksr_object_travel_acceleration_follows_object_comment() {
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
    assert_eq!(lines[label + 1], "G1 Z.6 F60000");
    assert!(lines[label + 2].starts_with("G1 X"));
    assert_eq!(lines[label + 3], "G1 Z.6");
    assert_eq!(lines[label + 4], "G1 Z.2");
    assert_eq!(lines[label + 5], "G1 E.4 F1800");

    let layer_2 = lines
        .iter()
        .position(|line| *line == "M991 S0 P1 ;notify layer change")
        .unwrap();
    let printing = lines[layer_2..]
        .iter()
        .position(|line| line.starts_with("; printing object "))
        .unwrap()
        + layer_2;
    assert_eq!(lines[printing - 1], "M204 S10000");

    let layer_43 = lines
        .iter()
        .position(|line| *line == "M991 S0 P42 ;notify layer change")
        .unwrap();
    let printing = lines[layer_43..]
        .iter()
        .position(|line| line.starts_with("; printing object "))
        .unwrap()
        + layer_43;
    assert_eq!(lines[printing + 1], "M204 S10000");

    let layer_345 = lines
        .iter()
        .position(|line| *line == "M991 S0 P344 ;notify layer change")
        .unwrap();
    let printing = lines[layer_345..]
        .iter()
        .position(|line| line.starts_with("; printing object "))
        .unwrap()
        + layer_345;
    let start_label = lines[printing..]
        .iter()
        .position(|line| *line == "; start printing object, unique label id: 133")
        .unwrap()
        + printing;
    assert_eq!(lines[start_label + 1], "M624 AQAAAAAAAAA=");
    assert!(lines[start_label + 2].starts_with("G1 X"));
    assert!(lines[start_label + 2].contains(" Z"));
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
            .matches("; printing object ksr_fdmtest_v4.drc id:0 copy 0\n")
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

    assert!(output.contains("; printing object ksr_fdmtest_v4.drc id:0 copy 0\n"));
    assert!(output.contains("; printing object ksr_fdmtest_v4-copy.drc id:1 copy 0\n"));
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
