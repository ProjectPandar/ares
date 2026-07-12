use super::{GCodeWriter, Point2, append_comment, format_xyzf};

pub(crate) struct SpiralLiftCommand<'a> {
    pub(crate) start: Point2,
    pub(crate) z_start: f64,
    pub(crate) z: f64,
    pub(crate) slope_radians: f64,
    pub(crate) resolution: f64,
    pub(crate) target: Point2,
    pub(crate) feedrate: f64,
    pub(crate) comment: Option<&'a str>,
}

impl GCodeWriter {
    pub(crate) fn travel_to_z_with_comment(
        &mut self,
        z: f64,
        feedrate: f64,
        comment: Option<&str>,
    ) -> String {
        self.current_position.2 = z;
        self.current_feedrate = feedrate;
        append_comment(
            format!("G1 Z{} F{}\n", format_xyzf(z), format_xyzf(feedrate)),
            comment,
        )
    }

    pub(crate) fn travel_to_xy_with_comment(
        &mut self,
        point: Point2,
        feedrate: f64,
        comment: Option<&str>,
    ) -> String {
        let (x, y) = self.offset_xy(point);
        self.current_position.0 = point.x();
        self.current_position.1 = point.y();
        self.current_feedrate = feedrate;
        append_comment(
            format!(
                "G1 X{} Y{} F{}\n",
                format_xyzf(x),
                format_xyzf(y),
                format_xyzf(feedrate)
            ),
            comment,
        )
    }

    pub(crate) fn travel_to_xyz_with_comment(
        &mut self,
        point: Point2,
        z: f64,
        feedrate: f64,
        comment: Option<&str>,
    ) -> String {
        let (x, y) = self.offset_xy(point);
        self.current_position = (point.x(), point.y(), z);
        self.current_feedrate = feedrate;
        append_comment(
            format!(
                "G1 X{} Y{} Z{} F{}\n",
                format_xyzf(x),
                format_xyzf(y),
                format_xyzf(z),
                format_xyzf(feedrate)
            ),
            comment,
        )
    }

    pub(crate) fn spiral_lift_with_comment(&mut self, command: SpiralLiftCommand<'_>) -> String {
        let SpiralLiftCommand {
            start,
            z_start,
            z,
            slope_radians,
            resolution,
            target,
            feedrate,
            comment,
        } = command;
        let dx = target.x() - start.x();
        let dy = target.y() - start.y();
        let travel_distance = dx.hypot(dy);
        let dx = dx / travel_distance;
        let dy = dy / travel_distance;
        let radius = (z - z_start) / (std::f64::consts::TAU * slope_radians.atan());
        let center_x = start.x() - dy * radius;
        let center_y = start.y() + dx * radius;
        let start_angle = (start.y() - center_y).atan2(start.x() - center_x);
        let segment_count = spiral_lift_segment_count(resolution);
        let mut gcode = String::new();
        if let Some(comment) = comment.filter(|comment| !comment.is_empty()) {
            gcode.push(';');
            gcode.push_str(comment);
            gcode.push('\n');
        }
        gcode.push_str(&format!("G1 F{}\n", format_xyzf(feedrate)));
        for segment in 1..=segment_count {
            let progress = f64::from(segment) / f64::from(segment_count);
            let angle = start_angle + std::f64::consts::TAU * progress;
            let point = Point2::new(
                center_x + radius * angle.cos(),
                center_y + radius * angle.sin(),
            );
            let (x, y) = self.offset_xy(point);
            let segment_z = z_start + (z - z_start) * progress;
            gcode.push_str(&format!(
                "G1 X{} Y{} Z{}\n",
                format_xyzf(x),
                format_xyzf(y),
                format_xyzf(segment_z)
            ));
        }
        self.current_position = (start.x(), start.y(), z);
        self.current_feedrate = feedrate;
        gcode
    }
}

fn spiral_lift_segment_count(resolution: f64) -> u32 {
    (16.0 * (0.01 / resolution)).round().clamp(4.0, 24.0) as u32
}
