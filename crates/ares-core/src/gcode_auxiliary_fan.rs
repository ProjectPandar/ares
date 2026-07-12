use crate::gcode_writer::GCodeWriter;
use crate::options::GCodeFlavor;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AuxiliaryFanState {
    current_speed: Option<u8>,
}

impl AuxiliaryFanState {
    pub(crate) const fn new() -> Self {
        Self {
            current_speed: None,
        }
    }
}

pub(crate) fn layer_command(
    writer: &GCodeWriter,
    gcode_flavor: GCodeFlavor,
    speed: Option<u8>,
    state: &mut AuxiliaryFanState,
) -> String {
    if gcode_flavor == GCodeFlavor::Klipper {
        return String::new();
    }
    let Some(speed) = speed.filter(|speed| state.current_speed != Some(*speed)) else {
        return String::new();
    };
    state.current_speed = Some(speed);
    writer.set_additional_fan(speed)
}

pub(crate) fn completion_command(
    writer: &GCodeWriter,
    gcode_flavor: GCodeFlavor,
    should_shutdown: bool,
    state: AuxiliaryFanState,
) -> String {
    if gcode_flavor == GCodeFlavor::Klipper || state.current_speed.is_none() || !should_shutdown {
        return String::new();
    }
    writer.set_additional_fan(0)
}
