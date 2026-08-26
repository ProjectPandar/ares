use super::compare;

#[test]
fn independent_island_order_is_not_observable() {
    let left = island(0, 1, 1_200);
    let right = island(10, 11, 1_200);
    let expected = document("1m 0s", "1m 5s", "10s", "2.00", &[&left, &right]);

    let left = island(0, 1, 1_205);
    let right = island(10, 11, 1_205);
    let actual = document("1m 2s", "1m 7s", "11s", "2.03", &[&right, &left]);

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

    let fast = document("1m", "1m", "10s", "2.00", &[&island(0, 1, 1_211)]);
    assert!(compare(expected.as_bytes(), fast.as_bytes()).is_err());

    let slow = document("1m 6s", "1m", "10s", "2.00", &[&baseline]);
    assert!(compare(expected.as_bytes(), slow.as_bytes()).is_err());

    let long = document("1m", "1m", "10s", "2.06", &[&baseline]);
    assert!(compare(expected.as_bytes(), long.as_bytes()).is_err());
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
    assert!(error.contains("wipe paths differs"), "{error}");
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
fn travel_arc_geometry_difference_is_rejected() {
    let expected_island = "G3 X0 Y0 Z.4 I1 J2 P1 F6000\nM204 S5000\n; FEATURE: Inner wall\n; LINE_WIDTH: 0.45\nG1 X1 Y0 E.1 F1200\n";
    let actual_island = expected_island.replace("I1 J2 P1", "I2 J2 P1");
    let expected = document("1m", "1m", "10s", "2.00", &[expected_island]);
    let actual = document("1m", "1m", "10s", "2.00", &[&actual_island]);

    let error = compare(expected.as_bytes(), actual.as_bytes()).unwrap_err();
    assert!(error.contains("lift lifecycle differs"), "{error}");
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
