use super::*;
use crate::{Layer, LayerToolpathMoves, Point2, ToolpathMove, ToolpathMoveKind};

#[test]
fn adaptive_volumetric_geometry_uses_effective_layer_height_and_role_width() {
    let layers = [Layer::new(1, 0.2, 0.4)];
    let moves = [LayerToolpathMoves::new(
        1,
        0.4,
        vec![
            ToolpathMove::new(
                ToolpathMoveKind::Travel,
                PrintPathRole::SparseInfill,
                Point2::new(0.0, 0.0),
            )
            .with_effective_layer_height_mm(Some(0.4)),
            ToolpathMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::SparseInfill,
                Point2::new(1.0, 0.0),
            )
            .with_effective_layer_height_mm(Some(0.4)),
        ],
    )];
    let options = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.5, (0.3, 0.25), 0.45);

    let output = generate_extrusion_moves(&layers, &moves, options).unwrap();

    assert_eq!(output[0].moves()[0].effective_layer_height_mm(), None);
    assert_eq!(output[0].moves()[0].effective_line_width_mm(), None);
    assert_eq!(output[0].moves()[1].effective_layer_height_mm(), Some(0.4));
    assert_eq!(output[0].moves()[1].effective_line_width_mm(), Some(0.45));
}

#[test]
fn adaptive_volumetric_geometry_falls_back_to_layer_height_and_first_layer_width() {
    let layers = [Layer::new(0, 0.2, 0.2)];
    let moves = [LayerToolpathMoves::new(
        0,
        0.2,
        vec![
            ToolpathMove::new(
                ToolpathMoveKind::Travel,
                PrintPathRole::ExternalPerimeter,
                Point2::new(0.0, 0.0),
            ),
            ToolpathMove::new(
                ToolpathMoveKind::Print,
                PrintPathRole::ExternalPerimeter,
                Point2::new(1.0, 0.0),
            ),
        ],
    )];
    let options = ExtrusionOptions::new_for_tests(0.4, 2.0, 0.4, (0.5, 0.25), 0.3)
        .with_initial_layer_line_width(0.6);

    let output = generate_extrusion_moves(&layers, &moves, options).unwrap();

    assert_eq!(output[0].moves()[1].effective_layer_height_mm(), Some(0.2));
    assert_eq!(output[0].moves()[1].effective_line_width_mm(), Some(0.6));
}
