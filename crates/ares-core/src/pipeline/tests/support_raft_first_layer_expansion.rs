use crate::{LayerPrintPaths, Point2, PrintPath, PrintPathRole, SliceError, SliceOptions};
use serde_json::{Value, json};

#[test]
fn raft_default_first_layer_expansion_expands_support_material_before_base_spacing() {
    let finalized = finalize_layer(
        0,
        vec![support_rectangle(PrintPathRole::SupportMaterial)],
        json!({
            "enable_support": false,
            "raft_layers": 1,
            "raft_expansion": 0.0
        }),
    );

    let expected = horizontal_lines(
        -1.0,
        5.0,
        &[
            -1.0,
            -0.5555555555555556,
            -0.11111111111111116,
            0.33333333333333326,
            0.7777777777777777,
            1.222222222222222,
            1.6666666666666665,
            2.1111111111111107,
            2.5555555555555554,
            3.0,
            3.4444444444444446,
            3.8888888888888893,
        ],
    );
    assert_support_lines(finalized[0].paths(), &expected);
}

#[test]
fn raft_explicit_first_layer_expansion_expands_interface_before_spacing() {
    let finalized = finalize_layer(
        0,
        vec![
            support_rectangle(PrintPathRole::SupportMaterialInterface)
                .with_extrusion_role(PrintPathRole::SupportMaterialInterface),
        ],
        json!({
            "enable_support": false,
            "raft_layers": 1,
            "raft_expansion": 0.0,
            "raft_first_layer_expansion": "0.5"
        }),
    );

    assert_interface_lines(
        finalized[0].paths(),
        &[
            [Point2::new(0.5, 0.5), Point2::new(0.5, 2.5)],
            [Point2::new(1.4, 0.5), Point2::new(1.4, 2.5)],
            [Point2::new(2.3, 0.5), Point2::new(2.3, 2.5)],
            [Point2::new(3.2, 0.5), Point2::new(3.2, 2.5)],
        ],
    );
}

#[test]
fn raft_first_layer_expansion_expands_interface_before_support_ironing() {
    let finalized = finalize_layer(
        0,
        vec![
            support_rectangle(PrintPathRole::SupportMaterialInterface)
                .with_extrusion_role(PrintPathRole::SupportMaterialInterface),
        ],
        json!({
            "enable_support": false,
            "raft_layers": 1,
            "raft_expansion": 0.0,
            "raft_first_layer_expansion": 0.5,
            "support_ironing": true,
            "support_ironing_spacing": 1.0
        }),
    );

    assert_eq!(finalized[0].paths().len(), 4);
    assert_eq!(
        finalized[0].paths()[0].role(),
        PrintPathRole::SupportMaterialInterface
    );
    assert_eq!(
        finalized[0].paths()[0].points(),
        [
            Point2::new(0.5, 0.5),
            Point2::new(3.5, 0.5),
            Point2::new(3.5, 2.5),
            Point2::new(0.5, 2.5),
        ]
    );
    assert!(finalized[0].paths()[0].is_closed());
    let ironing_paths = finalized[0]
        .paths()
        .iter()
        .filter(|path| path.role() == PrintPathRole::Ironing)
        .collect::<Vec<_>>();
    assert_eq!(ironing_paths.len(), 3);
    assert_eq!(
        ironing_paths[0].points(),
        [Point2::new(0.5, 0.5), Point2::new(3.5, 0.5)]
    );
}

#[test]
fn expanded_first_layer_support_material_preserves_source_metadata() {
    let source = support_rectangle(PrintPathRole::SupportMaterial)
        .with_extrusion_role(PrintPathRole::SupportMaterial)
        .with_effective_layer_height_mm(0.13)
        .with_unsupported_span_mm(Some(2.5))
        .with_seam_gap_mm(0.07);
    let finalized = finalize_layer(
        0,
        vec![source],
        json!({
            "enable_support": false,
            "raft_layers": 1,
            "raft_expansion": 0.0,
            "raft_first_layer_expansion": 0.5
        }),
    );

    let expected = horizontal_lines(
        0.5,
        3.5,
        &[
            0.5,
            0.9444444444444444,
            1.3888888888888888,
            1.8333333333333333,
            2.2777777777777777,
        ],
    );
    assert_support_lines(finalized[0].paths(), &expected);
    for expanded in finalized[0].paths() {
        assert_eq!(
            expanded.extrusion_role(),
            Some(PrintPathRole::SupportMaterial)
        );
        assert_eq!(expanded.effective_layer_height_mm(), Some(0.13));
        assert_eq!(expanded.unsupported_span_mm(), Some(2.5));
        assert_eq!(expanded.seam_gap_mm(), 0.07);
    }
}

#[test]
fn zero_raft_first_layer_expansion_keeps_first_layer_support_geometry() {
    let finalized = finalize_layer(
        0,
        vec![support_rectangle(PrintPathRole::SupportMaterial)],
        json!({
            "enable_support": false,
            "raft_layers": 1,
            "raft_expansion": 0.0,
            "raft_first_layer_expansion": 0.0
        }),
    );

    let expected = horizontal_lines(1.0, 3.0, &[1.0, 1.4444444444444444, 1.8888888888888888]);
    assert_support_lines(finalized[0].paths(), &expected);
}

