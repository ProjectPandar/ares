use super::*;
use crate::{LayerPrintPaths, Point2, PrintPath, PrintPathRole};

fn sample_layer(role: PrintPathRole, points: Vec<Point2>) -> LayerPrintPaths {
    LayerPrintPaths::new(0, 0.2, vec![PrintPath::new(role, points).unwrap()])
}

#[test]
fn generates_travel_and_closed_perimeter_print_moves() {
    let layers = [sample_layer(
        PrintPathRole::ExternalPerimeter,
        vec![
            Point2::new(0.0, 0.0),
            Point2::new(1.0, 0.0),
            Point2::new(1.0, 1.0),
        ],
    )];

    let moves = generate_toolpath_moves(&layers);

    assert_eq!(moves[0].moves().len(), 4);
    assert_eq!(moves[0].moves()[0].kind(), ToolpathMoveKind::Travel);
    assert_eq!(moves[0].moves()[0].point(), Point2::new(0.0, 0.0));
    assert_eq!(moves[0].moves()[3].kind(), ToolpathMoveKind::Print);
    assert_eq!(moves[0].moves()[3].point(), Point2::new(0.0, 0.0));
}

#[test]
fn closes_internal_perimeter_print_moves() {
    let layers = [sample_layer(
        PrintPathRole::InternalPerimeter,
        vec![
            Point2::new(0.4, 0.4),
            Point2::new(3.6, 0.4),
            Point2::new(3.6, 3.6),
            Point2::new(0.4, 3.6),
        ],
    )];

    let moves = generate_toolpath_moves(&layers);

    assert_eq!(moves[0].moves().len(), 5);
    assert_eq!(moves[0].moves()[0].kind(), ToolpathMoveKind::Travel);
    assert_eq!(moves[0].moves()[4].kind(), ToolpathMoveKind::Print);
    assert_eq!(moves[0].moves()[4].point(), Point2::new(0.4, 0.4));
}

#[test]
fn generates_open_sparse_infill_print_moves() {
    let layers = [sample_layer(
        PrintPathRole::SparseInfill,
        vec![Point2::new(0.5, 0.0), Point2::new(0.5, 1.0)],
    )];

    let moves = generate_toolpath_moves(&layers);

    assert_eq!(moves[0].moves().len(), 2);
    assert_eq!(moves[0].moves()[0].kind(), ToolpathMoveKind::Travel);
    assert_eq!(moves[0].moves()[1].kind(), ToolpathMoveKind::Print);
    assert_eq!(moves[0].moves()[1].point(), Point2::new(0.5, 1.0));
}

#[test]
fn generates_open_solid_infill_print_moves() {
    let layers = [sample_layer(
        PrintPathRole::SolidInfill,
        vec![Point2::new(0.5, 0.0), Point2::new(0.5, 1.0)],
    )];

    let moves = generate_toolpath_moves(&layers);

    assert_eq!(moves[0].moves().len(), 2);
    assert_eq!(moves[0].moves()[0].kind(), ToolpathMoveKind::Travel);
    assert_eq!(moves[0].moves()[1].kind(), ToolpathMoveKind::Print);
    assert_eq!(moves[0].moves()[1].role(), PrintPathRole::SolidInfill);
}

#[test]
fn closes_skirt_loops_like_perimeters() {
    let layers = [sample_layer(
        PrintPathRole::Skirt,
        vec![
            Point2::new(-2.5, -2.5),
            Point2::new(2.5, -2.5),
            Point2::new(2.5, 2.5),
            Point2::new(-2.5, 2.5),
        ],
    )];

    let output = generate_toolpath_moves(&layers);

    assert_eq!(output[0].moves().len(), 5);
    assert_eq!(output[0].moves()[0].kind(), ToolpathMoveKind::Travel);
    assert_eq!(output[0].moves()[4].point(), Point2::new(-2.5, -2.5));
}

#[test]
fn closes_brim_loops_like_other_closed_paths() {
    let paths = [LayerPrintPaths::new(
        0,
        0.2,
        vec![
            PrintPath::new(
                PrintPathRole::Brim,
                vec![
                    Point2::new(-1.0, -1.0),
                    Point2::new(2.0, -1.0),
                    Point2::new(2.0, 2.0),
                    Point2::new(-1.0, 2.0),
                ],
            )
            .unwrap(),
        ],
    )];
    let output = generate_toolpath_moves(&paths);
    assert_eq!(output[0].moves().len(), 5);
    assert_eq!(output[0].moves()[4].point(), Point2::new(-1.0, -1.0));
}

