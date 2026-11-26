//! `ExportLines::update` M73 lookup over `g1_times_cache` (`GCodeProcessor.cpp:771-810`, `:1241-1252`, `:1458-1471`).

use super::export_cache;

#[derive(Clone, Copy)]
pub(in crate::project_slice::gcode_emit::processor) struct Snapshot {
    pub(in crate::project_slice::gcode_emit::processor) elapsed: f32,
    pub(in crate::project_slice::gcode_emit::processor) next: Option<f32>,
}

#[derive(Default)]
pub(in crate::project_slice::gcode_emit::processor) struct ProgressElapsedCursor {
    cache_index: usize,
    update_cache_index: usize,
    exported_g1_lines: usize,
}

impl ProgressElapsedCursor {
    pub(in crate::project_slice::gcode_emit::processor) fn step(
        &mut self,
        line: &str,
        cache: &[export_cache::Entry],
    ) -> Option<Snapshot> {
        let command = line
            .split(';')
            .next()
            .unwrap_or_default()
            .split_whitespace()
            .next()
            .unwrap_or_default();
        if !matches!(command, "G0" | "G1" | "G2" | "G3" | "G28") {
            return None;
        }
        // `ExportLines::update` (:778-800) walks the current cache, not
        // the source arc's segment count: VG1 advances estimate ids only.
        let update_id = self.exported_g1_lines + 1;
        while self.update_cache_index < cache.len() && cache[self.update_cache_index].id < update_id
        {
            self.update_cache_index += 1;
        }
        let mut internal = 0;
        if matches!(command, "G2" | "G3")
            && cache
                .get(self.update_cache_index)
                .is_some_and(|entry| entry.id == update_id)
        {
            while cache
                .get(self.update_cache_index)
                .is_some_and(|entry| entry.remaining > 0)
            {
                self.update_cache_index += 1;
                internal += 1;
            }
        }
        if command == "G28" {
            self.exported_g1_lines += 1;
            return None;
        }
        // Caller :1458-1471 probes before advancing its physical counter;
        // its progress-cache iterator is separate from update's cursor.
        let lookup_id = self.exported_g1_lines + internal;
        self.exported_g1_lines += 1 + internal;
        while self.cache_index < cache.len() && cache[self.cache_index].id < lookup_id {
            self.cache_index += 1;
        }
        let hit = cache
            .get(self.cache_index)
            .is_some_and(|entry| entry.id == lookup_id);
        hit.then(|| Snapshot {
            elapsed: cache[self.cache_index].elapsed,
            next: cache.get(self.cache_index + 1).map(|entry| entry.elapsed),
        })
    }
}
