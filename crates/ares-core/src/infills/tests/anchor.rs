use super::*;
use crate::SliceOptions;
use serde_json::{Value, json};

fn anchor_options(anchor: Value) -> InfillOptions {
    let options: SliceOptions = serde_json::from_value(json!({
        "sparse_infill_density": 50,
        "sparse_infill_line_width": 0.5,
        "minimum_sparse_infill_area": 0,
        "sparse_infill_pattern": "rectilinear",
        "infill_direction": 0,
        "infill_anchor": anchor,
        "infill_anchor_max": 20,
        "wall_loops": 0,
        "bottom_shell_layers": 0,
        "top_shell_layers": 0
    }))
    .unwrap();
    options.infill_options().unwrap()
}

#[test]
fn infill_anchor_extends_sparse_segments_at_both_ends() {
    let layers = vec![square_layer()];
    let options = anchor_options(json!(0.25));

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_eq!(
        infills[0].paths()[0].points(),
        &[Point2::new(0.5, -0.25), Point2::new(0.5, 2.25)]
    );
}

#[test]
fn zero_infill_anchor_preserves_sparse_segments() {
    let layers = vec![square_layer()];

    let infills = generate_infills(
        &print_layers(&layers),
        &layers,
        options(InfillPattern::Rectilinear),
    )
    .unwrap();

    assert_eq!(
        infills[0].paths()[0].points(),
        &[Point2::new(0.5, 0.0), Point2::new(0.5, 2.0)]
    );
}

#[test]
fn infill_anchor_keeps_hole_segments_split() {
    let layers = vec![LayerContours::new(
        0,
        0.4,
        vec![
            Contour::new(vec![
                Point2::new(0.0, 0.0),
                Point2::new(4.0, 0.0),
                Point2::new(4.0, 4.0),
                Point2::new(0.0, 4.0),
            ]),
            Contour::new(vec![
                Point2::new(1.0, 1.0),
                Point2::new(3.0, 1.0),
                Point2::new(3.0, 3.0),
                Point2::new(1.0, 3.0),
            ]),
        ],
    )];
    let options = anchor_options(json!(0.25));

    let infills = generate_infills(&print_layers(&layers), &layers, options).unwrap();

    assert_eq!(infills[0].paths().len(), 6);
    assert_eq!(
        infills[0].paths()[1].points(),
        &[Point2::new(1.5, -0.25), Point2::new(1.5, 1.25)]
    );
    assert_eq!(
        infills[0].paths()[2].points(),
        &[Point2::new(1.5, 2.75), Point2::new(1.5, 4.25)]
    );
}
