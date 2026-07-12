use crate::{LayerPrintPaths, Point2, PrintPath, PrintPathRole, SliceError, SliceOptions};
use serde_json::{Value, json};

#[test]
fn omitted_density_uses_orca_default_first_layer_pitch() {
    let finalized = finalize_layer(
        0,
        vec![support_rectangle(PrintPathRole::SupportMaterial)],
        json!({ "enable_support": true }),
    );

    assert_support_lines(
        finalized[0].paths(),
        &[
            [Point2::new(1.0, 1.0), Point2::new(3.0, 1.0)],
            [
                Point2::new(1.0, 1.4444444444444444),
                Point2::new(3.0, 1.4444444444444444),
            ],
            [
                Point2::new(1.0, 1.8888888888888888),
                Point2::new(3.0, 1.8888888888888888),
            ],
        ],
    );
}

#[test]
fn explicit_density_changes_first_layer_pitch() {
    let full_density = finalize_layer(
        0,
        vec![support_rectangle(PrintPathRole::SupportMaterial)],
        json!({
            "enable_support": true,
            "raft_first_layer_density": 100.0
        }),
    );
    let half_density = finalize_layer(
        0,
        vec![support_rectangle(PrintPathRole::SupportMaterial)],
        json!({
            "enable_support": true,
            "raft_first_layer_density": 50.0
        }),
    );

    assert_support_lines(
        full_density[0].paths(),
        &[
            [Point2::new(1.0, 1.0), Point2::new(3.0, 1.0)],
            [Point2::new(1.0, 1.4), Point2::new(3.0, 1.4)],
            [Point2::new(1.0, 1.8), Point2::new(3.0, 1.8)],
        ],
    );
    assert_support_lines(
        half_density[0].paths(),
        &[
            [Point2::new(1.0, 1.0), Point2::new(3.0, 1.0)],
            [Point2::new(1.0, 1.8), Point2::new(3.0, 1.8)],
        ],
    );
}

#[test]
fn non_first_layer_keeps_base_spacing_pitch() {
    let finalized = finalize_layer(
        1,
        vec![support_rectangle(PrintPathRole::SupportMaterial)],
        json!({
            "enable_support": true,
            "support_base_pattern_spacing": 0.0,
            "raft_first_layer_density": 50.0
        }),
    );

    assert_support_lines(
        finalized[0].paths(),
        &[
            [Point2::new(1.0, 1.0), Point2::new(3.0, 1.0)],
            [Point2::new(1.0, 1.4), Point2::new(3.0, 1.4)],
            [Point2::new(1.0, 1.8), Point2::new(3.0, 1.8)],
        ],
    );
}

#[test]
fn rectilinear_grid_uses_density_pitch_for_both_families() {
    let finalized = finalize_layer(
        0,
        vec![support_rectangle(PrintPathRole::SupportMaterial)],
        json!({
            "enable_support": true,
            "support_base_pattern": "rectilinear-grid",
            "raft_first_layer_density": 100.0
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
fn non_target_paths_are_unchanged_by_density() {
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
    let interface_rectangle = support_rectangle(PrintPathRole::SupportMaterialInterface);
    let solid_rectangle = PrintPath::new(PrintPathRole::SolidInfill, rectangle_points())
        .unwrap()
        .with_closed(true);

    let finalized = finalize_layer(
        0,
        vec![
            triangle.clone(),
            open_rectangle.clone(),
            interface_rectangle.clone(),
            solid_rectangle.clone(),
        ],
        json!({
            "enable_support": true,
            "raft_first_layer_density": 50.0,
            "support_ironing": true
        }),
    );

    assert_eq!(finalized[0].paths()[0], triangle);
    assert_eq!(finalized[0].paths()[1], open_rectangle);
    assert_eq!(finalized[0].paths()[2], interface_rectangle);
    assert!(
        finalized[0].paths()[3..finalized[0].paths().len() - 1]
            .iter()
            .all(|path| path.role() == PrintPathRole::Ironing)
    );
    assert_eq!(finalized[0].paths().last(), Some(&solid_rectangle));
}

#[test]
fn interface_conversion_runs_before_density() {
    let finalized = finalize_layer(
        0,
        vec![support_rectangle(PrintPathRole::SupportMaterialInterface)],
        json!({
            "enable_support": true,
            "support_interface_top_layers": 0,
            "raft_first_layer_density": 50.0
        }),
    );

    assert_support_lines(
        finalized[0].paths(),
        &[
            [Point2::new(1.0, 1.0), Point2::new(3.0, 1.0)],
            [Point2::new(1.0, 1.8), Point2::new(3.0, 1.8)],
        ],
    );
}

#[test]
fn density_composes_after_raft_first_layer_expansion() {
    let finalized = finalize_layer(
        0,
        vec![support_rectangle(PrintPathRole::SupportMaterial)],
        json!({
            "enable_support": false,
            "raft_layers": 1,
            "raft_expansion": 0.0,
            "raft_first_layer_expansion": 0.5,
            "raft_first_layer_density": 100.0
        }),
    );

    assert_support_lines(
        finalized[0].paths(),
        &[
            [Point2::new(0.5, 0.5), Point2::new(3.5, 0.5)],
            [Point2::new(0.5, 0.9), Point2::new(3.5, 0.9)],
            [Point2::new(0.5, 1.3), Point2::new(3.5, 1.3)],
            [Point2::new(0.5, 1.7), Point2::new(3.5, 1.7)],
            [Point2::new(0.5, 2.1), Point2::new(3.5, 2.1)],
            [Point2::new(0.5, 2.5), Point2::new(3.5, 2.5)],
        ],
    );
}

#[test]
fn invalid_density_is_reported_before_disabled_support_filtering() {
    let err = crate::finalize_print_paths(
        vec![LayerPrintPaths::new(
            0,
            0.2,
            vec![support_rectangle(PrintPathRole::SupportMaterial)],
        )],
        &options(json!({
            "enable_support": false,
            "raft_first_layer_density": 9.99
        })),
    )
    .unwrap_err();

    assert!(matches!(err, SliceError::InvalidInput(_)));
    assert!(err.to_string().contains("raft_first_layer_density"));
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
