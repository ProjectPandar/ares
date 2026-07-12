use crate::{
    Contour, LayerContours, LayerPrintPaths, Point2, PrintPath, PrintPathRole, SliceError,
    SliceOptions,
};
use serde_json::{Value, json};

#[test]
fn no_context_finalizer_preserves_build_plate_only_support_behavior() {
    let finalized = crate::finalize_print_paths(
        vec![
            empty_layer(0),
            layer(
                1,
                support_rect(PrintPathRole::SupportMaterialInterface, 0.0, 0.0, 4.0, 4.0),
            ),
        ],
        &options(json!({
            "support_on_build_plate_only": true,
            "support_ironing": true
        })),
    )
    .unwrap();

    assert!(contains_support_rect(&finalized[1], 0.0, 0.0, 4.0, 4.0));
}

#[test]
fn disabled_build_plate_only_preserves_floating_upper_support() {
    let finalized = contour_finalize(
        vec![
            empty_layer(0),
            layer(
                1,
                support_rect(PrintPathRole::SupportMaterialInterface, 0.0, 0.0, 4.0, 4.0),
            ),
        ],
        json!({
            "support_on_build_plate_only": false,
            "support_ironing": true
        }),
        empty_contours(2),
    );

    assert!(contains_support_rect(&finalized[1], 0.0, 0.0, 4.0, 4.0));
}

#[test]
fn build_plate_only_drops_floating_upper_support() {
    let finalized = contour_finalize(
        vec![
            empty_layer(0),
            layer(
                1,
                support_rect(PrintPathRole::SupportMaterialInterface, 0.0, 0.0, 4.0, 4.0),
            ),
        ],
        json!({
            "support_on_build_plate_only": true,
            "support_remove_small_overhang": false,
            "support_ironing": true
        }),
        empty_contours(2),
    );

    assert!(finalized[1].paths().is_empty());
}

#[test]
fn build_plate_only_retains_layer_zero_and_overlapping_upper_support() {
    let finalized = contour_finalize(
        vec![
            layer(
                0,
                support_rect(PrintPathRole::SupportMaterialInterface, 0.0, 0.0, 4.0, 4.0),
            ),
            layer(
                1,
                support_rect(PrintPathRole::SupportMaterialInterface, 1.0, 1.0, 3.0, 3.0),
            ),
        ],
        json!({
            "support_on_build_plate_only": true,
            "support_remove_small_overhang": false,
            "support_ironing": true
        }),
        empty_contours(2),
    );

    assert!(contains_support_rect(&finalized[0], 0.0, 0.0, 4.0, 4.0));
    assert!(contains_support_rect(&finalized[1], 1.0, 1.0, 3.0, 3.0));
}

#[test]
fn build_plate_only_requires_positive_area_overlap() {
    let finalized = contour_finalize(
        vec![
            layer(
                0,
                support_rect(PrintPathRole::SupportMaterialInterface, 0.0, 0.0, 2.0, 2.0),
            ),
            layer_paths(
                1,
                vec![
                    support_rect(PrintPathRole::SupportMaterialInterface, 1.5, 1.5, 2.5, 2.5),
                    support_rect(PrintPathRole::SupportMaterialInterface, 2.0, 0.5, 3.0, 1.5),
                    support_rect(PrintPathRole::SupportMaterialInterface, 2.0, 2.0, 3.0, 3.0),
                ],
            ),
        ],
        json!({
            "support_on_build_plate_only": true,
            "support_remove_small_overhang": false,
            "support_ironing": true
        }),
        empty_contours(2),
    );

    assert!(contains_support_rect(&finalized[1], 1.5, 1.5, 2.5, 2.5));
    assert!(!contains_support_rect(&finalized[1], 2.0, 0.5, 3.0, 1.5));
    assert!(!contains_support_rect(&finalized[1], 2.0, 2.0, 3.0, 3.0));
}

