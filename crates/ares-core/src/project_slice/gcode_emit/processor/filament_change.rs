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

    pub(super) fn current_filament(&self) -> usize {
        self.physical.and_then(|id| self.current[id]).unwrap_or(0)
    }

    /// Returns the selected filament and the change's extra time.
    pub(super) fn command(&mut self, code: &str) -> Option<(usize, f32)> {
        let command = code.split_whitespace().next()?;
        if unloads_filament(code) {
            // Upstream sets this before consulting its unload-time getter.
            self.unloaded = true;
        }
        let filament = match selected_filament(command, self.count) {
            Some(filament) => filament,
            // `GCodeProcessor.cpp:1829` binds M1020 to the same
            // filament-change accounting as T (`:5444-5466`).
            None => m1020_filament(command, code, self.count)?,
        };
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
        Some((filament, extra))
    }

    pub(super) fn selection(&self) -> (usize, usize, usize) {
        let physical = self.physical.unwrap_or(0);
        (
            physical,
            self.current[physical].unwrap_or(0),
            self.last[physical].unwrap_or(0),
        )
    }

    pub(super) fn initialized(&self) -> bool {
        self.physical.is_some()
    }

    fn unload_time(&self) -> f32 {
        if self.unloaded { 0.0 } else { self.unload }
    }
}

/// `GCodeProcessor.cpp:5444-5466`: M1020 selects the filament from the
/// raw line's `substr(7)` when the line is longer than 5 characters;
/// `stoi` skips leading whitespace and one optional sign and tolerates
/// trailing junk, and ids outside `[0, filaments_count)` select nothing.
fn m1020_filament(command: &str, code: &str, count: usize) -> Option<usize> {
    if command != "M1020" || code.len() <= 5 {
        return None;
    }
    let rest = code.get(7..)?.trim_start();
    let (sign, digits) = match rest.as_bytes().first() {
        Some(b'-') => (-1_i64, &rest[1..]),
        Some(b'+') => (1_i64, &rest[1..]),
        _ => (1_i64, rest),
    };
    let end = digits.bytes().take_while(u8::is_ascii_digit).count();
    if end == 0 {
        // `stoi` would throw; no producer emits a nonnumeric payload.
        return None;
    }
    let filament = sign * digits[..end].parse::<i64>().unwrap_or(i64::MAX);
    (0..=254)
        .contains(&filament)
        .then_some(filament as usize)
        .filter(|filament| *filament < count)
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
