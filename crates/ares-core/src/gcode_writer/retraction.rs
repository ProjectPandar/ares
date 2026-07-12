use super::{ExtrusionAxisMode, GCodeWriter, Point2, append_comment, format_e, format_xyzf};

impl GCodeWriter {
    pub(crate) fn firmware_retract(&mut self) -> String {
        if self.gcode_flavor == crate::options::GCodeFlavor::Machinekit {
            "G22 ; retract\n".to_owned()
        } else {
            "G10 ; retract\n".to_owned()
        }
    }

    pub(crate) fn firmware_unretract(&mut self) -> String {
        if self.gcode_flavor == crate::options::GCodeFlavor::Machinekit {
            "G23 ; unretract\n".to_owned()
        } else {
            "G11 ; unretract\n".to_owned()
        }
    }

    pub(crate) fn retract_with_comment(
        &mut self,
        length: f64,
        feedrate: f64,
        comment: Option<&str>,
    ) -> String {
        self.extrude_e_only_with_comment(-length, feedrate, comment)
    }

    pub(crate) fn unretract_with_comment(
        &mut self,
        length: f64,
        feedrate: f64,
        comment: Option<&str>,
    ) -> String {
        self.extrude_e_only_with_comment(length, feedrate, comment)
    }

    pub(crate) fn extrude_to_xy_with_feedrate_and_comment(
        &mut self,
        point: Point2,
        delta_e: f64,
        feedrate: f64,
        comment: Option<&str>,
    ) -> String {
        let (x, y) = self.offset_xy(point);
        self.current_position.0 = point.x();
        self.current_position.1 = point.y();
        self.current_feedrate = feedrate;
        if delta_e.abs() <= f64::EPSILON {
            append_comment(
                format!(
                    "G1 X{} Y{} F{}\n",
                    format_xyzf(x),
                    format_xyzf(y),
                    format_xyzf(feedrate)
                ),
                comment,
            )
        } else {
            self.current_e += delta_e;
            let emitted_e = match self.extrusion_axis_mode {
                ExtrusionAxisMode::Relative => delta_e,
                ExtrusionAxisMode::Absolute => self.current_e,
            };
            append_comment(
                format!(
                    "G1 X{} Y{} E{} F{}\n",
                    format_xyzf(x),
                    format_xyzf(y),
                    format_e(emitted_e),
                    format_xyzf(feedrate)
                ),
                comment,
            )
        }
    }

    fn extrude_e_only_with_comment(
        &mut self,
        delta_e: f64,
        feedrate: f64,
        comment: Option<&str>,
    ) -> String {
        self.current_e += delta_e;
        self.current_feedrate = feedrate;
        let emitted_e = match self.extrusion_axis_mode {
            ExtrusionAxisMode::Relative => delta_e,
            ExtrusionAxisMode::Absolute => self.current_e,
        };
        append_comment(
            format!("G1 E{} F{}\n", format_e(emitted_e), format_xyzf(feedrate)),
            comment,
        )
    }
}
