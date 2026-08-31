use super::{compare, compare_cross_target, compare_ignoring_time};

#[test]
fn layer_shaped_comments_in_start_gcode_remain_preamble() {
    let island = island(0, 1, 1_200);
    let expected = document("1m", "1m", "10s", "2.00", &[&island]).replacen(
        "; HEADER_BLOCK_END",
        ";Z:99\n;HEIGHT:1\n; HEADER_BLOCK_END",
        1,
    );

    compare(expected.as_bytes(), expected.as_bytes()).unwrap();
}

#[test]
fn fractional_duration_metadata_is_accepted() {
    let island = island(0, 1, 1_200);
    let expected = document("0.811081s", "1.499s", "0.25s", "2.00", &[&island]);
    let actual = document("1s", "1s", "0s", "2.00", &[&island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap();
}

#[test]
fn command_inline_comments_are_semantically_ignored() {
    let expected_island = "M204 S5000 ; acceleration\nM106 S102;fan comment\nG1 X0 Y0 F6000\n; FEATURE: Inner wall\n; LINE_WIDTH: 0.45\nG1 X1 Y0 E.1 F1200\n";
    let actual_island = expected_island
        .replace("M204 S5000 ; acceleration", "M204 S5000")
        .replace("M106 S102;fan comment", "M106 S102");
    let expected = document("1m", "1m", "10s", "2.00", &[expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap();
}

#[test]
fn motion_parameter_letters_in_inline_comments_are_ignored() {
    let expected_island = format!("G1 Z30 F960\n{}", island(0, 1, 1_200));
    let actual_island =
        expected_island.replace("G1 Z30 F960", "G1 Z30 F960 ; move Z down after heating");
    let expected = document("1m", "1m", "10s", "2.00", &[&expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap();
}

#[test]
fn source_variation_with_stable_island_order_is_tolerated() {
    let left = island(0, 1, 1_200);
    let right = island(10, 11, 1_200);
    let expected = document("1m 0s", "1m 5s", "10s", "2.00", &[&left, &right]);

    let left = island(0, 1, 1_205);
    let right = island(10, 11, 1_205);
    let actual = document("1m 2s", "1m 7s", "11s", "2.03", &[&left, &right]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap();
}

#[test]
fn cross_target_tolerates_quantized_path_split() {
    let expected_island = island(0, 1, 1_200);
    let expected = document("1m", "1m", "10s", "2.00", &[&expected_island]);
    let actual_island = expected_island.replace(
        "G1 X1 Y0 E.1 F1200",
        "G1 X.001 Y0 E.001 F1200\nG1 X1 Y0 E.099 F1200",
    );
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare_cross_target(expected.as_bytes(), actual.as_bytes()).unwrap();
}

#[test]
fn cross_target_rejects_large_path_drift() {
    let expected_island = island(0, 1, 1_200);
    let expected = document("1m", "1m", "10s", "2.00", &[&expected_island]);
    let actual_island = expected_island.replace("G1 X1 Y0 E.1 F1200", "G1 X1.2 Y0 E.1 F1200");
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    let error = compare_cross_target(expected.as_bytes(), actual.as_bytes()).unwrap_err();
    assert!(error.contains("deposition"), "{error}");
}

#[test]
fn independent_island_order_is_not_observable() {
    let left = island(0, 1, 1_200);
    let right = island(10, 11, 1_200);
    let expected = document("1m", "1m", "10s", "2.00", &[&left, &right]);
    let actual = document("1m", "1m", "10s", "2.00", &[&right, &left]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap();
}

#[test]
fn deposited_geometry_difference_is_rejected() {
    let island = island(0, 1, 1_200);
    let expected = document("1m", "1m", "10s", "2.00", &[&island]);
    let actual = expected.replace("X1 Y0 E.1", "X2 Y0 E.1");

    let error = compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
    assert!(error.contains("deposition 1 differs"), "{error}");
}

#[test]
fn source_variation_bounds_are_enforced() {
    let baseline = island(0, 1, 1_200);
    let expected = document("1m", "1m", "10s", "2.00", &[&baseline]);
    let bounded = document("1m", "1m", "10s", "2.00", &[&island(0, 1, 1_210)]);
    compare(expected.as_bytes(), bounded.as_bytes()).unwrap();

    let fast = document("1m", "1m", "10s", "2.00", &[&island(0, 1, 1_211)]);
    assert!(compare(expected.as_bytes(), fast.as_bytes()).is_err());

    let slow = document("1m 6s", "1m", "10s", "2.00", &[&baseline]);
    assert!(compare(expected.as_bytes(), slow.as_bytes()).is_err());

    let long = document("1m", "1m", "10s", "2.06", &[&baseline]);
    assert!(compare(expected.as_bytes(), long.as_bytes()).is_err());
}

#[test]
fn path_segmentation_difference_is_rejected() {
    let expected_island = island(0, 1, 1_200);
    let actual_island = expected_island.replace(
        "G1 X1 Y0 E.1 F1200",
        "G1 X.5 Y0 E.05 F1200\nG1 X1 Y0 E.05 F1200",
    );
    let expected = document("1m", "1m", "10s", "2.00", &[&expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    let error = compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
    assert!(error.contains("deposition"), "{error}");
}

#[test]
fn path_extrusion_redistribution_is_rejected() {
    let expected_island = format!("{}G1 X2 Y0 E.1 F1200\n", island(0, 1, 1_200));
    let actual_island = expected_island.replace(
        "G1 X1 Y0 E.1 F1200\nG1 X2 Y0 E.1",
        "G1 X1.06 Y0 E.106 F1200\nG1 X2 Y0 E.094",
    );
    let expected = document("1m", "1m", "10s", "2.00", &[&expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    let error = compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
    assert!(error.contains("deposition"), "{error}");
}

#[test]
fn wipe_and_retraction_lifecycle_must_match() {
    let expected_island = format!(
        "{}G1 E-.4 F1800\nG1 X1 Y1 E-.1 F12000\n",
        island(0, 1, 1_200)
    );
    let actual_island = expected_island.replace("E-.1", "E-.09");
    let expected = document("1m", "1m", "10s", "2.00", &[&expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    let error = compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
    assert!(error.contains("island lifecycle differs"), "{error}");
}

#[test]
fn wipe_extrusion_drift_beyond_one_formatting_step_is_rejected() {
    let expected_island = lifecycle_island(0, 1, ".1");
    let actual_island = expected_island.replace("E-.05 F1200", "E-.0501 F1200");
    let expected = document("1m", "1m", "10s", "2.00", &[&expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    let error = compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
    assert!(error.contains("island lifecycle differs"), "{error}");
}

#[test]
fn wipe_extrusion_last_decimal_drift_is_tolerated() {
    // The wipe distributes a fixed retraction over the just-printed path, so
    // sub-micron perimeter geometry — invisible at the emitted 3-decimal
    // coordinates — can flip the 5th decimal of one segment.
    let expected_island = lifecycle_island(0, 1, ".1");
    let actual_island = expected_island.replace("E-.05 F1200", "E-.05001 F1200");
    let expected = document("1m", "1m", "10s", "2.00", &[&expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap();
}

#[test]
fn wipe_coordinate_last_decimal_drift_is_rejected() {
    let expected_island = lifecycle_island(0, 1, ".1");
    let actual_island = expected_island.replace("G1 X0 Y0 E-.05", "G1 X.001 Y0 E-.05");
    let expected = document("1m", "1m", "10s", "2.00", &[&expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    let error = compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
    assert!(error.contains("island lifecycle differs"), "{error}");
}

#[test]
fn retract_and_unretract_order_difference_is_rejected() {
    let expected_island = "G1 E-.4 F1800\nG1 E.4 F1800\nM204 S5000\n; FEATURE: Inner wall\n; LINE_WIDTH: 0.45\nG1 X1 Y0 E.1 F1200\n";
    let actual_island = expected_island.replace("G1 E-.4 F1800\nG1 E.4", "G1 E.4 F1800\nG1 E-.4");
    let expected = document("1m", "1m", "10s", "2.00", &[expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    let error = compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
    assert!(error.contains("island lifecycle differs"), "{error}");
}

#[test]
fn retract_stays_associated_with_its_island_wipe() {
    let left = lifecycle_island(0, 1, ".1");
    let right = lifecycle_island(10, 11, ".2");
    let expected = document("1m", "1m", "10s", "2.00", &[&left, &right]);
    let actual = document(
        "1m",
        "1m",
        "10s",
        "2.00",
        &[
            &left.replace("E-.1 F1800", "E-.2 F1800"),
            &right.replace("E-.2 F1800", "E-.1 F1800"),
        ],
    );

    let error = compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
    assert!(error.contains("lifecycle"), "{error}");
}
#[test]
fn project_object_identity_difference_is_rejected() {
    let expected_island = format!(
        "; printing object part id:2 copy 0\n{}",
        island(0, 1, 1_200)
    );
    let actual_island = expected_island.replace("id:2", "id:3");
    let expected = document("1m", "1m", "10s", "2.00", &[&expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    let error = compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
    assert!(error.contains("control events differs"), "{error}");
}

#[test]
fn control_whitespace_difference_is_rejected() {
    let expected_island = format!("{}; custom control  \n", island(0, 1, 1_200));
    let actual_island = expected_island.replace("control  \n", "control\n");
    let expected = document("1m", "1m", "10s", "2.00", &[&expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    let error = compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
    assert!(error.contains("control events differs"), "{error}");
}

#[test]
fn control_blank_line_difference_is_rejected() {
    let expected_island = format!("{}; first\n\n; second\n", island(0, 1, 1_200));
    let actual_island = expected_island.replace("; first\n\n", "; first\n");
    let expected = document("1m", "1m", "10s", "2.00", &[&expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    let error = compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
    assert!(error.contains("control events differs"), "{error}");
}
fn lifecycle_island(start: u32, end: u32, retract: &str) -> String {
    format!(
        "G1 X{start} Y0 F6000\nG1 E.4 F1800\nM204 S5000\n; FEATURE: Outer wall\n; LINE_WIDTH: 0.42\nG1 X{end} Y0 E.1 F1200\nG1 E-{retract} F1800\n; WIPE_START\nG1 X{start} Y0 E-.05 F1200\n; WIPE_END\n"
    )
}

#[test]
fn travel_arc_geometry_difference_is_rejected() {
    let expected_island = "G3 X0 Y0 Z.4 I1 J2 P1 F6000\nM204 S5000\n; FEATURE: Inner wall\n; LINE_WIDTH: 0.45\nG1 X1 Y0 E.1 F1200\n";
    let actual_island = expected_island.replace("I1 J2 P1", "I2 J2 P1");
    let expected = document("1m", "1m", "10s", "2.00", &[expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    let error = compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
    assert!(error.contains("travel geometry differs"), "{error}");
}

#[test]
fn travel_arc_rounding_is_bounded() {
    let expected_island = "G3 X0 Y0 Z.4 I1 J2 P1 F6000\nM204 S5000\n; FEATURE: Inner wall\n; LINE_WIDTH: 0.45\nG1 X1 Y0 E.1 F1200\n";
    let actual_island = expected_island.replace("I1 J2 P1", "I1.001 J2 P1");
    let expected = document("1m", "1m", "10s", "2.00", &[expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap();

    let outside = actual.replace("I1.001 J2 P1", "I1.004 J2 P1");
    let error = compare(expected.as_bytes(), outside.as_bytes()).unwrap_err();
    assert!(error.contains("travel geometry differs"), "{error}");
}

#[test]
fn travel_lift_shape_difference_is_rejected() {
    let expected_island = "G1 X0 Y0 F6000\nG1 X2 Y0 Z.3\nG1 X0 Y0 Z.2\nM204 S5000\n; FEATURE: Inner wall\n; LINE_WIDTH: 0.45\nG1 X1 Y0 E.1 F1200\n";
    let actual_island = expected_island.replace("X2 Y0 Z.3", "X2 Y0 Z.4");
    let expected = document("1m", "1m", "10s", "2.00", &[expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    let error = compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
    assert!(error.contains("travel geometry differs"), "{error}");
}

#[test]
fn clockwise_travel_arc_geometry_difference_is_rejected() {
    let expected_island = "G2 X0 Y0 Z.4 I1 J2 P1 F6000\nM204 S5000\n; FEATURE: Inner wall\n; LINE_WIDTH: 0.45\nG1 X1 Y0 E.1 F1200\n";
    let actual_island = expected_island.replace("I1 J2 P1", "I2 J2 P1");
    let expected = document("1m", "1m", "10s", "2.00", &[expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    let error = compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
    assert!(error.contains("travel geometry differs"), "{error}");
}

#[test]
fn travel_feed_difference_is_rejected() {
    let expected_island = "G1 X0 Y0 F6000\nG1 F1200\nM204 S5000\n; FEATURE: Inner wall\n; LINE_WIDTH: 0.45\nG1 X1 Y0 E.1\n";
    let actual_island = expected_island.replacen("F6000", "F5000", 1);
    let expected = document("1m", "1m", "10s", "2.00", &[expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    let error = compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
    assert!(error.contains("travel feed differs"), "{error}");
}

#[test]
fn marlin_role_acceleration_words_match_equivalent_s_updates() {
    let expected_island = "M204 P5000 T9000\nG1 X0 Y0 F6000\n; FEATURE: Inner wall\n; LINE_WIDTH: 0.45\nG1 X1 Y0 E.1 F1200\n";
    let actual_island = "M204 S9000\nG1 X0 Y0 F6000\nM204 S5000\n; FEATURE: Inner wall\n; LINE_WIDTH: 0.45\nG1 X1 Y0 E.1 F1200\n";
    let expected = document("1m", "1m", "10s", "2.00", &[expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[actual_island]);

    compare(expected.as_bytes(), actual.as_bytes()).unwrap();
}

#[test]
fn legacy_file_start_stats_are_classified_not_compared_as_preamble() {
    let body = document("1m", "1m", "10s", "2.00", &[]);
    let expected = format!(";TIME:60.00\n;Filament used:0.24m\n{body}");
    let actual = format!(";TIME:90.00\n;Filament used:0.25m\n{body}");

    compare_ignoring_time(expected.as_bytes(), actual.as_bytes()).unwrap();
}

#[test]
fn travel_acceleration_difference_is_rejected() {
    let expected_island = "M204 S10000\nG1 X0 Y0 F6000\nM204 S5000\n; FEATURE: Inner wall\n; LINE_WIDTH: 0.45\nG1 X1 Y0 E.1 F1200\n";
    let actual_island = expected_island.replacen("S10000", "S9000", 1);
    let expected = document("1m", "1m", "10s", "2.00", &[expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    let error = compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
    assert!(error.contains("travel acceleration differs"), "{error}");
}

fn island(start: u32, end: u32, feed: u32) -> String {
    format!(
        "G1 X{start} Y0 F6000\nM204 S5000\n; FEATURE: Inner wall\n; LINE_WIDTH: 0.45\nG1 X{end} Y0 E.1 F{feed}\n"
    )
}

fn document(
    model_time: &str,
    total_time: &str,
    first_layer_time: &str,
    filament_length: &str,
    islands: &[&str],
) -> String {
    format!(
        "; HEADER_BLOCK_START\n; model printing time: {model_time}; total estimated time: {total_time}\n; estimated first layer printing time (normal mode) = {first_layer_time}\n; HEADER_BLOCK_END\n; CHANGE_LAYER\n; Z_HEIGHT: 0.2\n; LAYER_HEIGHT: 0.2\n{}; filament used [mm] = {filament_length}, 0.00\n; filament used [cm3] = 0.01, 0.00\n",
        islands.concat()
    )
}
