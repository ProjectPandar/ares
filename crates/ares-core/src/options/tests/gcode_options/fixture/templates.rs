use std::collections::BTreeSet;

use super::load::Fixture;

const TEMPLATE_KEYS: [&str; 16] = [
    "before_layer_change_gcode",
    "printing_by_object_gcode",
    "machine_end_gcode",
    "layer_change_gcode",
    "time_lapse_gcode",
    "wrapping_detection_gcode",
    "file_start_gcode",
    "machine_start_gcode",
    "change_filament_gcode",
    "change_extrusion_role_gcode",
    "machine_pause_gcode",
    "template_custom_gcode",
    "process_change_extrusion_role_gcode",
    "filament_end_gcode",
    "filament_start_gcode",
    "filament_change_extrusion_role_gcode",
];

pub(super) fn assert_template_bytes(fixture: &Fixture) {
    let checked = BTreeSet::from([
        assert_scalar(
            fixture,
            "before_layer_change_gcode",
            &fixture.printer.before_layer_change_gcode.0,
            &fixture.projected.before_layer_change_gcode.0,
        ),
        assert_scalar(
            fixture,
            "printing_by_object_gcode",
            &fixture.printer.printing_by_object_gcode.0,
            &fixture.projected.printing_by_object_gcode.0,
        ),
        assert_scalar(fixture, "machine_end_gcode", &fixture.printer.machine_end_gcode.0, &fixture.projected.machine_end_gcode.0),
        assert_scalar(fixture, "layer_change_gcode", &fixture.printer.layer_change_gcode.0, &fixture.projected.layer_change_gcode.0),
        assert_scalar(fixture, "time_lapse_gcode", &fixture.printer.time_lapse_gcode.0, &fixture.projected.time_lapse_gcode.0),
        assert_scalar(fixture, "wrapping_detection_gcode", &fixture.printer.wrapping_detection_gcode.0, &fixture.projected.wrapping_detection_gcode.0),
        assert_scalar(fixture, "file_start_gcode", &fixture.printer.file_start_gcode.0, &fixture.projected.file_start_gcode.0),
        assert_scalar(fixture, "machine_start_gcode", &fixture.printer.machine_start_gcode.0, &fixture.projected.machine_start_gcode.0),
        assert_scalar(fixture, "change_filament_gcode", &fixture.printer.change_filament_gcode.0, &fixture.projected.change_filament_gcode.0),
        assert_scalar(fixture, "change_extrusion_role_gcode", &fixture.printer.change_extrusion_role_gcode.0, &fixture.projected.change_extrusion_role_gcode.0),
        assert_scalar(fixture, "machine_pause_gcode", &fixture.printer.machine_pause_gcode.0, &fixture.projected.machine_pause_gcode.0),
        assert_scalar(fixture, "template_custom_gcode", &fixture.printer.template_custom_gcode.0, &fixture.projected.template_custom_gcode.0),
        assert_scalar(
            fixture,
            "process_change_extrusion_role_gcode",
            &fixture.process.process_change_extrusion_role_gcode.0,
            &fixture.projected.process_change_extrusion_role_gcode.0,
        ),
        assert_vector(fixture, "filament_end_gcode", &fixture.filament.filament_end_gcode.0, &fixture.projected.filament_end_gcode.0),
        assert_vector(fixture, "filament_start_gcode", &fixture.filament.filament_start_gcode.0, &fixture.projected.filament_start_gcode.0),
        assert_vector(
            fixture,
            "filament_change_extrusion_role_gcode",
            &fixture.filament.filament_change_extrusion_role_gcode.0,
            &fixture.projected.filament_change_extrusion_role_gcode.0,
        ),
    ]);
    assert_eq!(checked, BTreeSet::from(TEMPLATE_KEYS));

    for key in ["machine_end_gcode", "machine_start_gcode"] {
        let value = fixture.raw[key].as_str().unwrap();
        assert!(value.contains('\n'), "{key} must remain multiline");
        assert!(value.ends_with('\n'), "{key} must retain its trailing newline");
    }
}

fn assert_scalar<'a>(fixture: &Fixture, key: &'a str, source: &str, projected: &str) -> &'a str {
    let raw = fixture.raw[key].as_str().unwrap();
    assert_eq!(source.as_bytes(), raw.as_bytes(), "{key} source bytes");
    assert_eq!(projected.as_bytes(), raw.as_bytes(), "{key} projected bytes");
    key
}

fn assert_vector<'a>(
    fixture: &Fixture,
    key: &'a str,
    source: &[String],
    projected: &[String],
) -> &'a str {
    let raw = fixture.raw[key].as_array().unwrap();
    assert_eq!(source.len(), raw.len(), "{key} source length");
    assert_eq!(projected.len(), raw.len(), "{key} projected length");
    for (index, raw) in raw.iter().enumerate() {
        let raw = raw.as_str().unwrap().as_bytes();
        assert_eq!(source[index].as_bytes(), raw, "{key}[{index}] source bytes");
        assert_eq!(projected[index].as_bytes(), raw, "{key}[{index}] projected bytes");
    }
    key
}
