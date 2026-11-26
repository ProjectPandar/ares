//! `ExportLines` and postprocess T consumer (`GCodeProcessor.cpp:682–960,1300–1488`).

pub(super) mod reader;

use std::collections::VecDeque;

use crate::gcode_sink::ChunkWriter;
use crate::{GCodeFlavor, GCodeSink, OrcaInts, ProjectSettings, SliceError};
use reader::{axis, command, is_start};

struct LineData {
    line: String,
    time: f32,
    /// 1-based processor input line this output line exports; `0` for
    /// inserted or synthesized lines (`m_gcode_lines_map`, `:754`).
    source: usize,
}

/// Input line → 1-based exported line id, filled as lines are written.
struct LineMap {
    lines: Vec<u32>,
    written: u32,
}

struct Backtrace {
    time: f32,
    steps: i32,
    xl: bool,
}

pub(super) struct ExportLines<'a, 's> {
    settings: &'a ProjectSettings,
    flavor: GCodeFlavor,
    layer_marker: &'static str,
    layer: usize,
    time: f32,
    max_backtrace_time: f32,
    backtrace: Option<Backtrace>,
    lines: VecDeque<LineData>,
    size: usize,
    writer: ChunkWriter<'s>,
    line_map: Option<LineMap>,
}

impl<'a, 's> ExportLines<'a, 's> {
    pub(super) fn new(
        settings: &'a ProjectSettings,
        flavor: GCodeFlavor,
        bbl: bool,
        source_lines: Option<usize>,
        sink: &'s mut dyn GCodeSink,
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
            writer: ChunkWriter::new(sink),
            line_map: source_lines.map(|count| LineMap {
                lines: vec![0; count + 1],
                written: 0,
            }),
        }
    }

    pub(super) const fn has_backtrace(&self) -> bool {
        self.backtrace.is_some()
    }

    pub(super) fn begin(&mut self, line: &str, elapsed: Option<f32>) -> Result<(), SliceError> {
        let Some(backtrace) = &self.backtrace else {
            return Ok(());
        };
        let backtrace_time = backtrace.time;
        // Flush the preceding input line before updating this line's timestamp.
        self.write()?;
        if let Some(elapsed) = elapsed {
            self.time = elapsed;
        }
        if line == self.layer_marker {
            self.layer += 1;
        }
        if command(line).starts_with('T') {
            self.insert_lines(command(line));
            self.max_backtrace_time = self.max_backtrace_time.max(backtrace_time);
        }
        Ok(())
    }

    pub(super) fn flush_section(&mut self) -> Result<(), SliceError> {
        self.writer.flush_buffer()
    }

    pub(super) fn append(&mut self, line: String) -> Result<(), SliceError> {
        if self.backtrace.is_none() {
            if !line.is_empty() {
                self.write_direct(&[&line], 0)?;
            }
            return Ok(());
        }
        self.append_from(line, 0);
        Ok(())
    }

    /// Appends the export of processor input line `source` (1-based) without
    /// allocating a temporary `String` when tool backtrace is disabled.
    pub(super) fn append_parts(
        &mut self,
        line: &str,
        ending: &str,
        source: usize,
    ) -> Result<(), SliceError> {
        if self.backtrace.is_none() {
            if !line.is_empty() || !ending.is_empty() {
                self.write_direct(&[line, ending], source)?;
            }
            return Ok(());
        }
        let mut owned = String::with_capacity(line.len() + ending.len());
        owned.push_str(line);
        owned.push_str(ending);
        self.append_from(owned, source);
        Ok(())
    }

    fn write_direct(&mut self, parts: &[&str], source: usize) -> Result<(), SliceError> {
        if let Some(map) = &mut self.line_map {
            let newlines: usize = parts
                .iter()
                .map(|part| part.bytes().filter(|&byte| byte == b'\n').count())
                .sum();
            map.written += newlines as u32;
            if source != 0 {
                map.lines[source] = map.written;
            }
        }
        for part in parts {
            self.writer.push_bytes(part.as_bytes())?;
        }
        Ok(())
    }

    /// Appends the export of processor input line `source` (1-based).
    fn append_from(&mut self, line: String, source: usize) {
        if !line.is_empty() {
            self.size += line.len();
            self.lines.push_back(LineData {
                line,
                time: self.time,
                source,
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
                self.lines.insert(
                    index + 1,
                    LineData {
                        line,
                        time,
                        source: 0,
                    },
                );
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

    fn write(&mut self) -> Result<(), SliceError> {
        if self.backtrace.is_some() {
            let threshold = self.time - 1.1_f32 * self.max_backtrace_time;
            while self.lines.front().is_some_and(|line| line.time < threshold) {
                self.write_front()?;
            }
        } else if self.size > 65_535 {
            while !self.lines.is_empty() {
                self.write_front()?;
            }
        }
        Ok(())
    }

    fn write_front(&mut self) -> Result<(), SliceError> {
        let line = self.lines.pop_front().expect("a buffered line exists");
        self.size -= line.line.len();
        if let Some(map) = &mut self.line_map {
            map.written += line.line.bytes().filter(|byte| *byte == b'\n').count() as u32;
            if line.source != 0 {
                map.lines[line.source] = map.written;
            }
        }
        self.writer.push_bytes(line.line.as_bytes())
    }

    /// Flushes all exported G-code chunks to the sink and returns total bytes
    /// written plus, when tracked, the input → output line map.
    pub(super) fn finish(mut self) -> Result<(usize, Option<Vec<u32>>), SliceError> {
        while !self.lines.is_empty() {
            self.write_front()?;
        }
        let bytes_written = self.writer.finish()?;
        Ok((bytes_written, self.line_map.map(|map| map.lines)))
    }
}

fn at(values: &OrcaInts, index: usize) -> i32 {
    values.0.get(index).unwrap_or(&values.0[0]).0
}
