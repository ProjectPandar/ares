use super::sets_temperature;

#[test]
fn temperature_detection_ignores_comments_and_matches_commands() {
    assert!(!sets_temperature(
        "; M104 S220\nG28 ; M109 S220",
        &["M104", "M109"],
        false,
    ));
    assert!(sets_temperature(
        "G28\n  M104 S220 ; heat",
        &["M104", "M109"],
        false,
    ));
}

#[test]
fn reprap_g10_requires_a_temperature_parameter() {
    assert!(!sets_temperature(
        "G10 ; firmware retract",
        &["M104", "M109"],
        true,
    ));
    assert!(sets_temperature("G10 S220 P0", &["M104", "M109"], true,));
}