#[test]
fn raft_inactive_first_layer_expansion_keeps_support_geometry() {
    for extra in [
        json!({
            "enable_support": true,
            "raft_expansion": 0.0,
            "raft_first_layer_expansion": 0.5
        }),
        json!({
            "enable_support": true,
            "raft_layers": 0,
            "raft_expansion": 0.0,
            "raft_first_layer_expansion": 0.5
        }),
    ] {
        let finalized = finalize_layer(
            0,
            vec![support_rectangle(PrintPathRole::SupportMaterial)],
            extra,
        );

        let expected = horizontal_lines(1.0, 3.0, &[1.0, 1.4444444444444444, 1.8888888888888888]);
        assert_support_lines(finalized[0].paths(), &expected);
    }
}

#[test]
fn non_first_layer_support_geometry_is_not_expanded() {
    let finalized = finalize_layer(
        1,
        vec![support_rectangle(PrintPathRole::SupportMaterial)],
        json!({
            "enable_support": false,
            "raft_layers": 1,
            "raft_expansion": 0.0,
            "raft_first_layer_expansion": 0.5
        }),
    );

    assert_eq!(finalized[0].layer_id(), 1);
    assert_eq!(finalized[0].print_z(), 0.4);
    assert_support_lines(
        finalized[0].paths(),
        &[[Point2::new(1.0, 1.0), Point2::new(3.0, 1.0)]],
    );
}

#[test]
fn raft_first_layer_expansion_composes_after_support_expansion() {
    let finalized = finalize_layer(
        0,
        vec![support_rectangle(PrintPathRole::SupportMaterial)],
        json!({
            "enable_support": false,
            "raft_layers": 1,
            "support_expansion": 0.25,
            "raft_expansion": 0.0,
            "raft_first_layer_expansion": 0.5
        }),
    );

    let expected = horizontal_lines(
        0.25,
        3.75,
        &[
            0.25,
            0.6944444444444444,
            1.1388888888888888,
            1.5833333333333333,
            2.0277777777777777,
            2.4722222222222223,
        ],
    );
    assert_support_lines(finalized[0].paths(), &expected);
}

#[test]
fn non_rectangular_open_and_non_support_paths_are_not_expanded() {
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
    let solid_rectangle = PrintPath::new(PrintPathRole::SolidInfill, rectangle_points())
        .unwrap()
        .with_closed(true);

    let finalized = finalize_layer(
        0,
        vec![
            triangle.clone(),
            open_rectangle.clone(),
            solid_rectangle.clone(),
        ],
        json!({
            "enable_support": false,
            "raft_layers": 1,
            "raft_expansion": 0.0,
            "raft_first_layer_expansion": 0.5
        }),
    );

    assert_eq!(
        finalized[0].paths(),
        [triangle, open_rectangle, solid_rectangle]
    );
}

#[test]
fn invalid_raft_first_layer_expansion_values_reach_slice_error() {
    for value in [
        json!(-0.1),
        json!("NaN"),
        json!("inf"),
        json!("0.5mm"),
        json!([]),
        json!({ "value": 0.5 }),
        json!(true),
        Value::Null,
    ] {
        let err = crate::finalize_print_paths(
            vec![LayerPrintPaths::new(
                0,
                0.2,
                vec![support_rectangle(PrintPathRole::SupportMaterial)],
            )],
            &options(json!({
                "enable_support": false,
                "raft_layers": 1,
                "raft_expansion": 0.0,
                "raft_first_layer_expansion": value
            })),
        )
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("raft_first_layer_expansion"));
    }
}

fn finalize_layer(layer_id: usize, paths: Vec<PrintPath>, extra: Value) -> Vec<LayerPrintPaths> {
    crate::finalize_print_paths(
        vec![LayerPrintPaths::new(layer_id, print_z(layer_id), paths)],
        &options(extra),
    )
    .unwrap()
}

fn print_z(layer_id: usize) -> f64 {
    if layer_id == 0 { 0.2 } else { 0.4 }
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

fn horizontal_lines(min_x: f64, max_x: f64, y_values: &[f64]) -> Vec<[Point2; 2]> {
    y_values
        .iter()
        .map(|y| [Point2::new(min_x, *y), Point2::new(max_x, *y)])
        .collect()
}

fn assert_interface_lines(paths: &[PrintPath], expected: &[[Point2; 2]]) {
    assert_eq!(paths.len(), expected.len());
    for (path, points) in paths.iter().zip(expected) {
        assert_eq!(path.role(), PrintPathRole::SupportMaterialInterface);
        assert_eq!(path.points(), *points);
        assert!(!path.is_closed());
    }
}

fn options(extra: Value) -> SliceOptions {
    let mut value = json!({
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
