use crate::{SpeedMove, gcode_role_fan::RoleFanGCodeState, gcode_writer::GCodeWriter};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BufferedMove {
    gcode: String,
    speed_move: SpeedMove,
}

impl BufferedMove {
    pub(crate) fn new(gcode: String, speed_move: SpeedMove) -> Self {
        Self { gcode, speed_move }
    }
}

pub(crate) fn flush(
    gcode: &mut String,
    writer: &GCodeWriter,
    role_fan_state: &mut RoleFanGCodeState,
    buffered_move: &mut Option<BufferedMove>,
) {
    let Some(buffered) = buffered_move.take() else {
        return;
    };
    gcode.push_str(&buffered.gcode);
    gcode.push_str(&role_fan_state.after_move(writer, &buffered.speed_move));
}
