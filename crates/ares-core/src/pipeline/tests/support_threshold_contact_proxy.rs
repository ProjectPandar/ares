use crate::{
    Contour, LayerContours, LayerPrintPaths, Point2, PrintPath, PrintPathRole, SliceOptions,
};
use serde_json::{Value, json};

#[test]
fn disabled_support_does_not_generate_threshold_contacts() {
    for extra in [json!({}), json!({ "enable_support": false })] {
        let finalized = contour_finalize(Vec::new(), extra);

        assert!(support_interface_paths(&finalized).is_empty());
    }
}

#[test]
fn raft_or_enforced_support_activation_does_not_generate_threshold_contacts() {
    for extra in [
        json!({ "raft_layers": 1 }),
        json!({ "enable_support": false, "raft_layers": 1 }),
        json!({ "enforce_support_layers": 1 }),
        json!({ "enable_support": false, "enforce_support_layers": 1 }),
    ] {
        let existing = support_rectangle(PrintPathRole::SupportMaterial);
        let finalized = contour_finalize(vec![existing], extra);

        assert_eq!(support_interface_paths(&finalized).len(), 0);
        assert!(!support_material_paths(&finalized).is_empty());
    }
}

#[test]
fn normal_auto_default_threshold_generates_contact_below_unsupported_rectangle() {
    let finalized = contour_finalize(
        Vec::new(),
        json!({
            "enable_support": true,
            "support_object_xy_distance": 0.0
        }),
    );

    assert_has_interface_rect(&finalized[0], 10.0, 0.0, 14.0, 4.0);
    assert!(finalized[1].paths().is_empty());
}

#[test]
fn manual_and_tree_support_types_do_not_generate_threshold_contacts() {
    for support_type in ["normal(manual)", "tree(auto)", "tree(manual)"] {
        let finalized = contour_finalize(
            Vec::new(),
            json!({
                "enable_support": true,
                "support_type": support_type,
                "support_object_xy_distance": 0.0
            }),
        );

        assert!(support_interface_paths(&finalized).is_empty());
    }
}

#[test]
fn low_angle_suppresses_shallow_partial_overhang() {
    let default = partial_contour_finalize(json!({
        "enable_support": true,
        "support_object_xy_distance": 0.0
    }));
    let low = partial_contour_finalize(json!({
        "enable_support": true,
        "support_threshold_angle": 1,
        "support_object_xy_distance": 0.0
    }));

    assert_has_interface_rect(&default[0], 4.0, 0.0, 7.0, 4.0);
    assert!(support_interface_paths(&low).is_empty());
}

#[test]
fn high_angle_and_default_restore_full_overhang_after_expand_back() {
    for extra in [
        json!({
            "enable_support": true,
            "support_object_xy_distance": 0.0
        }),
        json!({
            "enable_support": true,
            "support_threshold_angle": 90,
            "support_object_xy_distance": 0.0
        }),
    ] {
        let finalized = partial_contour_finalize(extra);

        assert_has_interface_rect(&finalized[0], 4.0, 0.0, 7.0, 4.0);
        assert_eq!(support_interface_paths(&finalized).len(), 1);
    }
}

#[test]
fn zero_angle_uses_threshold_overlap() {
    let absolute = partial_contour_finalize(json!({
        "enable_support": true,
        "support_threshold_angle": 0,
        "support_threshold_overlap": 1.0,
        "outer_wall_line_width": 0.4,
        "support_object_xy_distance": 0.0
    }));
    let percent = partial_contour_finalize(json!({
        "enable_support": true,
        "support_threshold_angle": 0,
        "support_threshold_overlap": "50%",
        "outer_wall_line_width": 0.8,
        "support_object_xy_distance": 0.0
    }));

    assert_has_interface_rect(&absolute[0], 4.0, 1.2, 6.4, 2.8);
    assert_has_interface_rect(&percent[0], 4.0, 0.0, 7.0, 4.0);
}

#[test]
fn zero_angle_percent_overlap_uses_external_width_not_support_width() {
    let percent = partial_contour_finalize(json!({
        "enable_support": true,
        "support_threshold_angle": 0,
        "support_threshold_overlap": "50%",
        "line_width": 0.4,
        "outer_wall_line_width": 0.8,
        "support_line_width": 4.0,
        "support_object_xy_distance": 0.0
    }));
    let absolute_external = partial_contour_finalize(json!({
        "enable_support": true,
        "support_threshold_angle": 0,
        "support_threshold_overlap": 0.4,
        "line_width": 0.4,
        "outer_wall_line_width": 0.8,
        "support_line_width": 4.0,
        "support_object_xy_distance": 0.0
    }));

    assert_eq!(
        support_interface_paths(&percent),
        support_interface_paths(&absolute_external)
    );
    assert_has_interface_rect(&percent[0], 4.0, 0.0, 7.0, 4.0);
}