#[test]
fn build_plate_only_requires_immediate_retained_support_ancestry() {
    let finalized = contour_finalize(
        vec![
            layer(
                0,
                support_rect(PrintPathRole::SupportMaterialInterface, 0.0, 0.0, 1.0, 1.0),
            ),
            layer(
                1,
                support_rect(PrintPathRole::SupportMaterialInterface, 3.0, 3.0, 4.0, 4.0),
            ),
            layer(
                2,
                support_rect(PrintPathRole::SupportMaterialInterface, 3.1, 3.1, 3.9, 3.9),
            ),
        ],
        json!({
            "support_on_build_plate_only": true,
            "support_remove_small_overhang": false,
            "support_ironing": true
        }),
        empty_contours(3),
    );

    assert!(contains_support_rect(&finalized[0], 0.0, 0.0, 1.0, 1.0));
    assert!(finalized[1].paths().is_empty());
    assert!(finalized[2].paths().is_empty());
}

#[test]
fn raft_layer_anchors_upper_build_plate_only_support() {
    let finalized = contour_finalize(
        vec![
            layer(
                0,
                support_rect(PrintPathRole::SupportMaterialInterface, 0.0, 0.0, 4.0, 4.0),
            ),
            layer(
                1,
                support_rect(PrintPathRole::SupportMaterialInterface, 1.0, 1.0, 3.0, 3.0),
            ),
        ],
        json!({
            "support_on_build_plate_only": true,
            "raft_layers": 1,
            "support_ironing": true
        }),
        empty_contours(2),
    );

    assert!(
        finalized[0]
            .paths()
            .iter()
            .any(|path| path.role() == PrintPathRole::SupportMaterialInterface)
    );
    assert!(contains_support_rect(&finalized[1], 1.0, 1.0, 3.0, 3.0));
}

#[test]
fn clipped_lower_support_anchors_only_overlapping_upper_pieces() {
    let finalized = contour_finalize(
        vec![
            layer(
                0,
                support_rect(PrintPathRole::SupportMaterialInterface, 0.0, 0.0, 4.0, 4.0),
            ),
            layer_paths(
                1,
                vec![
                    support_rect(PrintPathRole::SupportMaterialInterface, 0.0, 3.2, 4.0, 4.0),
                    support_rect(PrintPathRole::SupportMaterialInterface, 1.2, 1.2, 2.8, 2.8),
                ],
            ),
        ],
        json!({
            "support_on_build_plate_only": true,
            "support_object_first_layer_gap": 0.0,
            "support_object_xy_distance": 0.0,
            "support_remove_small_overhang": false,
            "support_ironing": true
        }),
        vec![
            LayerContours::new(0, 0.2, vec![rect_contour(1.0, 1.0, 3.0, 3.0)]),
            LayerContours::new(1, 0.4, Vec::new()),
        ],
    );

    assert!(contains_support_rect(&finalized[1], 0.0, 3.2, 4.0, 4.0));
    assert!(!contains_support_rect(&finalized[1], 1.2, 1.2, 2.8, 2.8));
}

#[test]
fn filtered_support_interface_is_not_resurrected_by_spacing_or_ironing() {
    let finalized = contour_finalize(
        vec![
            empty_layer(0),
            layer(
                1,
                support_rect(PrintPathRole::SupportMaterialInterface, 0.0, 0.0, 4.0, 4.0)
                    .with_extrusion_role(PrintPathRole::SupportMaterialInterface),
            ),
        ],
        json!({
            "support_on_build_plate_only": true,
            "support_ironing": true,
            "support_ironing_spacing": 0.5
        }),
        empty_contours(2),
    );

    assert!(finalized[1].paths().is_empty());
}

#[test]
fn invalid_build_plate_only_fails_before_disabled_support_filtering() {
    let err = crate::finalize_print_paths_with_layer_contours(
        vec![
            empty_layer(0),
            layer(
                1,
                support_rect(PrintPathRole::SupportMaterial, 0.0, 0.0, 4.0, 4.0),
            ),
        ],
        &options(json!({
            "enable_support": false,
            "support_on_build_plate_only": "true"
        })),
        &empty_contours(2),
    )
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("support_on_build_plate_only"));
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

fn empty_layer(layer_id: usize) -> LayerPrintPaths {
    layer_paths(layer_id, Vec::new())
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
