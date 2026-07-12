use crate::{
    Contour, LayerContours, LayerPrintPaths, Point2, PrintPath, PrintPathRole, SliceError,
    SliceOptions,
};
use serde_json::{Value, json};

#[test]
fn no_context_finalizer_preserves_small_support_with_default_remove_small_overhang() {
    let finalized = crate::finalize_print_paths(
        vec![layer(
            0,
            support_rect(PrintPathRole::SupportMaterialInterface, 0.0, 0.0, 1.0, 1.0),
        )],
        &options(json!({
            "line_width": 0.5,
            "support_ironing": true
        })),
    )
    .unwrap();

    assert!(contains_support_rect(&finalized[0], 0.0, 0.0, 1.0, 1.0));
}

#[test]
fn disabled_remove_small_overhang_preserves_small_support() {
    let finalized = contour_finalize(
        vec![layer(
            0,
            support_rect(PrintPathRole::SupportMaterialInterface, 0.0, 0.0, 1.9, 5.0),
        )],
        json!({
            "line_width": 0.5,
            "support_remove_small_overhang": false,
            "support_ironing": true
        }),
        empty_contours(1),
    );

    assert!(contains_support_rect(&finalized[0], 0.0, 0.0, 1.9, 5.0));
}

#[test]
fn default_remove_small_overhang_drops_narrow_support() {
    let finalized = contour_finalize(
        vec![layer(
            0,
            support_rect(PrintPathRole::SupportMaterialInterface, 0.0, 0.0, 1.9, 5.0),
        )],
        json!({
            "line_width": 0.5,
            "support_ironing": true
        }),
        empty_contours(1),
    );

    assert!(finalized[0].paths().is_empty());
}

#[test]
fn default_remove_small_overhang_drops_short_support() {
    let finalized = contour_finalize(
        vec![layer(
            0,
            support_rect(PrintPathRole::SupportMaterialInterface, 0.0, 0.0, 5.0, 1.9),
        )],
        json!({
            "line_width": 0.5,
            "support_ironing": false
        }),
        empty_contours(1),
    );

    assert!(finalized[0].paths().is_empty());
}

#[test]
fn default_remove_small_overhang_retains_threshold_support() {
    let finalized = contour_finalize(
        vec![layer(
            0,
            support_rect(PrintPathRole::SupportMaterialInterface, 0.0, 0.0, 2.0, 2.0),
        )],
        json!({
            "line_width": 0.5,
            "support_ironing": true
        }),
        empty_contours(1),
    );

    assert!(contains_support_rect(&finalized[0], 0.0, 0.0, 2.0, 2.0));
}

#[test]
fn remove_small_overhang_uses_generic_line_width_not_support_line_width() {
    let finalized = contour_finalize(
        vec![layer(
            0,
            support_rect(PrintPathRole::SupportMaterialInterface, 0.0, 0.0, 1.3, 1.3),
        )],
        json!({
            "line_width": 0.3,
            "support_line_width": 1.0,
            "support_ironing": true
        }),
        empty_contours(1),
    );

    assert!(contains_support_rect(&finalized[0], 0.0, 0.0, 1.3, 1.3));
}

#[test]
fn remove_small_overhang_preserves_non_support_open_and_non_rectangular_paths() {
    let open_support = support_rect(PrintPathRole::SupportMaterialInterface, 0.0, 0.0, 1.0, 1.0)
        .with_closed(false);
    let non_rect_support = PrintPath::new(
        PrintPathRole::SupportMaterialInterface,
        vec![
            Point2::new(0.0, 0.0),
            Point2::new(1.0, 0.0),
            Point2::new(0.5, 1.0),
        ],
    )
    .unwrap()
    .with_closed(true);
    let ordinary = support_rect(PrintPathRole::BottomSurface, 0.0, 0.0, 1.0, 1.0);

    let finalized = contour_finalize(
        vec![layer_paths(
            0,
            vec![open_support, non_rect_support, ordinary],
        )],
        json!({
            "line_width": 0.5,
            "support_ironing": false
        }),
        empty_contours(1),
    );

    assert_eq!(finalized[0].paths().len(), 3);
}

#[test]
fn clipped_support_piece_is_removed_before_spacing_and_ironing() {
    let finalized = contour_finalize(
        vec![layer(
            0,
            support_rect(PrintPathRole::SupportMaterialInterface, 0.0, 0.0, 5.0, 5.0),
        )],
        json!({
            "line_width": 0.5,
            "support_object_first_layer_gap": 0.0,
            "support_object_xy_distance": 0.0,
            "support_ironing": true
        }),
        vec![LayerContours::new(
            0,
            0.2,
            vec![rect_contour(1.5, 0.0, 5.0, 5.0)],
        )],
    );

    assert!(finalized[0].paths().is_empty());
}

#[test]
fn filtered_support_interface_is_not_resurrected_by_spacing_or_ironing() {
    let finalized = contour_finalize(
        vec![layer(
            0,
            support_rect(PrintPathRole::SupportMaterialInterface, 0.0, 0.0, 1.0, 1.0)
                .with_extrusion_role(PrintPathRole::SupportMaterialInterface),
        )],
        json!({
            "line_width": 0.5,
            "support_ironing": true,
            "support_ironing_spacing": 0.5
        }),
        empty_contours(1),
    );

    assert!(finalized[0].paths().is_empty());
}

#[test]
fn invalid_remove_small_overhang_fails_before_disabled_support_filtering() {
    let err = crate::finalize_print_paths_with_layer_contours(
        vec![layer(
            0,
            support_rect(PrintPathRole::SupportMaterial, 0.0, 0.0, 1.0, 1.0),
        )],
        &options(json!({
            "enable_support": false,
            "support_remove_small_overhang": null
        })),
        &empty_contours(1),
    )
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("support_remove_small_overhang"));
}

fn contour_finalize(
    layers: Vec<LayerPrintPaths>,
    extra: Value,
    contours: Vec<LayerContours>,
) -> Vec<LayerPrintPaths> {
    crate::finalize_print_paths_with_layer_contours(layers, &options(extra), &contours).unwrap()
}

fn layer(layer_id: usize, path: PrintPath) -> LayerPrintPaths {
    layer_paths(layer_id, vec![path])
}

fn layer_paths(layer_id: usize, paths: Vec<PrintPath>) -> LayerPrintPaths {
    LayerPrintPaths::new(layer_id, 0.2 * (layer_id + 1) as f64, paths)
}

fn contains_support_rect(
    layer: &LayerPrintPaths,
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
) -> bool {
    layer.paths().iter().any(|path| {
        path.role() == PrintPathRole::SupportMaterialInterface
            && path.is_closed()
            && path.points() == rect_points(min_x, min_y, max_x, max_y)
    })
}

fn empty_contours(count: usize) -> Vec<LayerContours> {
    (0..count)
        .map(|layer_id| LayerContours::new(layer_id, 0.2 * (layer_id + 1) as f64, Vec::new()))
        .collect()
}

fn support_rect(role: PrintPathRole, min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> PrintPath {
    PrintPath::new(role, rect_points(min_x, min_y, max_x, max_y).to_vec())
        .unwrap()
        .with_closed(true)
}

fn rect_points(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> [Point2; 4] {
    [
        Point2::new(min_x, min_y),
        Point2::new(max_x, min_y),
        Point2::new(max_x, max_y),
        Point2::new(min_x, max_y),
    ]
}

fn rect_contour(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Contour {
    Contour::new(rect_points(min_x, min_y, max_x, max_y).to_vec())
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
