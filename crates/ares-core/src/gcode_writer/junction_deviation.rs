use super::GCodeWriter;
use crate::options::GCodeFlavor;

impl GCodeWriter {
    pub(crate) fn set_junction_deviation(
        &self,
        junction_deviation: f64,
        max_junction_deviation: f64,
        comments_enabled: bool,
    ) -> String {
        if self.gcode_flavor != GCodeFlavor::MarlinFirmware
            || max_junction_deviation <= 0.0
            || junction_deviation <= 0.0
        {
            return String::new();
        }
        let value = junction_deviation.min(max_junction_deviation);
        let mut gcode = format!("M205 J{value:.3}\n");
        if comments_enabled {
            gcode.pop();
            gcode.push_str(" ; Junction Deviation\n");
        }
        gcode
    }
}
