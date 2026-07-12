use crate::{
    ToolpathMoveKind,
    gcode_writer::GCodeWriter,
    options::{AccelToDecelConfig, GCodeFlavor},
};

impl GCodeWriter {
    pub(crate) fn set_accel_to_decel_config(&mut self, config: AccelToDecelConfig) {
        self.accel_to_decel_config = config;
    }

    pub(crate) fn set_print_acceleration_with_comment(
        &mut self,
        acceleration: Option<f64>,
        comment: Option<&str>,
    ) -> String {
        self.set_move_acceleration_with_comment(ToolpathMoveKind::Print, acceleration, comment)
    }

    pub(crate) fn set_travel_acceleration_with_comment(
        &mut self,
        acceleration: Option<f64>,
        comment: Option<&str>,
    ) -> String {
        self.set_move_acceleration_with_comment(ToolpathMoveKind::Travel, acceleration, comment)
    }

    fn set_move_acceleration_with_comment(
        &mut self,
        kind: ToolpathMoveKind,
        acceleration: Option<f64>,
        comment: Option<&str>,
    ) -> String {
        let Some(acceleration) = acceleration else {
            return String::new();
        };
        let acceleration = (acceleration + 0.5).floor() as u32;
        let current_acceleration = self.current_acceleration_mut(kind);
        if acceleration == 0 || acceleration == *current_acceleration {
            return String::new();
        }
        *current_acceleration = acceleration;
        super::append_comment(self.acceleration_command(kind, acceleration), comment)
    }

    fn current_acceleration_mut(&mut self, kind: ToolpathMoveKind) -> &mut u32 {
        if self.gcode_flavor.supports_separate_travel_acceleration()
            && kind == ToolpathMoveKind::Travel
        {
            &mut self.current_travel_acceleration
        } else {
            &mut self.current_print_acceleration
        }
    }

    fn acceleration_command(&self, kind: ToolpathMoveKind, acceleration: u32) -> String {
        match self.gcode_flavor {
            GCodeFlavor::Klipper => self.klipper_acceleration_command(acceleration),
            GCodeFlavor::Repetier => {
                let command = if kind == ToolpathMoveKind::Travel
                    && self.gcode_flavor.supports_separate_travel_acceleration()
                {
                    "M202"
                } else {
                    "M201"
                };
                format!("{command} X{acceleration} Y{acceleration}\n")
            }
            GCodeFlavor::RepRapFirmware | GCodeFlavor::MarlinFirmware => {
                let axis = if kind == ToolpathMoveKind::Travel
                    && self.gcode_flavor.supports_separate_travel_acceleration()
                {
                    "T"
                } else {
                    "P"
                };
                format!("M204 {axis}{acceleration}\n")
            }
            _ => format!("M204 S{acceleration}\n"),
        }
    }

    fn klipper_acceleration_command(&self, acceleration: u32) -> String {
        let mut command = format!("SET_VELOCITY_LIMIT ACCEL={acceleration}");
        if self.accel_to_decel_config.enabled() {
            let accel_to_decel = (f64::from(acceleration)
                * self.accel_to_decel_config.factor_percent()
                / 100.0) as u32;
            command.push_str(&format!(" ACCEL_TO_DECEL={accel_to_decel}"));
        }
        command.push('\n');
        command
    }
}