#[test]
fn positive_angle_uses_previous_lower_layer_height() {
    let finalized = crate::finalize_print_paths_with_layer_contours(
        three_empty_layers(),
        &options(json!({
            "enable_support": true,
            "support_threshold_angle": 30,
            "support_object_xy_distance": 0.0
        })),
        &[
            LayerContours::new(0, 0.2, vec![rect_contour(0.0, 0.0, 4.0, 4.0)]),
            LayerContours::new(1, 1.2, vec![rect_contour(3.0, 0.0, 7.0, 4.0)]),
            LayerContours::new(2, 1.4, vec![rect_contour(3.6, 0.0, 7.6, 4.0)]),
        ],
    )
    .unwrap();

    assert_has_interface_rect(&finalized[0], 4.0, 0.0, 7.0, 4.0);
    assert!(finalized[1].paths().is_empty());
}

#[test]
fn generated_contacts_are_trimmed_by_support_object_xy_distance() {
    let finalized = partial_contour_finalize(json!({
        "enable_support": true,
        "support_object_first_layer_gap": 0.5
    }));

    assert_has_interface_rect(&finalized[0], 4.5, 0.0, 7.0, 4.0);
    assert!(!has_interface_rect(&finalized[0], 4.0, 0.0, 7.0, 4.0));
}

#[test]
fn existing_support_proxy_paths_are_preserved() {
    let existing = support_rectangle(PrintPathRole::SupportMaterial);
    let finalized = contour_finalize(
        vec![existing.clone()],
        json!({
            "enable_support": true,
            "support_object_xy_distance": 0.0
        }),
    );

    assert_has_interface_rect(&finalized[0], 10.0, 0.0, 14.0, 4.0);
    let preserved_support = support_material_paths(&finalized);
    assert!(!preserved_support.is_empty());
    assert!(
        preserved_support.iter().flatten().all(|point| {
            (20.0..=24.0).contains(&point.x()) && (0.0..=4.0).contains(&point.y())
        })
    );
}

#[test]
fn disabled_interface_layer_settings_downgrade_generated_contacts() {
    let finalized = partial_contour_finalize(json!({
        "enable_support": true,
        "support_interface_top_layers": 0,
        "support_interface_bottom_layers": 0,
        "support_object_xy_distance": 0.0
    }));

    assert!(support_interface_paths(&finalized).is_empty());
    assert!(!support_material_paths(&finalized).is_empty());
}

#[test]
fn expand_back_contacts_are_disjoint_around_interior_previous_rectangle() {
    let finalized = crate::finalize_print_paths_with_layer_contours(
        vec![
            LayerPrintPaths::new(0, 0.2, Vec::new()),
            LayerPrintPaths::new(1, 0.4, Vec::new()),
        ],
        &options(json!({
            "enable_support": true,
            "support_object_xy_distance": 0.0
        })),
        &[
            LayerContours::new(0, 0.2, vec![rect_contour(2.0, 2.0, 6.0, 6.0)]),
            LayerContours::new(1, 0.4, vec![rect_contour(0.0, 0.0, 8.0, 8.0)]),
        ],
    )
    .unwrap();
    let rects = support_interface_rects(&finalized[0]);

    assert_eq!(rects.len(), 4);
    for left in 0..rects.len() {
        for right in left + 1..rects.len() {
            assert!(!rects[left].overlaps(rects[right]));
        }
    }
}

fn contour_finalize(paths: Vec<PrintPath>, extra: Value) -> Vec<LayerPrintPaths> {
    crate::finalize_print_paths_with_layer_contours(
        vec![
            LayerPrintPaths::new(0, 0.2, paths),
            LayerPrintPaths::new(1, 0.4, Vec::new()),
        ],
        &options(extra),
        &[
            LayerContours::new(0, 0.2, vec![rect_contour(0.0, 0.0, 4.0, 4.0)]),
            LayerContours::new(1, 0.4, vec![rect_contour(10.0, 0.0, 14.0, 4.0)]),
        ],
    )
    .unwrap()
}

