use super::*;
use crate::SpeedMoveKinematics;

#[test]
fn emits_two_no_extrusion_moves_to_nearest_internal_segment_and_returns_to_start() {
    let mut writer = GCodeWriter::new();
    writer.travel_to_xy_with_comment(Point2::new(0.0, 0.0), 7200.0, None);
    let layer_print_paths = layer_print_paths(vec![internal_path(vec![
        Point2::new(0.4, 0.4),
        Point2::new(3.6, 0.4),
    ])]);
    let toolpath_moves = [
        ToolpathMove::new(
            ToolpathMoveKind::Travel,
            PrintPathRole::ExternalPerimeter,
            Point2::new(0.0, 0.0),
        ),
        ToolpathMove::new(
            ToolpathMoveKind::Print,
            PrintPathRole::ExternalPerimeter,
            Point2::new(4.0, 0.0),
        ),
    ];
    let speed_move = speed_move(4.0, 0.0, Some(0.4));
    let output = loop_wipe(
        true,
        &layer_print_paths,
        &toolpath_moves,
        Some("wipe before external loop"),
    )
    .gcode(&mut writer, 1, &speed_move);

    assert_eq!(
        output,
        "G1 X0.141 Y0.141 F3600 ; wipe before external loop\nG1 X0 Y0 F3600 ; wipe before external loop\n"
    );
    assert_eq!(writer.current_position(), (0.0, 0.0, 0.0));
    assert_eq!(writer.current_e(), 0.0);
    assert_eq!(writer.current_feedrate(), 3600.0);
}

#[test]
fn skips_when_disabled_or_not_first_external_print_move() {
    let mut writer = GCodeWriter::new();
    writer.travel_to_xy_with_comment(Point2::new(0.0, 0.0), 7200.0, None);
    let layer_print_paths = layer_print_paths(vec![internal_path(vec![
        Point2::new(0.4, 0.4),
        Point2::new(3.6, 0.4),
    ])]);
    let moves = [
        ToolpathMove::new(
            ToolpathMoveKind::Travel,
            PrintPathRole::ExternalPerimeter,
            Point2::new(1.0, 0.0),
        ),
        ToolpathMove::new(
            ToolpathMoveKind::Print,
            PrintPathRole::ExternalPerimeter,
            Point2::new(4.0, 0.0),
        ),
    ];
    let speed_move = speed_move(4.0, 0.0, Some(0.4));
    assert_eq!(
        loop_wipe(false, &layer_print_paths, &moves, None).gcode(&mut writer, 1, &speed_move),
        ""
    );
    assert_eq!(
        loop_wipe(true, &layer_print_paths, &moves, None).gcode(&mut writer, 1, &speed_move),
        ""
    );
}

#[test]
fn skips_without_internal_segment_coincident_projection_or_effective_width() {
    let mut writer = GCodeWriter::new();
    writer.travel_to_xy_with_comment(Point2::new(0.0, 0.0), 7200.0, None);
    let moves = [
        ToolpathMove::new(
            ToolpathMoveKind::Travel,
            PrintPathRole::ExternalPerimeter,
            Point2::new(0.0, 0.0),
        ),
        ToolpathMove::new(
            ToolpathMoveKind::Print,
            PrintPathRole::ExternalPerimeter,
            Point2::new(4.0, 0.0),
        ),
    ];
    let empty_paths = layer_print_paths(Vec::new());
    let coincident_paths = layer_print_paths(vec![internal_path(vec![
        Point2::new(0.0, 0.0),
        Point2::new(3.6, 0.0),
    ])]);
    let internal_paths = layer_print_paths(vec![internal_path(vec![
        Point2::new(0.4, 0.4),
        Point2::new(3.6, 0.4),
    ])]);
    let speed_move_with_width = speed_move(4.0, 0.0, Some(0.4));
    let speed_move_without_width = speed_move(4.0, 0.0, None);
    assert_eq!(
        loop_wipe(true, &empty_paths, &moves, None).gcode(
            &mut writer,
            1,
            &speed_move_with_width
        ),
        ""
    );
    assert_eq!(
        loop_wipe(true, &coincident_paths, &moves, None).gcode(
            &mut writer,
            1,
            &speed_move_with_width
        ),
        ""
    );
    assert_eq!(
        loop_wipe(true, &internal_paths, &moves, None).gcode(
            &mut writer,
            1,
            &speed_move_without_width
        ),
        ""
    );
}

#[test]
fn uses_closing_segment_of_closed_internal_perimeter() {
    let mut writer = GCodeWriter::new();
    writer.travel_to_xy_with_comment(Point2::new(0.0, 0.0), 7200.0, None);
    let layer_print_paths = layer_print_paths(vec![internal_path(vec![
        Point2::new(2.0, -1.0),
        Point2::new(4.0, -1.0),
        Point2::new(4.0, 1.0),
        Point2::new(2.0, 1.0),
    ])]);
    let moves = [
        ToolpathMove::new(
            ToolpathMoveKind::Travel,
            PrintPathRole::ExternalPerimeter,
            Point2::new(0.0, 0.0),
        ),
        ToolpathMove::new(
            ToolpathMoveKind::Print,
            PrintPathRole::ExternalPerimeter,
            Point2::new(4.0, 0.0),
        ),
    ];
    let output = loop_wipe(true, &layer_print_paths, &moves, None).gcode(
        &mut writer,
        1,
        &speed_move(4.0, 0.0, Some(0.4)),
    );

    assert_eq!(output, "G1 X0.2 Y0 F3600\nG1 X0 Y0 F3600\n");
}

fn layer_print_paths(paths: Vec<PrintPath>) -> LayerPrintPaths {
    LayerPrintPaths::new(0, 0.2, paths)
}

fn loop_wipe<'a>(
    enabled: bool,
    layer_print_paths: &'a LayerPrintPaths,
    toolpath_moves: &'a [ToolpathMove],
    comment: Option<&'a str>,
) -> WipeBeforeExternalLoop<'a> {
    WipeBeforeExternalLoop::new(enabled, layer_print_paths, toolpath_moves, comment)
}

fn internal_path(points: Vec<Point2>) -> PrintPath {
    PrintPath::new(PrintPathRole::InternalPerimeter, points).unwrap()
}

fn speed_move(x: f64, y: f64, width: Option<f64>) -> SpeedMove {
    SpeedMove::new(
        ToolpathMoveKind::Print,
        PrintPathRole::ExternalPerimeter,
        Point2::new(x, y),
        Some(0.0),
        SpeedMoveKinematics::new(60.0, None, None),
    )
    .with_effective_line_width_mm(width)
}
