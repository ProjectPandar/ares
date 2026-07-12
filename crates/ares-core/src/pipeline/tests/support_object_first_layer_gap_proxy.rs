use crate::{
    Contour, LayerContours, LayerPrintPaths, Point2, PrintPath, PrintPathRole, SliceError,
    SliceOptions,
};
use serde_json::{Value, json};

#[test]
fn default_first_layer_gap_overrides_default_object_xy_distance_on_layer_zero() {
    let finalized = contour_finalize(
        0,
        0.2,
        vec![support_rectangle(PrintPathRole::SupportMaterial)],
        json!({ "support_remove_small_overhang": false }),
        vec![rect_contour(1.0, 1.0, 3.0, 3.0)],
    );

    assert!(
        finalized[0]
            .paths()
            .iter()
            .any(|path| path.points() == [Point2::new(0.0, 3.2), Point2::new(4.0, 3.2)])
    );
    assert!(
        !finalized[0]
            .paths()
            .iter()
            .any(|path| path.points() == [Point2::new(0.0, 3.35), Point2::new(4.0, 3.35)])
    );
}

#[test]
fn zero_first_layer_gap_clips_direct_layer_zero_overlap() {
    let finalized = contour_finalize(
        0,
        0.2,
        vec![support_rectangle(PrintPathRole::SupportMaterial)],
        json!({
            "support_object_first_layer_gap": 0.0,
            "support_remove_small_overhang": false
        }),
        vec![rect_contour(1.0, 1.0, 3.0, 3.0)],
    );

    assert!(
        finalized[0]
            .paths()
            .iter()
            .any(|path| path.points() == [Point2::new(0.0, 3.0), Point2::new(4.0, 3.0)])
    );
}

#[test]
fn larger_first_layer_gap_clips_more_and_can_drop_layer_zero_support() {
    let clipped = contour_finalize(
        0,
        0.2,
        vec![support_rectangle(PrintPathRole::SupportMaterial)],
        json!({
            "support_object_first_layer_gap": 0.75,
            "support_remove_small_overhang": false
        }),
        vec![rect_contour(1.0, 1.0, 3.0, 3.0)],
    );
    let dropped = contour_finalize(
        0,
        0.2,
        vec![support_rectangle(PrintPathRole::SupportMaterial)],
        json!({
            "support_object_first_layer_gap": 2.0,
            "support_remove_small_overhang": false
        }),
        vec![rect_contour(1.0, 1.0, 3.0, 3.0)],
    );

    assert!(
        clipped[0]
            .paths()
            .iter()
            .any(|path| path.points() == [Point2::new(0.0, 3.75), Point2::new(4.0, 3.75)])
    );
    assert!(dropped[0].paths().is_empty());
}

#[test]
fn upper_layers_continue_to_use_object_xy_distance() {
    let finalized = contour_finalize(
        7,
        1.6,
        vec![support_rectangle(PrintPathRole::SupportMaterial)],
        json!({
            "support_object_xy_distance": 0.6,
            "support_object_first_layer_gap": 0.0,
            "support_remove_small_overhang": false
        }),
        vec![rect_contour(1.0, 1.0, 3.0, 3.0)],
    );

    assert!(
        finalized[0]
            .paths()
            .iter()
            .any(|path| path.points() == [Point2::new(0.0, 3.6), Point2::new(4.0, 3.6)])
    );
}

#[test]
fn raft_layer_zero_is_not_clipped_by_first_layer_gap() {
    let finalized = crate::finalize_print_paths_with_layer_contours(
        vec![LayerPrintPaths::new(
            0,
            0.2,
            vec![support_rectangle(PrintPathRole::SupportMaterial)],
        )],
        &options(json!({
            "raft_layers": 1,
            "support_object_first_layer_gap": 2.0
        })),
        &[LayerContours::new(
            0,
            0.2,
            vec![rect_contour(1.0, 1.0, 3.0, 3.0)],
        )],
    )
    .unwrap();

    assert_eq!(
        finalized[0].paths()[0].points(),
        [Point2::new(-3.5, -3.5), Point2::new(7.5, -3.5)]
    );
}

#[test]
fn invalid_first_layer_gap_fails_before_disabled_support_filtering() {
    let err = crate::finalize_print_paths_with_layer_contours(
        vec![LayerPrintPaths::new(
            0,
            0.2,
            vec![support_rectangle(PrintPathRole::SupportMaterial)],
        )],
        &options(json!({
            "enable_support": false,
            "support_object_first_layer_gap": "bad"
        })),
        &[LayerContours::new(
            0,
            0.2,
            vec![rect_contour(1.0, 1.0, 3.0, 3.0)],
        )],
    )
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("support_object_first_layer_gap"));
}

#[test]
fn first_layer_interface_is_clipped_before_spacing_and_ironing() {
    let finalized = contour_finalize(
        0,
        0.2,
        vec![
            support_rectangle(PrintPathRole::SupportMaterialInterface)
                .with_extrusion_role(PrintPathRole::SupportMaterialInterface),
        ],
        json!({
            "support_object_first_layer_gap": 0.5,
            "support_remove_small_overhang": false,
            "support_ironing": true,
            "support_ironing_spacing": 1.0
        }),
        vec![rect_contour(1.0, 1.0, 3.0, 3.0)],
    );

    assert!(finalized[0].paths().iter().any(|path| path.role()
        == PrintPathRole::SupportMaterialInterface
        && path.points()
            == [
                Point2::new(0.0, 3.5),
                Point2::new(4.0, 3.5),
                Point2::new(4.0, 4.0),
                Point2::new(0.0, 4.0),
            ]));
    assert!(
        finalized[0]
            .paths()
            .iter()
            .any(|path| path.role() == PrintPathRole::Ironing
                && path.points() == [Point2::new(0.0, 3.5), Point2::new(4.0, 3.5)])
    );
}

fn contour_finalize(
    layer_id: usize,
    print_z: f64,
    paths: Vec<PrintPath>,
    extra: Value,
    contours: Vec<Contour>,
) -> Vec<LayerPrintPaths> {
    crate::finalize_print_paths_with_layer_contours(
        vec![LayerPrintPaths::new(layer_id, print_z, paths)],
        &options(extra),
        &[LayerContours::new(layer_id, print_z, contours)],
    )
    .unwrap()
}

fn support_rectangle(role: PrintPathRole) -> PrintPath {
    PrintPath::new(
        role,
        vec![
            Point2::new(0.0, 0.0),
            Point2::new(4.0, 0.0),
            Point2::new(4.0, 4.0),
            Point2::new(0.0, 4.0),
        ],
    )
    .unwrap()
    .with_closed(true)
}

fn rect_contour(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Contour {
    Contour::new(vec![
        Point2::new(min_x, min_y),
        Point2::new(max_x, min_y),
        Point2::new(max_x, max_y),
        Point2::new(min_x, max_y),
    ])
}

fn options(extra: Value) -> SliceOptions {
    let mut value = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "top_surface_line_width": 0.4,
        "enable_support": true,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    });
    for (key, value_extra) in extra.as_object().expect("test options must be an object") {
        value[key] = value_extra.clone();
    }
    serde_json::from_value(value).unwrap()
}
