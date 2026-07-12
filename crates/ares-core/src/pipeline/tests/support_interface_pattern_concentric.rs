use crate::{LayerPrintPaths, Point2, PrintPath, PrintPathRole, SliceOptions};
use serde_json::{Value, json};

const COORD_EPSILON: f64 = 1e-9;

#[test]
fn concentric_generates_outermost_first_closed_interface_loops() {
    let finalized = finalize(
        vec![support_rectangle()],
        json!({ "support_interface_pattern": "concentric" }),
    );

    assert_interface_loops(
        finalized[0].paths(),
        &[
            [
                Point2::new(0.0, 0.0),
                Point2::new(4.0, 0.0),
                Point2::new(4.0, 3.0),
                Point2::new(0.0, 3.0),
                Point2::new(0.0, 0.0),
            ],
            [
                Point2::new(0.9, 0.9),
                Point2::new(3.1, 0.9),
                Point2::new(3.1, 2.1),
                Point2::new(0.9, 2.1),
                Point2::new(0.9, 0.9),
            ],
        ],
    );
}

#[test]
fn concentric_uses_support_interface_spacing_pitch() {
    let default_spacing = finalize(
        vec![support_rectangle()],
        json!({ "support_interface_pattern": "concentric" }),
    );
    let zero_spacing = finalize(
        vec![support_rectangle()],
        json!({
            "support_interface_pattern": "concentric",
            "support_interface_spacing": 0.0
        }),
    );

    assert_eq!(default_spacing[0].paths().len(), 2);
    assert_eq!(zero_spacing[0].paths().len(), 4);
    assert_interface_loops(
        zero_spacing[0].paths(),
        &[
            [
                Point2::new(0.0, 0.0),
                Point2::new(4.0, 0.0),
                Point2::new(4.0, 3.0),
                Point2::new(0.0, 3.0),
                Point2::new(0.0, 0.0),
            ],
            [
                Point2::new(0.4, 0.4),
                Point2::new(3.6, 0.4),
                Point2::new(3.6, 2.6),
                Point2::new(0.4, 2.6),
                Point2::new(0.4, 0.4),
            ],
            [
                Point2::new(0.8, 0.8),
                Point2::new(3.2, 0.8),
                Point2::new(3.2, 2.2),
                Point2::new(0.8, 2.2),
                Point2::new(0.8, 0.8),
            ],
            [
                Point2::new(1.2, 1.2),
                Point2::new(2.8, 1.2),
                Point2::new(2.8, 1.8),
                Point2::new(1.2, 1.8),
                Point2::new(1.2, 1.2),
            ],
        ],
    );
}

#[test]
fn concentric_stops_before_collapsed_loop_geometry() {
    let narrow = PrintPath::new(
        PrintPathRole::SupportMaterialInterface,
        vec![
            Point2::new(1.0, 1.0),
            Point2::new(1.6, 1.0),
            Point2::new(1.6, 1.2),
            Point2::new(1.0, 1.2),
        ],
    )
    .unwrap()
    .with_closed(true);
    let finalized = finalize(
        vec![narrow],
        json!({
            "support_interface_pattern": "concentric",
            "support_interface_spacing": 0.0
        }),
    );

    assert_interface_loops(
        finalized[0].paths(),
        &[[
            Point2::new(1.0, 1.0),
            Point2::new(1.6, 1.0),
            Point2::new(1.6, 1.2),
            Point2::new(1.0, 1.2),
            Point2::new(1.0, 1.0),
        ]],
    );
}

#[test]
fn concentric_preserves_source_metadata_and_extrusion_role() {
    let source = support_rectangle()
        .with_extrusion_role(PrintPathRole::SupportMaterialInterface)
        .with_effective_layer_height_mm(0.13)
        .with_effective_line_width_mm(Some(0.47))
        .with_unsupported_span_mm(Some(2.5))
        .with_seam_gap_mm(0.07);
    let finalized = finalize(
        vec![source],
        json!({ "support_interface_pattern": "concentric" }),
    );

    assert_eq!(finalized[0].layer_id(), 7);
    assert_eq!(finalized[0].print_z(), 1.6);
    for path in finalized[0].paths() {
        assert_eq!(path.role(), PrintPathRole::SupportMaterialInterface);
        assert_eq!(
            path.extrusion_role(),
            Some(PrintPathRole::SupportMaterialInterface)
        );
        assert_eq!(path.effective_layer_height_mm(), Some(0.13));
        assert_eq!(path.effective_line_width_mm(), Some(0.47));
        assert_eq!(path.unsupported_span_mm(), Some(2.5));
        assert_eq!(path.seam_gap_mm(), 0.07);
        assert!(path.is_closed());
    }
}

#[test]
fn concentric_does_not_prepend_extra_loop_pattern_shell() {
    let without_loop_pattern = finalize(
        vec![small_support_rectangle()],
        json!({ "support_interface_pattern": "concentric" }),
    );
    let with_loop_pattern = finalize(
        vec![small_support_rectangle()],
        json!({
            "support_interface_pattern": "concentric",
            "support_interface_loop_pattern": true
        }),
    );

    assert_eq!(with_loop_pattern, without_loop_pattern);
}

fn finalize(paths: Vec<PrintPath>, extra: Value) -> Vec<LayerPrintPaths> {
    crate::finalize_print_paths(vec![LayerPrintPaths::new(7, 1.6, paths)], &options(extra)).unwrap()
}

fn support_rectangle() -> PrintPath {
    PrintPath::new(
        PrintPathRole::SupportMaterialInterface,
        vec![
            Point2::new(0.0, 0.0),
            Point2::new(4.0, 0.0),
            Point2::new(4.0, 3.0),
            Point2::new(0.0, 3.0),
        ],
    )
    .unwrap()
    .with_closed(true)
}

fn small_support_rectangle() -> PrintPath {
    PrintPath::new(
        PrintPathRole::SupportMaterialInterface,
        vec![
            Point2::new(1.0, 1.0),
            Point2::new(3.0, 1.0),
            Point2::new(3.0, 2.0),
            Point2::new(1.0, 2.0),
        ],
    )
    .unwrap()
    .with_closed(true)
}

fn assert_interface_loops(paths: &[PrintPath], expected: &[[Point2; 5]]) {
    assert_eq!(paths.len(), expected.len());
    for (path, points) in paths.iter().zip(expected) {
        assert_eq!(path.role(), PrintPathRole::SupportMaterialInterface);
        assert_points(path.points(), points);
        assert!(path.is_closed());
    }
}

fn assert_points(actual: &[Point2], expected: &[Point2]) {
    assert_eq!(actual.len(), expected.len());
    for (actual_point, expected_point) in actual.iter().zip(expected) {
        assert!(
            (actual_point.x() - expected_point.x()).abs() <= COORD_EPSILON,
            "x mismatch: actual {actual_point:?}, expected {expected_point:?}"
        );
        assert!(
            (actual_point.y() - expected_point.y()).abs() <= COORD_EPSILON,
            "y mismatch: actual {actual_point:?}, expected {expected_point:?}"
        );
    }
}

fn options(extra: Value) -> SliceOptions {
    let mut value = json!({
        "enable_support": true,
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "top_surface_line_width": 0.4,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    });
    let extra = extra.as_object().expect("test options must be an object");
    for (key, value_extra) in extra {
        value[key] = value_extra.clone();
    }
    serde_json::from_value(value).unwrap()
}
