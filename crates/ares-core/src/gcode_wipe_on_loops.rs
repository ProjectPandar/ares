use crate::{
    LayerPrintPaths, Point2, PrintPath, PrintPathRole, SpeedMove, ToolpathMove, ToolpathMoveKind,
    gcode_writer::GCodeWriter,
};

pub(crate) struct WipeOnLoops<'a> {
    enabled: bool,
    wall_loops: u32,
    nozzle_diameter: f64,
    layer_print_paths: &'a LayerPrintPaths,
    toolpath_moves: &'a [ToolpathMove],
    comment: Option<&'a str>,
}

pub(crate) struct WipeOnLoopsCommand<'a> {
    pub(crate) enabled: bool,
    pub(crate) wall_loops: u32,
    pub(crate) nozzle_diameter: f64,
    pub(crate) layer_print_paths: &'a LayerPrintPaths,
    pub(crate) toolpath_moves: &'a [ToolpathMove],
    pub(crate) comment: Option<&'a str>,
}

impl<'a> WipeOnLoops<'a> {
    pub(crate) const fn new(command: WipeOnLoopsCommand<'a>) -> Self {
        Self {
            enabled: command.enabled,
            wall_loops: command.wall_loops,
            nozzle_diameter: command.nozzle_diameter,
            layer_print_paths: command.layer_print_paths,
            toolpath_moves: command.toolpath_moves,
            comment: command.comment,
        }
    }

    pub(crate) fn gcode(
        &self,
        writer: &mut GCodeWriter,
        move_index: usize,
        speed_move: &SpeedMove,
    ) -> String {
        if !self.enabled || self.wall_loops <= 1 {
            return String::new();
        }
        let Some(target) = self.wipe_target(move_index) else {
            return String::new();
        };
        writer.extrude_to_xy_with_feedrate_and_comment(
            target,
            0.0,
            speed_move.feedrate_mm_min(),
            self.comment,
        )
    }

    fn wipe_target(&self, move_index: usize) -> Option<Point2> {
        let move_ = self.toolpath_moves.get(move_index)?;
        if move_.kind() != ToolpathMoveKind::Print
            || move_.role() != PrintPathRole::ExternalPerimeter
        {
            return None;
        }
        let path = external_path_closed_by_move(self.layer_print_paths.paths(), move_.point())?;
        local_inward_point(path.points(), self.nozzle_diameter)
    }
}

fn external_path_closed_by_move(paths: &[PrintPath], point: Point2) -> Option<&PrintPath> {
    paths.iter().find(|path| {
        path.role() == PrintPathRole::ExternalPerimeter
            && path.is_closed()
            && path.points().len() >= 3
            && closing_target(path.points(), path.seam_gap_mm()) == Some(point)
    })
}

fn closing_target(points: &[Point2], seam_gap_mm: f64) -> Option<Point2> {
    let start = points[0];
    let end = *points.last()?;
    let length = distance(end, start);
    if length <= f64::EPSILON {
        return None;
    }
    if seam_gap_mm <= 0.0 {
        return Some(start);
    }
    if seam_gap_mm >= length {
        return None;
    }
    let ratio = (length - seam_gap_mm) / length;
    Some(Point2::new(
        end.x() + (start.x() - end.x()) * ratio,
        end.y() + (start.y() - end.y()) * ratio,
    ))
}

fn local_inward_point(points: &[Point2], nozzle_diameter: f64) -> Option<Point2> {
    if !nozzle_diameter.is_finite() || nozzle_diameter <= 0.0 {
        return None;
    }
    let start = points[0];
    let next = points[1];
    let previous = *points.last()?;
    let first = vector(start, next);
    let incoming = vector(previous, start);
    let first_length = length(first);
    if first_length <= f64::EPSILON || length(incoming) <= f64::EPSILON {
        return None;
    }
    let along = 0.2 * first_length.min(nozzle_diameter);
    let local = (
        first.0 / first_length * along,
        first.1 / first_length * along,
    );
    let rotated = rotate(local, signed_angle(incoming, first) / 3.0);
    Some(Point2::new(start.x() + rotated.0, start.y() + rotated.1))
}

fn vector(from: Point2, to: Point2) -> (f64, f64) {
    (to.x() - from.x(), to.y() - from.y())
}

fn distance(a: Point2, b: Point2) -> f64 {
    length(vector(a, b))
}

fn length(vector: (f64, f64)) -> f64 {
    (vector.0 * vector.0 + vector.1 * vector.1).sqrt()
}

fn signed_angle(a: (f64, f64), b: (f64, f64)) -> f64 {
    (a.0 * b.1 - a.1 * b.0).atan2(a.0 * b.0 + a.1 * b.1)
}

fn rotate(vector: (f64, f64), angle: f64) -> (f64, f64) {
    let (sin, cos) = angle.sin_cos();
    (
        vector.0 * cos - vector.1 * sin,
        vector.0 * sin + vector.1 * cos,
    )
}
