use crate::{
    Contour, LayerContours, LayerPrintPaths, Point2, PrintPath, PrintPathRole, SliceOptions,
};
use serde_json::{Value, json};

#[test]
fn snug_merges_closed_support_material_rectangles_after_closing_inflation() {
    let default = finalize(
        two_support_rects((0.0, 0.0, 2.0, 2.0), (5.0, 0.0, 7.0, 2.0)),
        json!({ "support_style": "default" }),
    );
    let grid = finalize(
        two_support_rects((0.0, 0.0, 2.0, 2.0), (5.0, 0.0, 7.0, 2.0)),
        json!({ "support_style": "grid" }),
    );
    let snug = finalize(
        two_support_rects((0.0, 0.0, 2.0, 2.0), (5.0, 0.0, 7.0, 2.0)),
        json!({ "support_style": "snug" }),
    );

    assert_support_lines(&default[0], &[(0.0, 0.0, 2.0, 0.0), (5.0, 0.0, 7.0, 0.0)]);
    assert_eq!(grid, default);
    assert_support_lines(&snug[0], &[(0.0, 0.0, 7.0, 0.0)]);
}

#[test]
fn snug_preserves_rectangles_separated_after_closing_inflation() {
    let snug = finalize(
        two_support_rects((0.0, 0.0, 2.0, 2.0), (7.0, 0.0, 9.0, 2.0)),
        json!({ "support_style": "snug" }),
    );

    assert_support_lines(&snug[0], &[(0.0, 0.0, 2.0, 0.0), (7.0, 0.0, 9.0, 0.0)]);
}

#[test]
fn snug_does_not_merge_through_union_bounding_box_gap() {
    let snug = finalize(
        vec![
            support_rect(0.0, 0.0, 2.0, 2.0),
            support_rect(5.0, 5.0, 7.0, 7.0),
            support_rect(8.0, -2.0, 10.0, -0.5),
        ],
        json!({
            "support_style": "snug",
            "raft_first_layer_density": 10
        }),
    );

    assert_support_lines(
        &snug[0],
        &[
            (0.0, 0.0, 7.0, 0.0),
            (0.0, 4.0, 7.0, 4.0),
            (8.0, -2.0, 10.0, -2.0),
        ],
    );
}

#[test]
fn normal_support_tree_style_fallback_preserves_grid_geometry() {
    let default = finalize(
        two_support_rects((0.0, 0.0, 2.0, 2.0), (5.0, 0.0, 7.0, 2.0)),
        json!({ "support_style": "default" }),
    );
    let tree_style = finalize(
        two_support_rects((0.0, 0.0, 2.0, 2.0), (5.0, 0.0, 7.0, 2.0)),
        json!({ "support_style": "tree_slim" }),
    );

    assert_eq!(tree_style, default);
}

#[test]
fn tree_support_type_with_snug_preserves_existing_support_body_geometry() {
    let default = finalize(
        two_support_rects((0.0, 0.0, 2.0, 2.0), (5.0, 0.0, 7.0, 2.0)),
        json!({ "support_style": "default" }),
    );
    let tree = finalize(
        two_support_rects((0.0, 0.0, 2.0, 2.0), (5.0, 0.0, 7.0, 2.0)),
        json!({
            "support_style": "snug",
            "support_type": "tree(auto)"
        }),
    );

    assert_eq!(tree, default);
}

#[test]
fn no_context_snug_merges_before_support_base_spacing() {
    let finalized = finalize(
        two_support_rects((0.0, 0.0, 2.0, 2.0), (5.0, 0.0, 7.0, 2.0)),
        json!({
            "support_style": "snug",
            "support_ironing": true
        }),
    );

    assert_support_lines(&finalized[0], &[(0.0, 0.0, 7.0, 0.0)]);
}

#[test]
fn no_context_non_snug_style_keeps_invalid_support_type_lazy() {
    let finalized = finalize(
        two_support_rects((0.0, 0.0, 2.0, 2.0), (5.0, 0.0, 7.0, 2.0)),
        json!({
            "support_style": "grid",
            "support_type": "invalid"
        }),
    );

    assert_support_lines(&finalized[0], &[(0.0, 0.0, 2.0, 0.0), (5.0, 0.0, 7.0, 0.0)]);
}

#[test]
fn no_context_explicit_snug_parses_support_type() {
    let err = crate::finalize_print_paths(
        vec![layer(
            0,
            two_support_rects((0.0, 0.0, 2.0, 2.0), (5.0, 0.0, 7.0, 2.0)),
        )],
        &options(json!({
            "support_style": "snug",
            "support_type": "invalid"
        })),
    )
    .unwrap_err();

    assert!(err.to_string().contains("support_type"));
}

#[test]
fn snug_does_not_merge_rectangles_across_layers() {
    let finalized = crate::finalize_print_paths(
        vec![
            layer(0, vec![support_rect(0.0, 0.0, 2.0, 2.0)]),
            layer(1, vec![support_rect(5.0, 0.0, 7.0, 2.0)]),
        ],
        &options(json!({
            "support_style": "snug",
            "support_ironing": true
        })),
    )
    .unwrap();

    assert_support_lines(&finalized[0], &[(0.0, 0.0, 2.0, 0.0)]);
    assert_support_lines(&finalized[1], &[(5.0, 0.0, 7.0, 0.0)]);
}

