use crate::Point2;
use crate::gcode_writer::GCodeWriter;

pub(crate) struct PrintMoveCommand<'a> {
    pub(crate) point: Point2,
    pub(crate) e_position: f64,
    pub(crate) feedrate: f64,
    pub(crate) speed_comment: Option<&'a str>,
    pub(crate) extrude_comment: Option<&'a str>,
}

impl<'a> PrintMoveCommand<'a> {
    pub(crate) const fn new(
        point: Point2,
        e_position: f64,
        feedrate: f64,
        speed_comment: Option<&'a str>,
        extrude_comment: Option<&'a str>,
    ) -> Self {
        Self {
            point,
            e_position,
            feedrate,
            speed_comment,
            extrude_comment,
        }
    }
}

pub(crate) fn print_move_command(
    writer: &mut GCodeWriter,
    move_command: PrintMoveCommand<'_>,
) -> String {
    let mut command = String::new();
    if writer.current_feedrate() != move_command.feedrate {
        command.push_str(
            &writer.set_speed_with_comment(move_command.feedrate, move_command.speed_comment),
        );
    }
    command.push_str(&writer.extrude_to_xy_with_comment(
        move_command.point,
        move_command.e_position - writer.current_e(),
        move_command.extrude_comment,
    ));
    command
}
