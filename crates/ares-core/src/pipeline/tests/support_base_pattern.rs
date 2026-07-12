use crate::{LayerPrintPaths, Point2, PrintPath, PrintPathRole, SliceError, SliceOptions};
use serde_json::{Value, json};

#[test]
fn omitted_default_and_rectilinear_keep_single_base_family() {
    let omitted = finalize(
        vec![support_rectangle(PrintPathRole::SupportMaterial)],
        json!({ "support_base_pattern_spacing": 0.0 }),
    );
    let default = finalize(
        vec![support_rectangle(PrintPathRole::SupportMaterial)],
        json!({
            "support_base_pattern": "default",
            "support_base_pattern_spacing": 0.0
        }),
    );
    let rectilinear = finalize(
        vec![support_rectangle(PrintPathRole::SupportMaterial)],
        json!({
            "support_base_pattern": "rectilinear",
            "support_base_pattern_spacing": 0.0
        }),
    );

    assert_support_lines(
        omitted[0].paths(),
        &[
            [Point2::new(1.0, 1.0), Point2::new(3.0, 1.0)],
            [Point2::new(1.0, 1.4), Point2::new(3.0, 1.4)],
            [Point2::new(1.0, 1.8), Point2::new(3.0, 1.8)],
        ],
    );
    assert_eq!(default, omitted);
    assert_eq!(rectilinear, omitted);
}

#[test]
fn rectilinear_grid_adds_perpendicular_base_family() {
    let finalized = finalize(
        vec![support_rectangle(PrintPathRole::SupportMaterial)],
        json!({
            "support_base_pattern": "rectilinear-grid",
            "support_base_pattern_spacing": 0.0
        }),
    );

    assert_support_lines(
        finalized[0].paths(),
        &[
            [Point2::new(1.0, 1.0), Point2::new(3.0, 1.0)],
            [Point2::new(1.0, 1.4), Point2::new(3.0, 1.4)],
            [Point2::new(1.0, 1.8), Point2::new(3.0, 1.8)],
            [Point2::new(1.0, 1.0), Point2::new(1.0, 2.0)],
            [Point2::new(1.4, 1.0), Point2::new(1.4, 2.0)],
            [Point2::new(1.8, 1.0), Point2::new(1.8, 2.0)],
            [Point2::new(2.2, 1.0), Point2::new(2.2, 2.0)],
            [Point2::new(2.6, 1.0), Point2::new(2.6, 2.0)],
            [Point2::new(3.0, 1.0), Point2::new(3.0, 2.0)],
        ],
    );
}

#[test]
fn grid_alias_is_local_legacy_compatibility_for_rectilinear_grid() {
    let upstream_name = finalize(
        vec![support_rectangle(PrintPathRole::SupportMaterial)],
        json!({
            "support_base_pattern": "rectilinear-grid",
            "support_base_pattern_spacing": 0.0
        }),
    );
    let legacy_alias = finalize(
        vec![support_rectangle(PrintPathRole::SupportMaterial)],
        json!({
            "support_base_pattern": "grid",
            "support_base_pattern_spacing": 0.0
        }),
    );

    assert_eq!(legacy_alias, upstream_name);
}

#[test]
fn rectilinear_grid_changes_support_material_coordinates_and_line_count() {
    let rectilinear = finalize(
        vec![support_rectangle(PrintPathRole::SupportMaterial)],
        json!({
            "support_base_pattern": "rectilinear",
            "support_base_pattern_spacing": 0.0
        }),
    );
    let grid = finalize(
        vec![support_rectangle(PrintPathRole::SupportMaterial)],
        json!({
            "support_base_pattern": "rectilinear-grid",
            "support_base_pattern_spacing": 0.0
        }),
    );

    assert_ne!(grid, rectilinear);
    assert_eq!(rectilinear[0].paths().len(), 3);
    assert_eq!(grid[0].paths().len(), 9);
    assert_eq!(
        grid[0].paths()[4].points(),
        [Point2::new(1.4, 1.0), Point2::new(1.4, 2.0)]
    );
    assert_eq!(
        grid[0].paths()[8].points(),
        [Point2::new(3.0, 1.0), Point2::new(3.0, 2.0)]
    );
}

#[test]
fn rectilinear_grid_composes_with_support_angle() {
    let finalized = finalize(
        vec![support_rectangle(PrintPathRole::SupportMaterial)],
        json!({
            "support_angle": 90.0,
            "support_base_pattern": "rectilinear-grid",
            "support_base_pattern_spacing": 0.6
        }),
    );

    assert_support_lines(
        finalized[0].paths(),
        &[
            [Point2::new(1.0, 1.0), Point2::new(1.0, 2.0)],
            [Point2::new(2.0, 1.0), Point2::new(2.0, 2.0)],
            [Point2::new(3.0, 1.0), Point2::new(3.0, 2.0)],
            [Point2::new(3.0, 1.0), Point2::new(1.0, 1.0)],
            [Point2::new(3.0, 2.0), Point2::new(1.0, 2.0)],
        ],
    );
}

