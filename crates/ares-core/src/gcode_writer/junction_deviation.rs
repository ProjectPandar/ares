use super::GCodeWriter;

impl GCodeWriter {
    pub(crate) fn set_junction_deviation(
        &self,
        junction_deviation: f64,
        max_junction_deviation: f64,
        comments_enabled: bool,
    ) -> String {
        crate::gcode_junction_deviation::junction_deviation_command(
            self.gcode_flavor,
            junction_deviation,
            max_junction_deviation,
            comments_enabled,
        )
    }
}