#[test]
fn merged_support_body_preserves_first_source_metadata() {
    let first = support_rect(0.0, 0.0, 2.0, 2.0)
        .with_extrusion_role(PrintPathRole::SupportMaterial)
        .with_effective_layer_height_mm(0.13)
        .with_effective_line_width_mm(Some(0.61))
        .with_unsupported_span_mm(Some(2.5))
        .with_seam_gap_mm(0.07);
    let second = support_rect(5.0, 0.0, 7.0, 2.0)
        .with_effective_layer_height_mm(0.27)
        .with_effective_line_width_mm(Some(0.82))
        .with_unsupported_span_mm(Some(8.0))
        .with_seam_gap_mm(0.19);
    let finalized = finalize(vec![first.clone()], json!({ "support_style": "snug" }));
    assert_support_lines(&finalized[0], &[(0.0, 0.0, 2.0, 0.0)]);

    let merged = finalize(vec![first, second], json!({ "support_style": "snug" }));
    let path = &merged[0].paths()[0];

    assert_support_lines(&merged[0], &[(0.0, 0.0, 7.0, 0.0)]);
    assert_eq!(path.extrusion_role(), Some(PrintPathRole::SupportMaterial));
    assert_eq!(path.effective_layer_height_mm(), Some(0.13));
    assert_eq!(path.effective_line_width_mm(), Some(0.61));
    assert_eq!(path.unsupported_span_mm(), Some(2.5));
    assert_eq!(path.seam_gap_mm(), 0.07);
}

#[test]
fn snug_preserves_interface_open_non_rectangular_and_non_support_paths() {
    let interface = support_path(PrintPathRole::SupportMaterialInterface, 0.0, 0.0, 2.0, 2.0);
    let open = PrintPath::new(
        PrintPathRole::SupportMaterial,
        rectangle_points(5.0, 0.0, 7.0, 2.0),
    )
    .unwrap();
    let triangle = PrintPath::new(
        PrintPathRole::SupportMaterial,
        vec![
            Point2::new(8.0, 0.0),
            Point2::new(10.0, 0.0),
            Point2::new(9.0, 2.0),
        ],
    )
    .unwrap()
    .with_closed(true);
    let solid = support_path(PrintPathRole::SolidInfill, 11.0, 0.0, 13.0, 2.0);
    let finalized = finalize(
        vec![
            interface.clone(),
            open.clone(),
            triangle.clone(),
            solid.clone(),
        ],
        json!({ "support_style": "snug" }),
    );

    for expected in [interface, open, triangle, solid] {
        assert!(finalized[0].paths().contains(&expected));
    }
}

#[test]
fn snug_merged_support_body_is_clipped_by_object_clearance() {
    let finalized = crate::finalize_print_paths_with_layer_contours(
        vec![layer(
            0,
            two_support_rects((0.0, 0.0, 2.0, 2.0), (5.0, 0.0, 7.0, 2.0)),
        )],
        &options(json!({
            "support_style": "snug",
            "support_object_first_layer_gap": 0.5,
            "support_remove_small_overhang": false,
            "support_ironing": true
        })),
        &[LayerContours::new(
            0,
            0.2,
            vec![rect_contour(2.0, 0.5, 4.0, 1.5)],
        )],
    )
    .unwrap();

    assert_support_lines(&finalized[0], &[(0.0, 0.0, 1.5, 0.0), (4.5, 0.0, 7.0, 0.0)]);
}

fn finalize(paths: Vec<PrintPath>, extra: Value) -> Vec<LayerPrintPaths> {
    crate::finalize_print_paths(vec![layer(0, paths)], &options(extra)).unwrap()
}

fn layer(layer_id: usize, paths: Vec<PrintPath>) -> LayerPrintPaths {
    LayerPrintPaths::new(layer_id, 0.2 * (layer_id + 1) as f64, paths)
}

fn two_support_rects(first: (f64, f64, f64, f64), second: (f64, f64, f64, f64)) -> Vec<PrintPath> {
    vec![
        support_rect(first.0, first.1, first.2, first.3),
        support_rect(second.0, second.1, second.2, second.3),
    ]
}

fn support_rect(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> PrintPath {
    support_path(PrintPathRole::SupportMaterial, min_x, min_y, max_x, max_y)
}

fn support_path(role: PrintPathRole, min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> PrintPath {
    PrintPath::new(role, rectangle_points(min_x, min_y, max_x, max_y))
        .unwrap()
        .with_closed(true)
}

fn rect_contour(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Contour {
    Contour::new(rectangle_points(min_x, min_y, max_x, max_y))
}

fn rectangle_points(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Vec<Point2> {
    vec![
        Point2::new(min_x, min_y),
        Point2::new(max_x, min_y),
        Point2::new(max_x, max_y),
        Point2::new(min_x, max_y),
    ]
}

fn assert_support_lines(layer: &LayerPrintPaths, expected: &[(f64, f64, f64, f64)]) {
    let actual = layer
        .paths()
        .iter()
        .filter(|path| path.role() == PrintPathRole::SupportMaterial)
        .map(|path| path.points().to_vec())
        .collect::<Vec<_>>();
    let expected = expected
        .iter()
        .map(|(x1, y1, x2, y2)| vec![Point2::new(*x1, *y1), Point2::new(*x2, *y2)])
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}

fn options(extra: Value) -> SliceOptions {
    let mut value = json!({
        "enable_support": true,
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "line_width": 0.4,
        "top_surface_line_width": 0.4,
        "support_remove_small_overhang": false,
        "support_ironing": true,
        "raft_first_layer_density": 10,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    });
    let extra = extra.as_object().expect("test options must be an object");
    for (key, value_extra) in extra {
        value[key] = value_extra.clone();
    }
    serde_json::from_value(value).unwrap()
}
