use crate::{
    SliceError, SliceOptions, gcode::format_gcode, pipeline::test_support::rectangular_pipeline,
};
use serde_json::{Value, json};

#[test]
fn notes_header_comments_emit_non_empty_profile_notes() {
    let options = options(json!({ "notes": "Calibrated profile" }));
    let gcode = gcode(&options);

    assert!(gcode.contains("; notes = Calibrated profile\n"));
}

#[test]
fn notes_header_comments_skip_missing_or_empty_profile_notes() {
    let missing = gcode(&options(json!({})));
    let empty = gcode(&options(json!({ "notes": "" })));

    assert!(!missing.contains("; notes ="));
    assert!(!empty.contains("; notes ="));
}

#[test]
fn notes_header_comments_split_multiline_profile_notes() {
    let options = options(json!({ "notes": "Line A\n\nLine B\n" }));
    let gcode = gcode(&options);

    assert!(gcode.contains("; notes = Line A\n; notes = \n; notes = Line B\n"));
    assert!(!gcode.contains("\nLine B\n"));
}

#[test]
fn notes_header_comments_reject_non_string_notes() {
    let options = options(json!({ "notes": ["bad"] }));
    let pipeline = rectangular_pipeline(&options);
    let err = format_gcode(&pipeline, &options).unwrap_err();

    assert!(matches!(
        err,
        SliceError::InvalidInput(message) if message == "notes must be a string"
    ));
}

#[test]
fn notes_header_comments_follow_btt_thumbnail_header_suppression() {
    let options = options(json!({
        "notes": "Hidden by BTT header suppression",
        "thumbnails": "64x64/BTT_TFT"
    }));
    let gcode = gcode(&options);

    assert!(!gcode.contains("; notes ="));
}

#[test]
fn filament_notes_header_comments_emit_string_vector_entries() {
    let options = options(json!({
        "filament_notes": ["PLA dry", "Second spool"]
    }));
    let gcode = gcode(&options);

    assert!(gcode.contains("; filament_notes = PLA dry\n"));
    assert!(gcode.contains("; filament_notes = Second spool\n"));
}

#[test]
fn filament_notes_header_comments_accept_scalar_and_split_multiline_entries() {
    let vector = gcode(&options(json!({
        "filament_notes": ["Line A\n\nLine B", ""]
    })));
    let scalar = gcode(&options(json!({
        "filament_notes": "Single filament note"
    })));

    assert!(
        vector.contains(
            "; filament_notes = Line A\n; filament_notes = \n; filament_notes = Line B\n"
        )
    );
    assert!(scalar.contains("; filament_notes = Single filament note\n"));
}

#[test]
fn printer_notes_header_comments_emit_non_empty_notes() {
    let options = options(json!({ "printer_notes": "Garage printer" }));
    let gcode = gcode(&options);

    assert!(gcode.contains("; printer_notes = Garage printer\n"));
}

#[test]
fn filament_and_printer_notes_skip_missing_and_empty_values() {
    let missing = gcode(&options(json!({})));
    let empty = gcode(&options(json!({
        "filament_notes": ["", ""],
        "printer_notes": ""
    })));

    assert!(!missing.contains("; filament_notes ="));
    assert!(!missing.contains("; printer_notes ="));
    assert!(!empty.contains("; filament_notes ="));
    assert!(!empty.contains("; printer_notes ="));
}

#[test]
fn filament_and_printer_notes_reject_invalid_values() {
    for (key, extra) in [
        ("filament_notes", json!({ "filament_notes": [7] })),
        ("filament_notes", json!({ "filament_notes": true })),
        ("filament_notes", json!({ "filament_notes": 7 })),
        (
            "filament_notes",
            json!({ "filament_notes": { "value": "bad" } }),
        ),
        ("filament_notes", json!({ "filament_notes": null })),
        ("printer_notes", json!({ "printer_notes": ["bad"] })),
        ("printer_notes", json!({ "printer_notes": 7 })),
        ("printer_notes", json!({ "printer_notes": null })),
    ] {
        let options = options(extra);
        let pipeline = rectangular_pipeline(&options);
        let err = format_gcode(&pipeline, &options).unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)), "{key}");
        assert!(err.to_string().contains(key), "{key}: {err}");
    }
}

#[test]
fn all_note_header_comments_follow_btt_thumbnail_header_suppression() {
    let options = options(json!({
        "notes": "Hidden process note",
        "filament_notes": ["Hidden filament note"],
        "printer_notes": "Hidden printer note",
        "thumbnails": "64x64/BTT_TFT"
    }));
    let gcode = gcode(&options);

    assert!(!gcode.contains("; notes ="));
    assert!(!gcode.contains("; filament_notes ="));
    assert!(!gcode.contains("; printer_notes ="));
}

#[test]
fn changing_only_profile_notes_preserves_command_lines() {
    let baseline = command_lines(&gcode(&options(json!({}))));
    let noted = command_lines(&gcode(&options(json!({
        "filament_notes": ["PLA dry"],
        "printer_notes": "Garage printer"
    }))));

    assert_eq!(baseline, noted);
}

fn gcode(options: &SliceOptions) -> String {
    String::from_utf8(format_gcode(&rectangular_pipeline(options), options).unwrap()).unwrap()
}

fn command_lines(gcode: &str) -> Vec<String> {
    gcode
        .lines()
        .filter(|line| line.starts_with('G') || line.starts_with('M'))
        .map(str::to_owned)
        .collect()
}

fn options(extra: Value) -> SliceOptions {
    let mut value = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 1,
        "line_width": 0.4,
        "sparse_infill_density": 0,
        "skirt_loops": 0,
        "brim_width": 0.0
    });
    value
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    serde_json::from_value(value).unwrap()
}
