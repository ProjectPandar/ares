// Source: OrcaSlicer/src/libslic3r/GCodeWriter.hpp
// Source: OrcaSlicer/src/libslic3r/GCodeWriter.cpp

mod acceleration;
mod junction_deviation;
mod retraction;
mod travel;

pub(crate) use travel::SpiralLiftCommand;

use crate::{
    Point2,
    options::{AccelToDecelConfig, GCodeFlavor},
};

pub(crate) const XYZF_EXPORT_DIGITS: usize = 3;
pub(crate) const E_EXPORT_DIGITS: usize = 5;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GCodeWriter {
    current_position: (f64, f64, f64),
    current_feedrate: f64,
    current_e: f64,
    current_print_acceleration: u32,
    current_travel_acceleration: u32,
    current_jerk: f64,
    extrusion_axis_mode: ExtrusionAxisMode,
    gcode_flavor: GCodeFlavor,
    accel_to_decel_config: AccelToDecelConfig,
    part_cooling_fan_min_pwm: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ExtrusionAxisMode {
    Relative,
    Absolute,
}

impl GCodeWriter {
    pub(crate) fn new() -> Self {
        Self {
            current_position: (0.0, 0.0, 0.0),
            current_feedrate: 0.0,
            current_e: 0.0,
            current_print_acceleration: 0,
            current_travel_acceleration: 0,
            current_jerk: 0.0,
            extrusion_axis_mode: ExtrusionAxisMode::Relative,
            gcode_flavor: GCodeFlavor::MarlinLegacy,
            accel_to_decel_config: AccelToDecelConfig::new(true, 50.0),
            part_cooling_fan_min_pwm: 0,
        }
    }

    pub(crate) fn set_gcode_flavor(&mut self, flavor: GCodeFlavor) {
        self.gcode_flavor = flavor;
    }

    pub(crate) fn set_extrusion_axis_mode(&mut self, mode: ExtrusionAxisMode) {
        self.extrusion_axis_mode = mode;
    }

    pub(crate) fn set_part_cooling_fan_min_pwm(&mut self, min_pwm: u8) {
        self.part_cooling_fan_min_pwm = min_pwm;
    }

    pub(crate) fn preamble(&mut self) -> String {
        let mut gcode = String::new();
        if self.gcode_flavor != GCodeFlavor::MakerWare {
            gcode.push_str("G90\nG21\n");
        }
        gcode.push_str(&self.extrusion_axis_mode_preamble());
        gcode
    }

    fn extrusion_axis_mode_preamble(&mut self) -> String {
        if !self.gcode_flavor.emits_extrusion_axis_mode() {
            return String::new();
        }
        match self.extrusion_axis_mode {
            ExtrusionAxisMode::Relative => {
                "M83 ; use relative distances for extrusion\n".to_owned()
            }
            ExtrusionAxisMode::Absolute => self.absolute_extrusion_axis_mode_preamble(),
        }
    }

    fn absolute_extrusion_axis_mode_preamble(&mut self) -> String {
        let mut gcode = "M82 ; use absolute distances for extrusion\n".to_owned();
        if self.gcode_flavor.resets_absolute_e() {
            self.reset_e();
            gcode.push_str("G92 E0\n");
        }
        gcode
    }

    pub(crate) fn set_speed_with_comment(
        &mut self,
        feedrate: f64,
        comment: Option<&str>,
    ) -> String {
        self.current_feedrate = feedrate;
        append_comment(format!("G1 F{}\n", format_xyzf(feedrate)), comment)
    }

    pub(crate) fn set_jerk_xy_with_comment(
        &mut self,
        jerk: Option<f64>,
        comment: Option<&str>,
    ) -> String {
        let Some(jerk) = jerk else {
            return String::new();
        };
        if jerk < 0.01 || jerk == self.current_jerk {
            return String::new();
        }
        self.current_jerk = jerk;
        append_comment(
            format!("M205 X{} Y{}\n", format_xyzf(jerk), format_xyzf(jerk)),
            comment,
        )
    }

    pub(crate) fn set_nozzle_temperature(
        &self,
        temperature: u32,
        wait: bool,
        tool: Option<u32>,
    ) -> String {
        if wait && self.gcode_flavor.skips_waiting_nozzle_temperature() {
            return String::new();
        }
        let code = self.gcode_flavor.nozzle_temperature_code(wait);
        let comment = if wait && code == "M109" {
            "set nozzle temperature and wait for it to be reached"
        } else {
            "set nozzle temperature"
        };
        let axis = if matches!(
            self.gcode_flavor,
            GCodeFlavor::Mach3 | GCodeFlavor::Machinekit
        ) {
            "P"
        } else {
            "S"
        };
        let mut gcode = format!("{code} {axis}{temperature}");
        if let Some(tool) = tool {
            if self.gcode_flavor == GCodeFlavor::RepRapFirmware {
                gcode.push_str(&format!(" P{tool}"));
            } else {
                gcode.push_str(&format!(" T{tool}"));
            }
        }
        gcode.push_str(&format!(" ; {comment}\n"));
        if self.gcode_flavor.waits_after_nozzle_temperature(wait) {
            gcode.push_str("M116 ; wait for temperature to be reached\n");
        }
        gcode
    }

    pub(crate) fn set_bed_temperature(&self, temperature: u32, wait: bool) -> String {
        let (code, comment) = if wait {
            ("M190", "set bed temperature and wait for it to be reached")
        } else {
            ("M140", "set bed temperature")
        };
        format!("{code} S{temperature} ; {comment}\n")
    }

    pub(crate) fn set_chamber_temperature(&self, temperature: u32, wait: bool) -> String {
        if wait {
            format!("M191 S{temperature} ;set chamber_temperature and wait for it to be reached\n")
        } else {
            format!("M141 S{temperature};set chamber_temperature\n")
        }
    }

    pub(crate) fn set_fan(&self, speed: u8) -> String {
        let speed = if speed > 0 && speed < self.part_cooling_fan_min_pwm {
            self.part_cooling_fan_min_pwm
        } else {
            speed
        };
        match (self.gcode_flavor, speed) {
            (GCodeFlavor::MakerWare | GCodeFlavor::Sailfish, 0) => "M127\n".to_owned(),
            (GCodeFlavor::MakerWare | GCodeFlavor::Sailfish, _) => "M126\n".to_owned(),
            (GCodeFlavor::Mach3 | GCodeFlavor::Machinekit, 0) => "M106 S0\n".to_owned(),
            (GCodeFlavor::Mach3 | GCodeFlavor::Machinekit, _) => {
                format!("M106 P{}\n", fan_pwm(speed))
            }
            _ => format!("M106 S{}\n", fan_pwm(speed)),
        }
    }

    pub(crate) fn set_exhaust_fan(&self, speed: u8) -> String {
        format!("M106 P3 S{}\n", exhaust_fan_pwm(speed))
    }

    pub(crate) fn set_additional_fan(&self, speed: u8) -> String {
        format!("M106 P2 S{}\n", additional_fan_pwm(speed))
    }

    pub(crate) fn set_pressure_advance(&self, pressure_advance: f64) -> String {
        if pressure_advance < 0.0 {
            return String::new();
        }
        let value = format_pressure_advance(pressure_advance);
        match self.gcode_flavor {
            GCodeFlavor::Klipper => {
                format!("SET_PRESSURE_ADVANCE ADVANCE={value}; Override pressure advance value\n")
            }
            GCodeFlavor::RepRapFirmware => {
                format!("M572 D0 S{value}; Override pressure advance value\n")
            }
            GCodeFlavor::Repetier => {
                format!("M233 X{value} Y{value} ; Override pressure advance value\n")
            }
            _ => format!("M900 K{value}; Override pressure advance value\n"),
        }
    }

    pub(crate) fn extrude_to_xy_with_comment(
        &mut self,
        point: Point2,
        delta_e: f64,
        comment: Option<&str>,
    ) -> String {
        self.current_position.0 = point.x();
        self.current_position.1 = point.y();
        if delta_e.abs() <= f64::EPSILON {
            append_comment(
                format!(
                    "G1 X{} Y{}\n",
                    format_xyzf(point.x()),
                    format_xyzf(point.y())
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
                    "G1 X{} Y{} E{}\n",
                    format_xyzf(point.x()),
                    format_xyzf(point.y()),
                    format_e(emitted_e)
                ),
                comment,
            )
        }
    }

    pub(crate) fn reset_e(&mut self) {
        self.current_e = 0.0;
    }

    pub(crate) const fn current_position(&self) -> (f64, f64, f64) {
        self.current_position
    }

    pub(crate) const fn current_feedrate(&self) -> f64 {
        self.current_feedrate
    }

    pub(crate) const fn current_e(&self) -> f64 {
        self.current_e
    }

    #[cfg(test)]
    pub(crate) const fn current_acceleration(&self) -> u32 {
        self.current_print_acceleration
    }

    #[cfg(test)]
    pub(crate) const fn current_jerk(&self) -> f64 {
        self.current_jerk
    }
}

fn format_xyzf(value: f64) -> String {
    format_axis(value, XYZF_EXPORT_DIGITS)
}

fn format_e(value: f64) -> String {
    format_axis(value, E_EXPORT_DIGITS)
}

fn format_pressure_advance(value: f64) -> String {
    format_axis(value, 4)
}

fn fan_pwm(speed: u8) -> u32 {
    (255.5 * f64::from(speed) / 100.0).floor() as u32
}

fn exhaust_fan_pwm(speed: u8) -> u32 {
    (f64::from(speed) / 100.0 * 255.0) as u32
}

fn additional_fan_pwm(speed: u8) -> u32 {
    (255.0 * f64::from(speed) / 100.0).floor() as u32
}

fn format_axis(value: f64, digits: usize) -> String {
    let mut text = format!("{value:.digits$}");
    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    if text == "-0" {
        text.clear();
        text.push('0');
    }
    text
}

pub(super) fn append_comment(mut command: String, comment: Option<&str>) -> String {
    if let Some(comment) = comment.filter(|comment| !comment.is_empty()) {
        command.pop();
        command.push_str(" ; ");
        command.push_str(comment);
        command.push('\n');
    }
    command
}

#[cfg(test)]
mod tests;