#[test]
fn rectilinear_grid_preserves_source_metadata_and_extrusion_role() {
    let source = support_rectangle(PrintPathRole::SupportMaterial)
        .with_extrusion_role(PrintPathRole::SupportMaterial)
        .with_effective_layer_height_mm(0.13)
        .with_unsupported_span_mm(Some(2.5))
        .with_seam_gap_mm(0.07)
        .with_closed(true);
    let finalized = finalize(
        vec![source],
        json!({
            "support_base_pattern": "rectilinear-grid",
            "support_base_pattern_spacing": 0.0
        }),
    );

    assert_eq!(finalized[0].layer_id(), 7);
    assert_eq!(finalized[0].print_z(), 1.6);
    for path in finalized[0].paths() {
        assert_eq!(path.role(), PrintPathRole::SupportMaterial);
        assert_eq!(path.extrusion_role(), Some(PrintPathRole::SupportMaterial));
        assert_eq!(path.effective_layer_height_mm(), Some(0.13));
        assert_eq!(path.unsupported_span_mm(), Some(2.5));
        assert_eq!(path.seam_gap_mm(), 0.07);
        assert!(!path.is_closed());
    }
}

#[test]
fn zero_top_interface_layers_converts_interface_before_base_pattern_selection() {
    let finalized = finalize(
        vec![
            support_rectangle(PrintPathRole::SupportMaterialInterface)
                .with_extrusion_role(PrintPathRole::SupportMaterialInterface),
        ],
        json!({
            "support_interface_top_layers": 0,
            "support_base_pattern": "rectilinear-grid",
            "support_base_pattern_spacing": 0.0
        }),
    );

    assert_support_lines(
        finalized[0].paths(),
        &[
            [Point2::new(1.0, 1.0), Point2::new(3.0, 1.0)],
            [Point2::new(1.0, 1.4), Point2::new(3.0, 1.4)],
            [Point2::new(1.0, 1.8), Point2::new(3.0, 1.8)],
            [Point2::new(1.0, 1.0), Point2::new(1.0, 2.0)],
            [Point2::new(1.4, 1.0), Point2::new(1.4, 2.0)],
            [Point2::new(1.8, 1.0), Point2::new(1.8, 2.0)],
            [Point2::new(2.2, 1.0), Point2::new(2.2, 2.0)],
            [Point2::new(2.6, 1.0), Point2::new(2.6, 2.0)],
            [Point2::new(3.0, 1.0), Point2::new(3.0, 2.0)],
        ],
    );
    assert_eq!(finalized[0].paths()[0].extrusion_role(), None);
}

#[test]
fn non_target_paths_are_unchanged() {
    let triangle = PrintPath::new(
        PrintPathRole::SupportMaterial,
        vec![
            Point2::new(1.0, 1.0),
            Point2::new(3.0, 1.0),
            Point2::new(2.0, 2.0),
        ],
    )
    .unwrap()
    .with_closed(true);
    let open_rectangle =
        PrintPath::new(PrintPathRole::SupportMaterial, rectangle_points()).unwrap();
    let interface = support_rectangle(PrintPathRole::SupportMaterialInterface)
        .with_extrusion_role(PrintPathRole::SupportMaterialInterface)
        .with_closed(true);
    let solid_rectangle = PrintPath::new(PrintPathRole::SolidInfill, rectangle_points())
        .unwrap()
        .with_closed(true);
    let finalized = finalize(
        vec![
            triangle.clone(),
            open_rectangle.clone(),
            interface.clone(),
            solid_rectangle.clone(),
        ],
        json!({
            "support_base_pattern": "rectilinear-grid",
            "support_ironing": true
        }),
    );

    assert!(finalized[0].paths().len() > 4);
    assert_eq!(
        &finalized[0].paths()[..3],
        [triangle, open_rectangle, interface]
    );
    assert_eq!(finalized[0].paths().last(), Some(&solid_rectangle));
}

#[test]
fn accepted_deferred_patterns_keep_current_rectangular_output() {
    let rectilinear = finalize(
        vec![support_rectangle(PrintPathRole::SupportMaterial)],
        json!({
            "support_base_pattern": "rectilinear",
            "support_base_pattern_spacing": 0.0
        }),
    );

    for pattern in ["honeycomb", "lightning", "hollow"] {
        let finalized = finalize(
            vec![support_rectangle(PrintPathRole::SupportMaterial)],
            json!({
                "support_base_pattern": pattern,
                "support_base_pattern_spacing": 0.0
            }),
        );
        assert_eq!(finalized, rectilinear, "{pattern}");
    }
}

#[test]
fn invalid_patterns_reach_slice_error() {
    for value in [
        json!("crosshatch"),
        json!(7),
        json!(true),
        Value::Null,
        json!([]),
        json!({ "value": "rectilinear-grid" }),
    ] {
        let err = crate::finalize_print_paths(
            vec![LayerPrintPaths::new(
                1,
                0.4,
                vec![support_rectangle(PrintPathRole::SupportMaterial)],
            )],
            &options(json!({ "support_base_pattern": value })),
        )
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("support_base_pattern"));
    }
}

fn finalize(paths: Vec<PrintPath>, extra: Value) -> Vec<LayerPrintPaths> {
    crate::finalize_print_paths(vec![LayerPrintPaths::new(7, 1.6, paths)], &options(extra)).unwrap()
}

fn support_rectangle(role: PrintPathRole) -> PrintPath {
    PrintPath::new(role, rectangle_points())
        .unwrap()
        .with_closed(true)
}

fn rectangle_points() -> Vec<Point2> {
    vec![
        Point2::new(1.0, 1.0),
        Point2::new(3.0, 1.0),
        Point2::new(3.0, 2.0),
        Point2::new(1.0, 2.0),
    ]
}

fn assert_support_lines(paths: &[PrintPath], expected: &[[Point2; 2]]) {
    assert_eq!(paths.len(), expected.len());
    for (path, points) in paths.iter().zip(expected) {
        assert_eq!(path.role(), PrintPathRole::SupportMaterial);
        assert_eq!(path.points(), *points);
        assert!(!path.is_closed());
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
