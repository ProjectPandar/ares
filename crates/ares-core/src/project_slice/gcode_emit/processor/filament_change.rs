//! `GCodeProcessor::process_filament_change` timing state, independent of the writer.

use super::ProcessorConfig;

pub(super) struct FilamentChange<'a> {
    map: &'a [crate::options::OrcaInt],
    count: usize,
    physical: Option<usize>,
    current: [Option<usize>; 64],
    last: [Option<usize>; 64],
    unloaded: bool,
    load: f32,
    unload: f32,
    change: f32,
}

impl<'a> FilamentChange<'a> {
    pub(super) fn new(config: &ProcessorConfig<'a>, load: f64) -> Self {
        Self {
            map: &config.settings.project.gcode.filament_map.0,
            count: config.settings.filament.gcode.filament_diameter.0.len(),
            physical: None,
            current: [None; 64],
            last: [None; 64],
            unloaded: true,
            load: load as f32,
            unload: config.settings.printer.gcode.machine_unload_filament_time.0 as f32,
            change: config.settings.printer.gcode.machine_tool_change_time.0 as f32,
        }
    }

    pub(super) fn command(&mut self, code: &str) -> Option<f32> {
        let command = code.split_whitespace().next()?;
        if unloads_filament(code) {
            // Upstream sets this before consulting its unload-time getter.
            self.unloaded = true;
        }
        let filament = selected_filament(command, self.count)?;
        let previous = self.physical.and_then(|id| self.current[id]);
        if previous == Some(filament) {
            return None;
        }
        let next_physical = (self.map[filament].0 - 1) as usize;
        if let Some(physical) = self.physical {
            self.last[physical] = previous;
        }
        let mut extra = 0.0_f32;
        if self.physical == Some(next_physical) {
            self.current[next_physical] = Some(filament);
            extra += self.unload_time();
            self.unloaded = false;
            extra += self.load;
        } else {
            let initial = self.physical.is_none();
            self.physical = Some(next_physical);
            if initial || self.last[next_physical].is_none() {
                self.current[next_physical] = Some(filament);
                self.unloaded = false;
                extra += self.load;
            } else if self.last[next_physical] != Some(filament) {
                self.current[next_physical] = Some(filament);
                extra += self.unload_time();
                self.unloaded = false;
                extra += self.load;
            }
            if !initial {
                extra += self.change;
            }
        }
        Some(extra)
    }

    fn unload_time(&self) -> f32 {
        if self.unloaded { 0.0 } else { self.unload }
    }
}

/// `GCodeProcessor.cpp:5470–5499`: logical T validation, not writer selection.
pub(in crate::project_slice::gcode_emit) fn selected_filament(
    code: &str,
    count: usize,
) -> Option<usize> {
    let command = code.split_whitespace().next()?;
    let digits = command.strip_prefix('T')?;
    let length = digits.bytes().take_while(u8::is_ascii_digit).count();
    if length == 0 {
        return None;
    }
    // unsigned from_chars leaves the initialized destination at zero on
    // overflow; trailing nonnumeric characters do not reject a prefix.
    let filament = digits[..length].parse::<u32>().unwrap_or(0) as usize;
    (filament <= 254 && filament < count).then_some(filament)
}

pub(super) fn unloads_filament(code: &str) -> bool {
    let code = code
        .split([';', '\r', '\n', '\0'])
        .next()
        .unwrap_or_default();
    let mut words = code.split([' ', '\t']).filter(|word| !word.is_empty());
    words.next() == Some("M702") && words.any(|word| word.starts_with('C'))
}
