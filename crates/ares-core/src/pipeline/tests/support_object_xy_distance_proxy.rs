use crate::{
    Contour, LayerContours, LayerPrintPaths, Point2, PrintPath, PrintPathRole, SliceError,
    SliceOptions,
};
use serde_json::{Value, json};

#[test]
fn no_context_finalizer_preserves_existing_support_behavior() {
    let finalized = crate::finalize_print_paths(
        vec![LayerPrintPaths::new(
            7,
            1.6,
            vec![support_rectangle(PrintPathRole::SupportMaterial)],
        )],
        &options(json!({ "support_object_xy_distance": 10.0 })),
    )
    .unwrap();

    assert_eq!(
        finalized[0].paths()[0].points(),
        [Point2::new(0.0, 0.0), Point2::new(4.0, 0.0)]
    );
}

#[test]
fn default_distance_clips_support_material_around_same_layer_object_contour() {
    let finalized = contour_finalize(
        vec![support_rectangle(PrintPathRole::SupportMaterial)],
        json!({ "support_remove_small_overhang": false }),
        vec![rect_contour(1.0, 1.0, 3.0, 3.0)],
    );

    assert!(
        finalized[0]
            .paths()
            .iter()
            .any(|path| path.points() == [Point2::new(0.0, 0.0), Point2::new(4.0, 0.0)])
    );
    assert!(
        finalized[0]
            .paths()
            .iter()
            .any(|path| path.points() == [Point2::new(0.0, 3.35), Point2::new(4.0, 3.35)])
    );
    assert!(
        finalized[0]
            .paths()
            .iter()
            .all(|path| !path.points().contains(&Point2::new(0.0, 1.0)))
    );
}

#[test]
fn zero_distance_clips_direct_overlap_without_extra_clearance() {
    let finalized = contour_finalize(
        vec![support_rectangle(PrintPathRole::SupportMaterial)],
        json!({
            "support_object_xy_distance": 0.0,
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
    assert!(
        !finalized[0]
            .paths()
            .iter()
            .any(|path| path.points() == [Point2::new(0.0, 3.35), Point2::new(4.0, 3.35)])
    );
}

#[test]
fn larger_distance_clips_more_and_full_coverage_drops_support_rectangle() {
    let clipped = contour_finalize(
        vec![support_rectangle(PrintPathRole::SupportMaterial)],
        json!({
            "support_object_xy_distance": 0.75,
            "support_remove_small_overhang": false
        }),
        vec![rect_contour(1.0, 1.0, 3.0, 3.0)],
    );
    let dropped = contour_finalize(
        vec![support_rectangle(PrintPathRole::SupportMaterial)],
        json!({
            "support_object_xy_distance": 2.0,
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
fn support_interface_is_clipped_before_spacing_and_ironing() {
    let finalized = contour_finalize(
        vec![
            support_rectangle(PrintPathRole::SupportMaterialInterface)
                .with_extrusion_role(PrintPathRole::SupportMaterialInterface),
        ],
        json!({
            "support_object_xy_distance": 0.5,
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

#[test]
fn clipped_paths_preserve_print_path_metadata() {
    let source = support_rectangle(PrintPathRole::SupportMaterial)
        .with_effective_layer_height_mm(0.18)
        .with_effective_line_width_mm(Some(0.62))
        .with_unsupported_span_mm(Some(1.25))
        .with_seam_gap_mm(0.04);
    let finalized = contour_finalize(
        vec![source],
        json!({
            "support_object_xy_distance": 0.5,
            "support_remove_small_overhang": false
        }),
        vec![rect_contour(1.0, 1.0, 3.0, 3.0)],
    );
    let clipped = finalized[0]
        .paths()
        .iter()
        .find(|path| path.points() == [Point2::new(0.0, 3.5), Point2::new(4.0, 3.5)])
        .expect("expected top clipped support line");

    assert_eq!(clipped.effective_layer_height_mm(), Some(0.18));
    assert_eq!(clipped.effective_line_width_mm(), Some(0.62));
    assert_eq!(clipped.unsupported_span_mm(), Some(1.25));
    assert_eq!(clipped.seam_gap_mm(), 0.04);
}

#[test]
fn non_rectangular_open_non_support_and_non_rectangular_contours_are_unchanged() {
    let triangle = PrintPath::new(
        PrintPathRole::SupportMaterial,
        vec![
            Point2::new(0.0, 0.0),
            Point2::new(4.0, 0.0),
            Point2::new(2.0, 4.0),
        ],
    )
    .unwrap()
    .with_closed(true);
    let open = PrintPath::new(PrintPathRole::SupportMaterial, rectangle_points()).unwrap();
    let solid = PrintPath::new(PrintPathRole::SolidInfill, rectangle_points())
        .unwrap()
        .with_closed(true);
    let finalized = contour_finalize(
        vec![triangle.clone(), open.clone(), solid.clone()],
        json!({}),
        vec![Contour::new(vec![
            Point2::new(1.0, 1.0),
            Point2::new(3.0, 1.0),
            Point2::new(2.0, 3.0),
        ])],
    );

    assert_eq!(finalized[0].paths(), [triangle, open, solid]);
}

#[test]
fn raft_layers_are_not_clipped_by_object_xy_distance() {
    let finalized = crate::finalize_print_paths_with_layer_contours(
        vec![LayerPrintPaths::new(
            0,
            0.2,
            vec![support_rectangle(PrintPathRole::SupportMaterial)],
        )],
        &options(json!({ "raft_layers": 1 })),
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
fn invalid_support_object_xy_distance_fails_before_disabled_support_filtering() {
    let err = crate::finalize_print_paths_with_layer_contours(
        vec![LayerPrintPaths::new(
            7,
            1.6,
            vec![support_rectangle(PrintPathRole::SupportMaterial)],
        )],
        &options(json!({
        "enable_support": false,
            "support_object_xy_distance": "bad"
        })),
        &[LayerContours::new(
            7,
            1.6,
            vec![rect_contour(1.0, 1.0, 3.0, 3.0)],
        )],
    )
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("support_object_xy_distance"));
}

fn contour_finalize(
    paths: Vec<PrintPath>,
    extra: Value,
    contours: Vec<Contour>,
) -> Vec<LayerPrintPaths> {
    crate::finalize_print_paths_with_layer_contours(
        vec![LayerPrintPaths::new(7, 1.6, paths)],
        &options(extra),
        &[LayerContours::new(7, 1.6, contours)],
    )
    .unwrap()
}

fn support_rectangle(role: PrintPathRole) -> PrintPath {
    PrintPath::new(role, rectangle_points())
        .unwrap()
        .with_closed(true)
}

fn rectangle_points() -> Vec<Point2> {
    vec![
        Point2::new(0.0, 0.0),
        Point2::new(4.0, 0.0),
        Point2::new(4.0, 4.0),
        Point2::new(0.0, 4.0),
    ]
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
