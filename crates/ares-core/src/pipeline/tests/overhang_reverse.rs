use super::*;
use serde_json::json;

#[test]
fn overhang_reverse_changes_unsupported_second_layer_gcode_path_order() {
    let off = options(json!({ "overhang_reverse": false }));
    let on = options(json!({ "overhang_reverse": true }));

    let off_gcode = String::from_utf8(
        crate::gcode::format_gcode(
            &crate::pipeline::test_support::unsupported_second_layer_pipeline(&off),
            &off,
        )
        .unwrap(),
    )
    .unwrap();
    let on_gcode = String::from_utf8(
        crate::gcode::format_gcode(
            &crate::pipeline::test_support::unsupported_second_layer_pipeline(&on),
            &on,
        )
        .unwrap(),
    )
    .unwrap();

    assert!(off_gcode.contains(";PERIMETER:overhang:10,0 -> 14,0 -> 14,4 -> 10,4"));
    assert!(on_gcode.contains(";PERIMETER:overhang:10,4 -> 14,4 -> 14,0 -> 10,0"));
    assert!(off_gcode.contains(";PRINT_PATH:overhang_perimeter:10,0 -> 14,0 -> 14,4 -> 10,4"));
    assert!(on_gcode.contains(";PRINT_PATH:overhang_perimeter:10,4 -> 14,4 -> 14,0 -> 10,0"));
    assert_eq!(
        overhang_print_moves(&off_gcode)[..2],
        [
            ";MOVE:print:overhang_perimeter:14,0",
            ";MOVE:print:overhang_perimeter:14,4",
        ]
    );
    assert_eq!(
        overhang_print_moves(&on_gcode)[..2],
        [
            ";MOVE:print:overhang_perimeter:14,4",
            ";MOVE:print:overhang_perimeter:14,0",
        ]
    );
    assert_ne!(off_gcode, on_gcode);
}

#[test]
fn overhang_reverse_internal_only_preserves_external_gcode_order_and_reverses_internal_order() {
    let all = options(json!({
        "wall_loops": 2,
        "overhang_reverse": true,
        "overhang_reverse_internal_only": false
    }));
    let internal_only = options(json!({
        "wall_loops": 2,
        "overhang_reverse": true,
        "overhang_reverse_internal_only": true
    }));

    let all_gcode = String::from_utf8(
        crate::gcode::format_gcode(
            &crate::pipeline::test_support::unsupported_second_layer_pipeline(&all),
            &all,
        )
        .unwrap(),
    )
    .unwrap();
    let internal_only_gcode = String::from_utf8(
        crate::gcode::format_gcode(
            &crate::pipeline::test_support::unsupported_second_layer_pipeline(&internal_only),
            &internal_only,
        )
        .unwrap(),
    )
    .unwrap();

    assert!(all_gcode.contains(";PRINT_PATH:overhang_perimeter:10,4 -> 14,4 -> 14,0 -> 10,0"));
    assert!(
        internal_only_gcode.contains(";PRINT_PATH:overhang_perimeter:10,0 -> 14,0 -> 14,4 -> 10,4")
    );
    assert!(
        all_gcode.contains(
            ";PRINT_PATH:internal_perimeter:10.4,3.6 -> 13.6,3.6 -> 13.6,0.4 -> 10.4,0.4"
        )
    );
    assert!(
        internal_only_gcode.contains(
            ";PRINT_PATH:internal_perimeter:10.4,3.6 -> 13.6,3.6 -> 13.6,0.4 -> 10.4,0.4"
        )
    );
    assert_eq!(
        layer_internal_print_moves(&internal_only_gcode, 1)[..2],
        [
            ";MOVE:print:internal_perimeter:13.6,3.6",
            ";MOVE:print:internal_perimeter:13.6,0.4",
        ]
    );
    assert_ne!(all_gcode, internal_only_gcode);
}

