//! Normal `ExportLines::update` cache cursor (`GCodeProcessor.cpp:771–810`).

use super::super::export_lines::reader::command;

pub(in crate::project_slice::gcode_emit::processor) struct Entry {
    pub(super) id: usize,
    pub(super) remaining: usize,
    pub(super) elapsed: f32,
}

#[derive(Default)]
pub(in crate::project_slice::gcode_emit::processor) struct ExportElapsedCursor {
    cache_index: usize,
    counter: usize,
}

impl ExportElapsedCursor {
    pub(in crate::project_slice::gcode_emit::processor) fn step(
        &mut self,
        line: &str,
        cache: &[Entry],
    ) -> Option<f32> {
        let command = command(line);
        if !matches!(command, "G0" | "G1" | "G2" | "G3" | "G28") {
            return None;
        }
        let mut lookup = self.counter + 1;
        while self.cache_index < cache.len() && cache[self.cache_index].id < lookup {
            self.cache_index += 1;
        }
        let mut internal = 0;
        if cache
            .get(self.cache_index)
            .is_none_or(|entry| entry.id != lookup)
        {
            self.counter += 1;
            return None;
        }
        if matches!(command, "G2" | "G3") {
            while cache
                .get(self.cache_index)
                .is_some_and(|entry| entry.remaining > 0)
            {
                self.cache_index += 1;
                lookup += 1;
                internal += 1;
            }
        }
        let elapsed = cache
            .get(self.cache_index)
            .filter(|entry| entry.id == lookup)
            .map(|entry| entry.elapsed);
        self.counter += 1 + internal;
        elapsed
    }
}