fn partial_contour_finalize(extra: Value) -> Vec<LayerPrintPaths> {
    crate::finalize_print_paths_with_layer_contours(
        vec![
            LayerPrintPaths::new(0, 0.2, Vec::new()),
            LayerPrintPaths::new(1, 0.4, Vec::new()),
        ],
        &options(extra),
        &[
            LayerContours::new(0, 0.2, vec![rect_contour(0.0, 0.0, 4.0, 4.0)]),
            LayerContours::new(1, 0.4, vec![rect_contour(3.0, 0.0, 7.0, 4.0)]),
        ],
    )
    .unwrap()
}

fn three_empty_layers() -> Vec<LayerPrintPaths> {
    vec![
        LayerPrintPaths::new(0, 0.2, Vec::new()),
        LayerPrintPaths::new(1, 1.2, Vec::new()),
        LayerPrintPaths::new(2, 1.4, Vec::new()),
    ]
}

fn support_interface_paths(layers: &[LayerPrintPaths]) -> Vec<Vec<Point2>> {
    layers
        .iter()
        .flat_map(|layer| layer.paths())
        .filter(|path| path.role() == PrintPathRole::SupportMaterialInterface)
        .map(|path| path.points().to_vec())
        .collect()
}

fn support_material_paths(layers: &[LayerPrintPaths]) -> Vec<Vec<Point2>> {
    layers
        .iter()
        .flat_map(|layer| layer.paths())
        .filter(|path| path.role() == PrintPathRole::SupportMaterial)
        .map(|path| path.points().to_vec())
        .collect()
}

#[derive(Clone, Copy)]
struct TestRect {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}

impl TestRect {
    fn overlaps(self, other: Self) -> bool {
        self.max_x.min(other.max_x) - self.min_x.max(other.min_x) > 1e-9
            && self.max_y.min(other.max_y) - self.min_y.max(other.min_y) > 1e-9
    }
}

fn support_interface_rects(layer: &LayerPrintPaths) -> Vec<TestRect> {
    layer
        .paths()
        .iter()
        .filter(|path| path.role() == PrintPathRole::SupportMaterialInterface)
        .map(|path| {
            path.points().iter().fold(
                TestRect {
                    min_x: f64::INFINITY,
                    min_y: f64::INFINITY,
                    max_x: f64::NEG_INFINITY,
                    max_y: f64::NEG_INFINITY,
                },
                |bounds, point| TestRect {
                    min_x: bounds.min_x.min(point.x()),
                    min_y: bounds.min_y.min(point.y()),
                    max_x: bounds.max_x.max(point.x()),
                    max_y: bounds.max_y.max(point.y()),
                },
            )
        })
        .collect()
}

fn assert_has_interface_rect(
    layer: &LayerPrintPaths,
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
) {
    assert!(
        has_interface_rect(layer, min_x, min_y, max_x, max_y),
        "missing interface rect ({min_x}, {min_y})-({max_x}, {max_y}); paths: {:?}",
        support_interface_paths(std::slice::from_ref(layer))
    );
}

fn has_interface_rect(
    layer: &LayerPrintPaths,
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
) -> bool {
    let points = rectangle_points(min_x, min_y, max_x, max_y);
    layer.paths().iter().any(|path| {
        path.role() == PrintPathRole::SupportMaterialInterface && path.points() == points
    })
}

fn support_rectangle(role: PrintPathRole) -> PrintPath {
    PrintPath::new(role, rectangle_points(20.0, 0.0, 24.0, 4.0))
        .unwrap()
        .with_closed(true)
}

fn rectangle_points(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Vec<Point2> {
    vec![
        Point2::new(min_x, min_y),
        Point2::new(max_x, min_y),
        Point2::new(max_x, max_y),
        Point2::new(min_x, max_y),
    ]
}

fn rect_contour(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Contour {
    Contour::new(rectangle_points(min_x, min_y, max_x, max_y))
}

fn options(extra: Value) -> SliceOptions {
    let mut value = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "top_surface_line_width": 0.4,
        "enable_support": false,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false,
        "support_remove_small_overhang": false,
        "support_object_first_layer_gap": 0.0,
        "support_ironing": true
    });
    for (key, value_extra) in extra.as_object().expect("test options must be an object") {
        value[key] = value_extra.clone();
    }
    serde_json::from_value(value).unwrap()
}