#[test]
fn overhang_reverse_threshold_suppresses_reversal_when_larger_than_unsupported_span() {
    let options = options(json!({
        "overhang_reverse": true,
        "overhang_reverse_threshold": 20
    }));

    let gcode = String::from_utf8(
        crate::gcode::format_gcode(
            &crate::pipeline::test_support::unsupported_second_layer_pipeline(&options),
            &options,
        )
        .unwrap(),
    )
    .unwrap();

    assert!(gcode.contains(";PRINT_PATH:overhang_perimeter:10,0 -> 14,0 -> 14,4 -> 10,4"));
    assert_eq!(
        overhang_print_moves(&gcode)[..2],
        [
            ";MOVE:print:overhang_perimeter:14,0",
            ";MOVE:print:overhang_perimeter:14,4",
        ]
    );
}

#[test]
fn overhang_reverse_threshold_zero_reverses_unsupported_second_layer() {
    let options = options(json!({
        "overhang_reverse": true,
        "overhang_reverse_threshold": 0
    }));

    let gcode = String::from_utf8(
        crate::gcode::format_gcode(
            &crate::pipeline::test_support::unsupported_second_layer_pipeline(&options),
            &options,
        )
        .unwrap(),
    )
    .unwrap();

    assert!(gcode.contains(";PRINT_PATH:overhang_perimeter:10,4 -> 14,4 -> 14,0 -> 10,0"));
}

#[test]
fn disabled_overhang_detection_reverses_external_perimeter_and_ignores_threshold() {
    let options = options(json!({
        "detect_overhang_wall": false,
        "overhang_reverse": true,
        "overhang_reverse_threshold": 20
    }));

    let gcode = String::from_utf8(
        crate::gcode::format_gcode(
            &crate::pipeline::test_support::unsupported_second_layer_pipeline(&options),
            &options,
        )
        .unwrap(),
    )
    .unwrap();

    assert!(gcode.contains(";PRINT_PATH:external_perimeter:10,4 -> 14,4 -> 14,0 -> 10,0"));
    assert!(!gcode.contains(";PRINT_PATH:overhang_perimeter:"));
    assert_eq!(
        layer_external_print_moves(&gcode, 1)[..2],
        [
            ";MOVE:print:external_perimeter:14,4",
            ";MOVE:print:external_perimeter:14,0",
        ]
    );
}

fn options(extra: serde_json::Value) -> SliceOptions {
    let mut value = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 1,
        "line_width": 0.4,
        "sparse_infill_density": 0,
        "skirt_loops": 0,
        "brim_width": 0.0,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    });
    for (key, value_extra) in extra.as_object().unwrap() {
        value[key] = value_extra.clone();
    }
    serde_json::from_value(value).unwrap()
}

fn overhang_print_moves(gcode: &str) -> Vec<&str> {
    gcode
        .lines()
        .filter(|line| line.starts_with(";MOVE:print:overhang_perimeter:"))
        .collect()
}

fn layer_internal_print_moves(gcode: &str, layer_id: usize) -> Vec<&str> {
    let mut current_layer = None;
    let mut moves = Vec::new();
    for line in gcode.lines() {
        if let Some(id) = line
            .strip_prefix(";LAYER:")
            .and_then(|id| id.parse::<usize>().ok())
        {
            current_layer = Some(id);
        }
        if current_layer == Some(layer_id) && line.starts_with(";MOVE:print:internal_perimeter:") {
            moves.push(line);
        }
    }
    moves
}

fn layer_external_print_moves(gcode: &str, layer_id: usize) -> Vec<&str> {
    let mut current_layer = None;
    let mut moves = Vec::new();
    for line in gcode.lines() {
        if let Some(id) = line
            .strip_prefix(";LAYER:")
            .and_then(|id| id.parse::<usize>().ok())
        {
            current_layer = Some(id);
        }
        if current_layer == Some(layer_id) && line.starts_with(";MOVE:print:external_perimeter:") {
            moves.push(line);
        }
    }
    moves
}
