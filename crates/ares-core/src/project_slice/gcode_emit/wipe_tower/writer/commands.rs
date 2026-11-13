use super::Writer;
use crate::GCodeFlavor;
use std::fmt::Write;

impl Writer {
    pub(in super::super) fn flush_planner_queue_command(flavor: GCodeFlavor) -> &'static str {
        if flavor == GCodeFlavor::Klipper {
            "M400\n"
        } else {
            "G4 S0\n"
        }
    }

    pub(in super::super) fn flush_planner_queue(gcode: &mut String, flavor: GCodeFlavor) {
        if flavor != GCodeFlavor::Klipper {
            gcode.push_str(Self::flush_planner_queue_command(flavor));
        }
    }

    pub(super) fn format_feedrate(&mut self, feedrate: f32) {
        write!(self.gcode, " F{}", (feedrate + 0.5).floor() as i32).unwrap();
        self.feedrate = feedrate;
    }

    pub(in super::super) fn feedrate(&mut self, feedrate: f32) {
        if feedrate != self.feedrate {
            self.gcode.push_str("G1");
            self.format_feedrate(feedrate);
            self.gcode.push('\n');
        }
    }

    pub(in super::super) fn load(&mut self, amount: f32, feedrate: f32) {
        if amount == 0.0 && (feedrate == 0.0 || feedrate == self.feedrate) {
            return;
        }
        self.gcode.push_str("G1");
        if amount != 0.0 {
            write!(self.gcode, " E{amount:.4}").unwrap();
        }
        if feedrate != 0.0 && feedrate != self.feedrate {
            self.format_feedrate(feedrate);
        }
        self.gcode.push('\n');
    }

    pub(in super::super) fn disable_linear_advance(&mut self) {
        match self.flavor {
            GCodeFlavor::RepRapSprinter | GCodeFlavor::RepRapFirmware => {
                writeln!(self.gcode, "M572 D{} S0", self.tool).unwrap()
            }
            GCodeFlavor::Klipper => self.gcode.push_str("SET_PRESSURE_ADVANCE ADVANCE=0\n"),
            _ => self.gcode.push_str("M900 K0\n"),
        }
    }

    pub(in super::super) fn switch_filament_monitoring(&mut self, enabled: bool) {
        self.gcode
            .push_str(Self::flush_planner_queue_command(self.flavor));
        self.gcode
            .push_str(if enabled { "M591 R\n" } else { "M591 S0\n" });
    }

    pub(in super::super) fn set_temperature(&mut self, temperature: i32, wait: bool, sync: bool) {
        if sync {
            self.gcode
                .push_str(Self::flush_planner_queue_command(self.flavor));
        }
        writeln!(
            self.gcode,
            "M{} S{temperature}",
            if wait { 109 } else { 104 }
        )
        .unwrap();
    }

    pub(in super::super) fn wait(&mut self, seconds: f32) {
        if seconds != 0.0 {
            writeln!(self.gcode, "G4 S{seconds:.3}").unwrap();
        }
    }

    pub(in super::super) fn speed_override(&mut self, speed: i32) {
        writeln!(self.gcode, "M220 S{speed}").unwrap();
    }

    pub(in super::super) fn speed_override_backup(&mut self) {
        if matches!(
            self.flavor,
            GCodeFlavor::MarlinLegacy | GCodeFlavor::MarlinFirmware
        ) {
            self.gcode.push_str("M220 B\n");
        }
    }

    pub(in super::super) fn speed_override_restore(&mut self) {
        if matches!(
            self.flavor,
            GCodeFlavor::MarlinLegacy | GCodeFlavor::MarlinFirmware
        ) {
            self.gcode.push_str("M220 R\n");
        }
    }

    pub(in super::super) fn trimpot(&mut self, current: i32) {
        match self.flavor {
            GCodeFlavor::Klipper => {}
            GCodeFlavor::RepRapSprinter | GCodeFlavor::RepRapFirmware => {
                writeln!(self.gcode, "M906 E{current}").unwrap()
            }
            _ => writeln!(self.gcode, "M907 E{current}").unwrap(),
        }
    }
}
