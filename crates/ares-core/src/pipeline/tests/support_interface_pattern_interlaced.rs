use crate::{LayerPrintPaths, Point2, PrintPath, PrintPathRole, SliceOptions};
use serde_json::{Value, json};

const COORD_EPSILON: f64 = 1e-9;

#[test]
fn rectilinear_interlaced_odd_layer_uses_negative_45_degree_family() {
    let finalized = finalize(
        vec![support_rectangle()],
        json!({ "support_interface_pattern": "rectilinear_interlaced" }),
    );

    assert_interface_lines(
        finalized[0].paths(),
        &[
            [
                Point2::new(1.2727922061357855, 2.0),
                Point2::new(2.272792206135785, 1.0),
            ],
            [
                Point2::new(2.5455844122715705, 2.0),
                Point2::new(3.0, 1.5455844122715707),
            ],
        ],
    );
}

#[test]
fn rectilinear_interlaced_even_layer_uses_positive_45_degree_family() {
    let finalized = finalize_layer(
        8,
        1.8,
        vec![support_rectangle()],
        json!({ "support_interface_pattern": "rectilinear_interlaced" }),
    );

    assert_interface_lines(
        finalized[0].paths(),
        &[
            [
                Point2::new(1.7272077938642145, 1.0),
                Point2::new(2.7272077938642143, 2.0),
            ],
            [
                Point2::new(1.0, 1.5455844122715712),
                Point2::new(1.4544155877284288, 2.0),
            ],
        ],
    );
}

#[test]
fn rectilinear_interlaced_ignores_support_angle_for_no_raft_proxy() {
    let default = finalize(
        vec![support_rectangle()],
        json!({ "support_interface_pattern": "rectilinear_interlaced" }),
    );
    let angled = finalize(
        vec![support_rectangle()],
        json!({
            "support_angle": 90,
            "support_interface_pattern": "rectilinear_interlaced"
        }),
    );

    assert_eq!(default[0].paths(), angled[0].paths());
}

#[test]
fn rectilinear_interlaced_uses_support_interface_spacing_pitch() {
    let finalized = finalize(
        vec![support_rectangle()],
        json!({
            "support_interface_pattern": "rectilinear_interlaced",
            "support_interface_spacing": 0.0
        }),
    );

    assert_eq!(finalized[0].paths().len(), 5);
}

#[test]
fn rectilinear_interlaced_lines_preserve_source_metadata_and_extrusion_role() {
    let source = support_rectangle()
        .with_extrusion_role(PrintPathRole::SupportMaterialInterface)
        .with_effective_layer_height_mm(0.13)
        .with_effective_line_width_mm(Some(0.47))
        .with_unsupported_span_mm(Some(2.5))
        .with_seam_gap_mm(0.07);
    let finalized = finalize(
        vec![source],
        json!({ "support_interface_pattern": "rectilinear_interlaced" }),
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
        assert!(!path.is_closed());
    }
}

#[test]
fn rectilinear_interlaced_loop_pattern_prepends_outer_shell() {
    let finalized = finalize(
        vec![support_rectangle()],
        json!({
            "support_interface_loop_pattern": true,
            "support_interface_pattern": "rectilinear_interlaced"
        }),
    );
    let paths = finalized[0].paths();

    assert_eq!(paths.len(), 3);
    assert!(paths[0].is_closed());
    assert_points(
        paths[0].points(),
        &[
            Point2::new(1.0, 1.0),
            Point2::new(3.0, 1.0),
            Point2::new(3.0, 2.0),
            Point2::new(1.0, 2.0),
            Point2::new(1.0, 1.0),
        ],
    );
    assert_interface_lines(
        &paths[1..],
        &[
            [
                Point2::new(1.2727922061357855, 2.0),
                Point2::new(2.272792206135785, 1.0),
            ],
            [
                Point2::new(2.5455844122715705, 2.0),
                Point2::new(3.0, 1.5455844122715707),
            ],
        ],
    );
}

fn finalize(paths: Vec<PrintPath>, extra: Value) -> Vec<LayerPrintPaths> {
    finalize_layer(7, 1.6, paths, extra)
}

fn finalize_layer(
    layer_id: usize,
    print_z: f64,
    paths: Vec<PrintPath>,
    extra: Value,
) -> Vec<LayerPrintPaths> {
    crate::finalize_print_paths(
        vec![LayerPrintPaths::new(layer_id, print_z, paths)],
        &options(extra),
    )
    .unwrap()
}

fn support_rectangle() -> PrintPath {
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

fn assert_interface_lines(paths: &[PrintPath], expected: &[[Point2; 2]]) {
    assert_eq!(paths.len(), expected.len());
    for (path, points) in paths.iter().zip(expected) {
        assert_eq!(path.role(), PrintPathRole::SupportMaterialInterface);
        assert_points(path.points(), points);
        assert!(!path.is_closed());
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
