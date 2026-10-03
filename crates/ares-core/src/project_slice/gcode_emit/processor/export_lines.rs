//! `ExportLines` and postprocess T consumer (`GCodeProcessor.cpp:682–960,1300–1488`).

pub(super) mod reader;

use std::collections::VecDeque;

use crate::{GCodeFlavor, OrcaInts, ProjectSettings};
use reader::{axis, command, is_start};

struct LineData {
    line: String,
    time: f32,
}

struct Backtrace {
    time: f32,
    steps: i32,
    xl: bool,
}

pub(super) struct ExportLines<'a> {
    settings: &'a ProjectSettings,
    flavor: GCodeFlavor,
    layer_marker: &'static str,
    layer: usize,
    time: f32,
    max_backtrace_time: f32,
    backtrace: Option<Backtrace>,
    lines: VecDeque<LineData>,
    size: usize,
    output: Vec<u8>,
}

impl<'a> ExportLines<'a> {
    pub(super) fn new(
        settings: &'a ProjectSettings,
        flavor: GCodeFlavor,
        bbl: bool,
        capacity: usize,
    ) -> Self {
        let print = &settings.process.print;
        let notes = &settings.printer.remaining.printer_notes.0;
        let xl = notes.contains("PRINTER_VENDOR_PRUSA3D") && notes.contains("PRINTER_MODEL_XL");
        let preheat_time = print.preheat_time.0 as f32;
        let enabled = print.ooze_prevention.0
            && preheat_time > 0.0
            && (xl
                || !settings.printer.gcode.single_extruder_multi_material.0
                    && settings.filament.gcode.filament_diameter.0.len() > 1);
        Self {
            settings,
            flavor,
            layer_marker: if bbl {
                super::super::tags::CHANGE_LAYER
            } else {
                super::super::tags::LAYER_CHANGE
            },
            layer: 0,
            time: 0.0,
            max_backtrace_time: 120.0,
            backtrace: enabled.then_some(Backtrace {
                time: preheat_time,
                steps: print.preheat_steps.0.max(1),
                xl,
            }),
            lines: VecDeque::new(),
            size: 0,
            output: Vec::with_capacity(capacity),
        }
    }

    pub(super) fn begin(&mut self, line: &str, elapsed: Option<f32>) {
        // Flush the preceding input line before updating this line's timestamp.
        self.write();
        if let Some(elapsed) = elapsed {
            self.time = elapsed;
        }
        if line == self.layer_marker {
            self.layer += 1;
        }
        if let Some(backtrace) = &self.backtrace
            && command(line).starts_with('T')
        {
            let time = backtrace.time;
            self.insert_lines(command(line));
            self.max_backtrace_time = self.max_backtrace_time.max(time);
        }
    }

    pub(super) fn append(&mut self, line: String) {
        if !line.is_empty() {
            self.size += line.len();
            self.lines.push_back(LineData {
                line,
                time: self.time,
            });
        }
    }

    fn insert_lines(&mut self, cmd: &str) {
        let Some(number) = cmd
            .strip_prefix('T')
            .filter(|number| !number.starts_with('+'))
            .and_then(|number| number.parse::<i32>().ok())
        else {
            return;
        };
        if number == -1 {
            return;
        }
        let count = self.settings.filament.gcode.filament_diameter.0.len();
        let tool = number.clamp(0, count as i32 - 1) as usize;
        let physical = self
            .settings
            .printer
            .gcode
            .physical_extruder_map
            .0
            .get(tool)
            .map_or(tool, |id| id.0 as usize);
        let backtrace = self.backtrace.as_ref().expect("tool backtrace is enabled");
        let step_time = backtrace.time / backtrace.steps as f32;
        let mut distance = 0;
        let mut last_insertion = 0.0;
        for step in 1..=backtrace.steps {
            let threshold = self.time - step as f32 * step_time;
            let start = distance;
            let reverse = self.walk_window(cmd, threshold, physical, start);
            if reverse == self.lines.len() {
                continue;
            }
            let index = self.lines.len() - 1 - reverse;
            let current = command(&self.lines[index].line);
            if current == cmd || is_start(current) {
                break;
            }
            let time = self.lines[index].time;
            if reverse != start && time != last_insertion {
                last_insertion = time;
                let seconds = (self.time - time).round() as i32;
                let line = self.preheat(tool, physical, seconds);
                distance = reverse + 1;
                self.size += line.len();
                self.lines.insert(index + 1, LineData { line, time });
            }
        }
    }

    fn walk_window(
        &mut self,
        cmd: &str,
        threshold: f32,
        physical: usize,
        mut reverse: usize,
    ) -> usize {
        while reverse < self.lines.len() {
            let index = self.lines.len() - 1 - reverse;
            let data = &mut self.lines[index];
            let current = command(&data.line);
            if data.time <= threshold || current == cmd || is_start(current) {
                break;
            }
            if current == "M104"
                && data.line.contains("cooldown")
                && axis(&data.line, 'T').is_some_and(|id| {
                    // External G-code: only defined casts can match a non-negative tool.
                    id > -1.0 && id < i32::MAX as f32 && id as i32 == physical as i32
                })
            {
                self.size -= data.line.len();
                data.line = "; removed M104\n".into();
                self.size += data.line.len();
            }
            reverse += 1;
        }
        reverse
    }

    fn preheat(&self, tool: usize, physical: usize, seconds: i32) -> String {
        let filament = &self.settings.filament.print;
        let first = at(&filament.nozzle_temperature_initial_layer, tool);
        let normal = at(&filament.nozzle_temperature, tool);
        // apply_config normalizes ordinary0 to initial (`2076–2082`).
        let temperature = if self.layer <= 1 || normal == 0 {
            first
        } else {
            normal
        };
        if self
            .backtrace
            .as_ref()
            .expect("tool backtrace is enabled")
            .xl
        {
            return format!("M104.1 T{tool} P{seconds} S{temperature}\n");
        }
        super::super::temperature::command(
            self.flavor,
            temperature as u32,
            false,
            Some(physical),
            &format!("preheat T{physical} time: {seconds}s"),
        )
    }

    fn write(&mut self) {
        if self.backtrace.is_some() {
            let threshold = self.time - 1.1_f32 * self.max_backtrace_time;
            while self.lines.front().is_some_and(|line| line.time < threshold) {
                self.write_front();
            }
        } else if self.size > 65_535 {
            while !self.lines.is_empty() {
                self.write_front();
            }
        }
    }

    fn write_front(&mut self) {
        let line = self.lines.pop_front().expect("a buffered line exists");
        self.size -= line.line.len();
        self.output.extend_from_slice(line.line.as_bytes());
    }

    pub(super) fn finish(mut self) -> Vec<u8> {
        while !self.lines.is_empty() {
            self.write_front();
        }
        self.output
    }
}

fn at(values: &OrcaInts, index: usize) -> i32 {
    values.0.get(index).unwrap_or(&values.0[0]).0
}
