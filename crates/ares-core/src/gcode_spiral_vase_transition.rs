use crate::{
    LayerExtrusionMoves, Point2, PrintPathRole, SpeedMove, ToolpathMoveKind,
    gcode_format::format_decimal, gcode_writer::GCodeWriter,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SpiralVaseTransitionOutState {
    finishing_flow_ratio: f64,
    total_xy: f64,
    printed_xy: f64,
    previous_point: Option<Point2>,
    moves: Vec<TransitionOutMove>,
    enabled: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TransitionOutMoveCommand {
    pub(crate) kind: ToolpathMoveKind,
    pub(crate) role: PrintPathRole,
    pub(crate) point: Point2,
    pub(crate) speed_move: SpeedMove,
    pub(crate) emitted_e_delta: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct TransitionOutMove {
    role: PrintPathRole,
    point: Point2,
    feedrate: f64,
    acceleration: Option<f64>,
    jerk: Option<f64>,
    e_delta: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct TransitionOutComments<'a> {
    speed: Option<&'a str>,
    acceleration: Option<&'a str>,
    jerk: Option<&'a str>,
    extrude: Option<&'a str>,
}

impl SpiralVaseTransitionOutState {
    pub(crate) fn new(
        finishing_flow_ratio: f64,
        layer: &LayerExtrusionMoves,
        enabled: bool,
    ) -> Self {
        if !enabled {
            return Self::disabled();
        }
        let total_xy = total_print_xy(layer);
        if total_xy <= f64::EPSILON {
            return Self::disabled();
        }
        Self {
            finishing_flow_ratio,
            total_xy,
            printed_xy: 0.0,
            previous_point: None,
            moves: Vec::new(),
            enabled: true,
        }
    }

    pub(crate) fn disabled() -> Self {
        Self {
            finishing_flow_ratio: 1.0,
            total_xy: 0.0,
            printed_xy: 0.0,
            previous_point: None,
            moves: Vec::new(),
            enabled: false,
        }
    }

    pub(crate) fn observe_move(&mut self, command: TransitionOutMoveCommand) {
        let previous = self.previous_point;
        self.previous_point = Some(command.point);
        if !self.enabled || command.kind != ToolpathMoveKind::Print {
            return;
        }
        let distance = previous.map_or(0.0, |start| distance(start, command.point));
        self.printed_xy += distance;
        let progress = (self.printed_xy / self.total_xy).clamp(0.0, 1.0);
        let factor = finishing_factor(progress, self.finishing_flow_ratio);
        self.moves.push(TransitionOutMove {
            role: command.role,
            point: command.point,
            feedrate: command.speed_move.feedrate_mm_min(),
            acceleration: command.speed_move.acceleration_mm_s2(),
            jerk: command.speed_move.jerk_mm_s(),
            e_delta: command.emitted_e_delta * factor,
        });
    }

    pub(crate) fn finish(&self, writer: &mut GCodeWriter, gcode_comments: bool) -> String {
        if !self.enabled {
            return String::new();
        }
        let comments = TransitionOutComments {
            speed: gcode_comments.then_some("set speed"),
            acceleration: gcode_comments.then_some("adjust acceleration"),
            jerk: gcode_comments.then_some("adjust jerk"),
            extrude: gcode_comments.then_some("extrude"),
        };
        let mut gcode = String::new();
        for move_ in &self.moves {
            append_move(&mut gcode, writer, *move_, comments);
        }
        gcode
    }
}

pub(super) fn finishing_factor(progress: f64, finishing_flow_ratio: f64) -> f64 {
    finishing_flow_ratio + (1.0 - progress) * (1.0 - finishing_flow_ratio)
}

fn append_move(
    gcode: &mut String,
    writer: &mut GCodeWriter,
    move_: TransitionOutMove,
    comments: TransitionOutComments<'_>,
) {
    let x = format_decimal(move_.point.x());
    let y = format_decimal(move_.point.y());
    let feedrate = format_decimal(move_.feedrate);
    let e_position = writer.current_e() + move_.e_delta;
    let e = format_decimal(e_position);
    gcode.push_str(&format!(
        ";SPEED:print:{}:{},{}:{}\n",
        move_.role.as_str(),
        x,
        y,
        feedrate
    ));
    let acceleration_command =
        writer.set_print_acceleration_with_comment(move_.acceleration, comments.acceleration);
    let jerk_command = writer.set_jerk_xy_with_comment(move_.jerk, comments.jerk);
    let move_command = crate::gcode_print_move::print_move_command(
        writer,
        crate::gcode_print_move::PrintMoveCommand::new(
            move_.point,
            e_position,
            move_.feedrate,
            comments.speed,
            comments.extrude,
        ),
    );
    gcode.push_str(&format!(
        ";EXTRUSION:print:{}:{},{}:{}\n;MOVE:print:{}:{},{}\n{}{}{}",
        move_.role.as_str(),
        x,
        y,
        e,
        move_.role.as_str(),
        x,
        y,
        acceleration_command,
        jerk_command,
        move_command
    ));
}

fn total_print_xy(layer: &LayerExtrusionMoves) -> f64 {
    let mut previous = None;
    let mut total = 0.0;
    for move_ in layer.moves() {
        let point = move_.point();
        if move_.kind() == ToolpathMoveKind::Print
            && let Some(start) = previous
        {
            total += distance(start, point);
        }
        previous = Some(point);
    }
    total
}

fn distance(start: Point2, end: Point2) -> f64 {
    ((end.x() - start.x()).powi(2) + (end.y() - start.y()).powi(2)).sqrt()
}
