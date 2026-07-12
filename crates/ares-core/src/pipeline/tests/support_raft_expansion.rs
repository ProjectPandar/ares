use crate::{LayerPrintPaths, Point2, PrintPath, PrintPathRole, SliceError, SliceOptions};
use serde_json::{Value, json};

#[test]
fn default_raft_expansion_expands_layer_zero_when_raft_active_and_first_layer_expansion_is_zero() {
    let finalized = finalize_layer(
        0,
        vec![support_rectangle(PrintPathRole::SupportMaterial)],
        json!({
            "enable_support": false,
            "raft_layers": 1,
            "raft_first_layer_expansion": 0.0
        }),
    );

    assert_support_lines(
        finalized[0].paths(),
        &horizontal_lines_by_pitch(-0.5, 4.5, -0.5, 3.5, density_pitch(90.0)),
    );
}

#[test]
fn omitted_raft_and_first_layer_expansions_compose_defaults_on_layer_zero() {
    let finalized = finalize_layer(
        0,
        vec![support_rectangle(PrintPathRole::SupportMaterial)],
        json!({
            "enable_support": false,
            "raft_layers": 1
        }),
    );

    assert_support_lines(
        finalized[0].paths(),
        &horizontal_lines_by_pitch(-2.5, 6.5, -2.5, 5.5, density_pitch(90.0)),
    );
}

#[test]
fn configured_raft_expansion_applies_to_existing_layers_below_raft_layer_count() {
    let finalized = finalize_layers(
        vec![
            layer(0, support_rectangle(PrintPathRole::SupportMaterial)),
            layer(1, support_rectangle(PrintPathRole::SupportMaterial)),
            layer(2, support_rectangle(PrintPathRole::SupportMaterial)),
        ],
        json!({
            "enable_support": false,
            "raft_layers": 2,
            "raft_expansion": 0.5,
            "raft_first_layer_expansion": 0.0,
            "support_base_pattern_spacing": 0.0,
            "raft_first_layer_density": 100.0
        }),
    );

    assert_support_lines(
        finalized[0].paths(),
        &horizontal_lines_by_pitch(0.5, 3.5, 0.5, 2.5, density_pitch(100.0)),
    );
    assert_support_lines(
        finalized[1].paths(),
        &horizontal_lines_by_pitch(0.5, 3.5, 0.5, 2.5, 0.4),
    );
    assert_support_lines(
        finalized[2].paths(),
        &horizontal_lines(1.0, 3.0, &[1.0, 1.4, 1.8]),
    );
}

#[test]
fn zero_or_inactive_raft_expansion_keeps_support_geometry() {
    for extra in [
        json!({
            "enable_support": false,
            "raft_layers": 1,
            "raft_expansion": 0.0,
            "raft_first_layer_expansion": 0.0
        }),
        json!({
            "enable_support": true,
            "raft_expansion": 0.5,
            "raft_first_layer_expansion": 0.0
        }),
        json!({
            "enable_support": true,
            "raft_layers": 0,
            "raft_expansion": 0.5,
            "raft_first_layer_expansion": 0.0
        }),
    ] {
        let finalized = finalize_layer(
            0,
            vec![support_rectangle(PrintPathRole::SupportMaterial)],
            extra,
        );

        assert_support_lines(
            finalized[0].paths(),
            &horizontal_lines(1.0, 3.0, &[1.0, 1.4444444444444444, 1.8888888888888888]),
        );
    }
}

#[test]
fn raft_expansion_expands_interface_before_spacing_and_support_ironing() {
    let finalized = finalize_layer(
        0,
        vec![
            support_rectangle(PrintPathRole::SupportMaterialInterface)
                .with_extrusion_role(PrintPathRole::SupportMaterialInterface),
        ],
        json!({
            "enable_support": false,
            "raft_layers": 1,
            "raft_expansion": 0.5,
            "raft_first_layer_expansion": 0.0,
            "support_ironing": true,
            "support_ironing_spacing": 1.0
        }),
    );

    assert_eq!(finalized[0].paths().len(), 4);
    assert_eq!(
        finalized[0].paths()[0].points(),
        [
            Point2::new(0.5, 0.5),
            Point2::new(3.5, 0.5),
            Point2::new(3.5, 2.5),
            Point2::new(0.5, 2.5),
        ]
    );
    let ironing = finalized[0]
        .paths()
        .iter()
        .filter(|path| path.role() == PrintPathRole::Ironing)
        .collect::<Vec<_>>();
    assert_eq!(ironing.len(), 3);
    assert_eq!(
        ironing[0].points(),
        [Point2::new(0.5, 0.5), Point2::new(3.5, 0.5)]
    );
}

#[test]
fn raft_expansion_composes_after_support_expansion_and_before_first_layer_expansion() {
    let finalized = finalize_layer(
        0,
        vec![support_rectangle(PrintPathRole::SupportMaterial)],
        json!({
            "enable_support": false,
            "raft_layers": 1,
            "support_expansion": 0.25,
            "raft_expansion": 0.5,
            "raft_first_layer_expansion": 0.75,
            "raft_first_layer_density": 100.0
        }),
    );

    assert_support_lines(
        finalized[0].paths(),
        &horizontal_lines_by_pitch(-0.5, 4.5, -0.5, 3.5, density_pitch(100.0)),
    );
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
            "raft_expansion": 0.5,
            "raft_first_layer_expansion": 0.0
        }),
    );

    assert_eq!(
        finalized[0].paths(),
        [triangle, open_rectangle, solid_rectangle]
    );
}

#[test]
fn invalid_raft_expansion_values_reach_slice_error_before_disabled_support_filtering() {
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
                "raft_expansion": value,
                "raft_first_layer_expansion": 0.0
            })),
        )
        .unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("raft_expansion"));
    }
}

fn finalize_layer(layer_id: usize, paths: Vec<PrintPath>, extra: Value) -> Vec<LayerPrintPaths> {
    finalize_layers(
        vec![LayerPrintPaths::new(layer_id, print_z(layer_id), paths)],
        extra,
    )
}

fn finalize_layers(layers: Vec<LayerPrintPaths>, extra: Value) -> Vec<LayerPrintPaths> {
    crate::finalize_print_paths(layers, &options(extra)).unwrap()
}

fn layer(layer_id: usize, path: PrintPath) -> LayerPrintPaths {
    LayerPrintPaths::new(layer_id, print_z(layer_id), vec![path])
}

fn print_z(layer_id: usize) -> f64 {
    if layer_id == 0 {
        0.2
    } else {
        0.2 + layer_id as f64 * 0.2
    }
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

fn horizontal_lines(min_x: f64, max_x: f64, y_values: &[f64]) -> Vec<[Point2; 2]> {
    y_values
        .iter()
        .map(|y| [Point2::new(min_x, *y), Point2::new(max_x, *y)])
        .collect()
}

fn horizontal_lines_by_pitch(
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
    pitch: f64,
) -> Vec<[Point2; 2]> {
    let mut y = min_y;
    let mut lines = Vec::new();
    while y <= max_y + 1.0e-9 {
        lines.push([Point2::new(min_x, y), Point2::new(max_x, y)]);
        y += pitch;
    }
    lines
}

fn density_pitch(percent: f64) -> f64 {
    0.4 / (percent / 100.0)
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