#[test]
fn closes_overhang_perimeter_print_moves() {
    let paths = [LayerPrintPaths::new(
        0,
        0.2,
        vec![
            PrintPath::new(
                PrintPathRole::OverhangPerimeter,
                vec![
                    Point2::new(1.0, 1.0),
                    Point2::new(3.0, 1.0),
                    Point2::new(3.0, 3.0),
                    Point2::new(1.0, 3.0),
                ],
            )
            .unwrap(),
        ],
    )];

    let output = generate_toolpath_moves(&paths);

    assert_eq!(output[0].moves().len(), 5);
    assert_eq!(output[0].moves()[4].kind(), ToolpathMoveKind::Print);
    assert_eq!(output[0].moves()[4].point(), Point2::new(1.0, 1.0));
}

#[test]
fn preserves_layer_and_role_metadata() {
    let layers = [LayerPrintPaths::new(
        3,
        0.6,
        vec![
            PrintPath::new(
                PrintPathRole::SparseInfill,
                vec![Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)],
            )
            .unwrap(),
        ],
    )];

    let moves = generate_toolpath_moves(&layers);

    assert_eq!(moves[0].layer_id(), 3);
    assert_eq!(moves[0].print_z(), 0.6);
    assert_eq!(moves[0].moves()[0].role(), PrintPathRole::SparseInfill);
}

#[test]
fn carries_effective_height_from_print_paths() {
    let layers = [LayerPrintPaths::new(
        3,
        0.6,
        vec![
            PrintPath::new(
                PrintPathRole::SparseInfill,
                vec![Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)],
            )
            .unwrap()
            .with_effective_layer_height_mm(0.4),
        ],
    )];

    let moves = generate_toolpath_moves(&layers);

    assert_eq!(moves[0].moves()[0].effective_layer_height_mm(), Some(0.4));
    assert_eq!(moves[0].moves()[1].effective_layer_height_mm(), Some(0.4));
}

#[test]
fn preserves_empty_represented_layers() {
    let layers = [LayerPrintPaths::new(7, 1.4, Vec::new())];

    let moves = generate_toolpath_moves(&layers);

    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].layer_id(), 7);
    assert_eq!(moves[0].print_z(), 1.4);
    assert!(moves[0].moves().is_empty());
}

#[test]
fn positive_seam_gap_does_not_clip_skirt_or_brim_closing_moves() {
    let paths = [LayerPrintPaths::new(
        0,
        0.2,
        vec![
            PrintPath::new(
                PrintPathRole::Skirt,
                vec![
                    Point2::new(-1.0, -1.0),
                    Point2::new(1.0, -1.0),
                    Point2::new(1.0, 1.0),
                    Point2::new(-1.0, 1.0),
                ],
            )
            .unwrap()
            .with_seam_gap_mm(1.0),
            PrintPath::new(
                PrintPathRole::Brim,
                vec![
                    Point2::new(-2.0, -2.0),
                    Point2::new(2.0, -2.0),
                    Point2::new(2.0, 2.0),
                    Point2::new(-2.0, 2.0),
                ],
            )
            .unwrap()
            .with_seam_gap_mm(1.0),
        ],
    )];

    let output = generate_toolpath_moves(&paths);

    assert_eq!(output[0].moves()[4].point(), Point2::new(-1.0, -1.0));
    assert_eq!(output[0].moves()[9].point(), Point2::new(-2.0, -2.0));
}

#[test]
fn positive_seam_gap_does_not_clip_open_sparse_infill() {
    let paths = [LayerPrintPaths::new(
        0,
        0.2,
        vec![
            PrintPath::new(
                PrintPathRole::SparseInfill,
                vec![Point2::new(0.5, 0.0), Point2::new(0.5, 1.0)],
            )
            .unwrap()
            .with_seam_gap_mm(1.0),
        ],
    )];

    let output = generate_toolpath_moves(&paths);

    assert_eq!(output[0].moves().len(), 2);
    assert_eq!(output[0].moves()[1].point(), Point2::new(0.5, 1.0));
}

#[test]
fn open_external_perimeter_does_not_emit_closing_move() {
    let layers = [LayerPrintPaths::new(
        0,
        0.2,
        vec![
            PrintPath::new(
                PrintPathRole::ExternalPerimeter,
                vec![Point2::new(0.4, 0.35), Point2::new(2.6, 0.35)],
            )
            .unwrap()
            .with_closed(false),
        ],
    )];

    let output = generate_toolpath_moves(&layers);

    assert_eq!(output[0].moves().len(), 2);
    assert_eq!(output[0].moves()[0].kind(), ToolpathMoveKind::Travel);
    assert_eq!(output[0].moves()[1].kind(), ToolpathMoveKind::Print);
    assert_eq!(output[0].moves()[1].point(), Point2::new(2.6, 0.35));
}
