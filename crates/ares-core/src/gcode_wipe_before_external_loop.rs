use crate::{
    LayerPrintPaths, Point2, PrintPath, PrintPathRole, SpeedMove, ToolpathMove, ToolpathMoveKind,
    gcode_writer::GCodeWriter,
};

pub(crate) struct WipeBeforeExternalLoop<'a> {
    enabled: bool,
    layer_print_paths: &'a LayerPrintPaths,
    toolpath_moves: &'a [ToolpathMove],
    comment: Option<&'a str>,
}

impl<'a> WipeBeforeExternalLoop<'a> {
    pub(crate) const fn new(
        enabled: bool,
        layer_print_paths: &'a LayerPrintPaths,
        toolpath_moves: &'a [ToolpathMove],
        comment: Option<&'a str>,
    ) -> Self {
        Self {
            enabled,
            layer_print_paths,
            toolpath_moves,
            comment,
        }
    }

    pub(crate) fn gcode(
        &self,
        writer: &mut GCodeWriter,
        move_index: usize,
        speed_move: &SpeedMove,
    ) -> String {
        let start = Point2::new(writer.current_position().0, writer.current_position().1);
        if !self.enabled || !is_first_external_print_move(self.toolpath_moves, move_index, start) {
            return String::new();
        }
        let Some(width) = speed_move.effective_line_width_mm() else {
            return String::new();
        };
        let Some(target) = wipe_target(start, width, self.layer_print_paths.paths()) else {
            return String::new();
        };
        let feedrate = speed_move.feedrate_mm_min();
        let mut gcode =
            writer.extrude_to_xy_with_feedrate_and_comment(target, 0.0, feedrate, self.comment);
        gcode.push_str(&writer.extrude_to_xy_with_feedrate_and_comment(
            start,
            0.0,
            feedrate,
            self.comment,
        ));
        gcode
    }
}

fn is_first_external_print_move(moves: &[ToolpathMove], index: usize, start: Point2) -> bool {
    let Some(current) = moves.get(index) else {
        return false;
    };
    if current.kind() != ToolpathMoveKind::Print
        || current.role() != PrintPathRole::ExternalPerimeter
    {
        return false;
    }
    let Some(previous) = index.checked_sub(1).and_then(|index| moves.get(index)) else {
        return false;
    };
    previous.kind() == ToolpathMoveKind::Travel
        && previous.role() == PrintPathRole::ExternalPerimeter
        && previous.point() == start
}

fn wipe_target(start: Point2, external_width: f64, paths: &[PrintPath]) -> Option<Point2> {
    if !external_width.is_finite() || external_width <= 0.0 {
        return None;
    }
    let nearest = paths
        .iter()
        .filter(|path| path.role() == PrintPathRole::InternalPerimeter)
        .flat_map(internal_segments)
        .map(|(a, b)| project_to_segment(start, a, b))
        .min_by(|a, b| {
            distance(start, *a)
                .partial_cmp(&distance(start, *b))
                .unwrap_or(std::cmp::Ordering::Equal)
        })?;
    let distance_to_internal = distance(start, nearest);
    if distance_to_internal <= f64::EPSILON {
        return None;
    }
    let wipe_distance = distance_to_internal.min(external_width / 2.0);
    let ratio = wipe_distance / distance_to_internal;
    Some(Point2::new(
        start.x() + (nearest.x() - start.x()) * ratio,
        start.y() + (nearest.y() - start.y()) * ratio,
    ))
}

fn internal_segments(path: &PrintPath) -> impl Iterator<Item = (Point2, Point2)> + '_ {
    let points = path.points();
    points
        .windows(2)
        .map(|points| (points[0], points[1]))
        .chain(
            (path.is_closed() && points.len() > 1).then(|| (points[points.len() - 1], points[0])),
        )
}

fn project_to_segment(point: Point2, a: Point2, b: Point2) -> Point2 {
    let ab_x = b.x() - a.x();
    let ab_y = b.y() - a.y();
    let length_squared = ab_x * ab_x + ab_y * ab_y;
    if length_squared <= f64::EPSILON {
        return a;
    }
    let t = (((point.x() - a.x()) * ab_x + (point.y() - a.y()) * ab_y) / length_squared)
        .clamp(0.0, 1.0);
    Point2::new(a.x() + ab_x * t, a.y() + ab_y * t)
}

fn distance(a: Point2, b: Point2) -> f64 {
    ((a.x() - b.x()).powi(2) + (a.y() - b.y()).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
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
}
