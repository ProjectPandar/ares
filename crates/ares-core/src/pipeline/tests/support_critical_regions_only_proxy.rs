use crate::{
    LayerContours, LayerPrintPaths, Point2, PrintPath, PrintPathRole, SliceError, SliceOptions,
};
use serde_json::{Value, json};

#[test]
fn no_context_finalizer_preserves_support_with_default_critical_regions_only() {
    let finalized = crate::finalize_print_paths(
        vec![layer(
            0,
            support_rect(PrintPathRole::SupportMaterialInterface, 0.0, 0.0, 3.0, 3.0),
        )],
        &options(json!({ "support_ironing": true })),
    )
    .unwrap();

    assert!(contains_support_rect(&finalized[0], 0.0, 0.0, 3.0, 3.0));
}

#[test]
fn no_context_finalizer_does_not_parse_invalid_support_type() {
    let finalized = crate::finalize_print_paths(
        vec![layer(
            0,
            support_rect(PrintPathRole::SupportMaterialInterface, 0.0, 0.0, 3.0, 3.0),
        )],
        &options(json!({
            "support_critical_regions_only": true,
            "support_type": "invalid",
            "support_ironing": true
        })),
    )
    .unwrap();

    assert!(contains_support_rect(&finalized[0], 0.0, 0.0, 3.0, 3.0));
}

#[test]
fn disabled_critical_regions_only_preserves_tree_auto_support() {
    let finalized = contour_finalize(
        vec![layer(
            0,
            support_rect(PrintPathRole::SupportMaterialInterface, 0.0, 0.0, 3.0, 3.0),
        )],
        json!({
            "support_type": "tree(auto)",
            "support_critical_regions_only": false,
            "support_ironing": true
        }),
        empty_contours(1),
    );

    assert!(contains_support_rect(&finalized[0], 0.0, 0.0, 3.0, 3.0));
}

#[test]
fn tree_auto_critical_regions_only_removes_closed_rectangular_support() {
    let finalized = contour_finalize(
        vec![layer(
            0,
            support_rect(PrintPathRole::SupportMaterialInterface, 0.0, 0.0, 3.0, 3.0),
        )],
        json!({
            "support_type": "tree(auto)",
            "support_critical_regions_only": true
        }),
        empty_contours(1),
    );

    assert!(finalized[0].paths().is_empty());
}

#[test]
fn normal_auto_critical_regions_only_preserves_support() {
    let finalized = support_type_output("normal(auto)");

    assert!(contains_support_rect(&finalized[0], 0.0, 0.0, 3.0, 3.0));
}

#[test]
fn normal_manual_critical_regions_only_preserves_support() {
    let finalized = support_type_output("normal(manual)");

    assert!(contains_support_rect(&finalized[0], 0.0, 0.0, 3.0, 3.0));
}

#[test]
fn tree_manual_critical_regions_only_preserves_support() {
    let finalized = support_type_output("tree(manual)");

    assert!(contains_support_rect(&finalized[0], 0.0, 0.0, 3.0, 3.0));
}

#[test]
fn critical_regions_only_preserves_non_support_open_and_non_rectangular_paths() {
    let open_support = support_rect(PrintPathRole::SupportMaterialInterface, 0.0, 0.0, 3.0, 3.0)
        .with_closed(false);
    let non_rect_support = PrintPath::new(
        PrintPathRole::SupportMaterialInterface,
        vec![
            Point2::new(0.0, 0.0),
            Point2::new(3.0, 0.0),
            Point2::new(1.5, 3.0),
        ],
    )
    .unwrap()
    .with_closed(true);
    let ordinary = support_rect(PrintPathRole::BottomSurface, 0.0, 0.0, 3.0, 3.0);

    let finalized = contour_finalize(
        vec![layer_paths(
            0,
            vec![open_support, non_rect_support, ordinary],
        )],
        json!({
            "support_type": "tree(auto)",
            "support_critical_regions_only": true
        }),
        empty_contours(1),
    );

    assert_eq!(finalized[0].paths().len(), 3);
}

#[test]
fn ordinary_overhang_and_bridge_paths_do_not_retain_tree_auto_support() {
    let finalized = contour_finalize(
        vec![layer_paths(
            0,
            vec![
                support_rect(PrintPathRole::SupportMaterialInterface, 0.0, 0.0, 3.0, 3.0),
                support_rect(PrintPathRole::OverhangPerimeter, 0.0, 0.0, 3.0, 3.0),
                support_rect(PrintPathRole::Bridge, 0.0, 0.0, 3.0, 3.0),
                support_rect(PrintPathRole::InternalBridge, 0.0, 0.0, 3.0, 3.0),
            ],
        )],
        json!({
            "support_type": "tree(auto)",
            "support_critical_regions_only": true
        }),
        empty_contours(1),
    );

    assert!(!contains_role(
        &finalized[0],
        PrintPathRole::SupportMaterialInterface
    ));
    assert!(contains_role(
        &finalized[0],
        PrintPathRole::OverhangPerimeter
    ));
    assert!(contains_role(&finalized[0], PrintPathRole::Bridge));
    assert!(contains_role(&finalized[0], PrintPathRole::InternalBridge));
}

#[test]
fn filtered_support_interface_is_not_resurrected_by_spacing_or_ironing() {
    let finalized = contour_finalize(
        vec![layer(
            0,
            support_rect(PrintPathRole::SupportMaterialInterface, 0.0, 0.0, 3.0, 3.0)
                .with_extrusion_role(PrintPathRole::SupportMaterialInterface),
        )],
        json!({
            "support_type": "tree(auto)",
            "support_critical_regions_only": true,
            "support_ironing": true,
            "support_ironing_spacing": 0.5
        }),
        empty_contours(1),
    );

    assert!(finalized[0].paths().is_empty());
}

#[test]
fn invalid_critical_regions_only_fails_before_disabled_support_filtering() {
    let err = crate::finalize_print_paths_with_layer_contours(
        vec![layer(
            0,
            support_rect(PrintPathRole::SupportMaterial, 0.0, 0.0, 3.0, 3.0),
        )],
        &options(json!({
            "enable_support": false,
            "support_critical_regions_only": null
        })),
        &empty_contours(1),
    )
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("support_critical_regions_only"));
}

fn support_type_output(support_type: &str) -> Vec<LayerPrintPaths> {
    contour_finalize(
        vec![layer(
            0,
            support_rect(PrintPathRole::SupportMaterialInterface, 0.0, 0.0, 3.0, 3.0),
        )],
        json!({
            "support_type": support_type,
            "support_critical_regions_only": true,
            "support_ironing": true
        }),
        empty_contours(1),
    )
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

fn contains_role(layer: &LayerPrintPaths, role: PrintPathRole) -> bool {
    layer.paths().iter().any(|path| path.role() == role)
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
