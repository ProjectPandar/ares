//! `ExportLines::update` M73 lookup over `g1_times_cache` (`GCodeProcessor.cpp:771-810`, `:1241-1252`, `:1458-1471`).

use super::{Estimate, export_cache};

impl Estimate {
    pub(in crate::project_slice::gcode_emit::processor) fn elapsed_at(
        &self,
        line: usize,
    ) -> Option<f64> {
        self.elapsed
            .get(line)
            .copied()
            .flatten()
            .map(|entry| f64::from(entry.elapsed))
    }

    pub(in crate::project_slice::gcode_emit::processor) fn next_elapsed_at(
        &self,
        line: usize,
    ) -> Option<f32> {
        self.elapsed
            .get(line)
            .copied()
            .flatten()
            .and_then(|entry| entry.next)
    }
}

#[derive(Clone, Copy)]
pub(super) struct Snapshot {
    pub(super) elapsed: f32,
    pub(super) next: Option<f32>,
}

pub(super) fn elapsed_snapshots(
    lines: &[String],
    cache: &[export_cache::Entry],
) -> Vec<Option<Snapshot>> {
    let mut elapsed = vec![None; lines.len() + 1];
    let mut cache_index = 0;
    let mut update_cache_index = 0;
    let mut exported_g1_lines = 0;
    for (index, line) in lines.iter().enumerate() {
        let command = line
            .split(';')
            .next()
            .unwrap_or_default()
            .split_whitespace()
            .next()
            .unwrap_or_default();
        if !matches!(command, "G0" | "G1" | "G2" | "G3" | "G28") {
            continue;
        }
        // `ExportLines::update` (:778-800) walks the current cache, not
        // the source arc's segment count: VG1 advances estimate ids only.
        let update_id = exported_g1_lines + 1;
        while update_cache_index < cache.len() && cache[update_cache_index].id < update_id {
            update_cache_index += 1;
        }
        let mut internal = 0;
        if matches!(command, "G2" | "G3")
            && cache
                .get(update_cache_index)
                .is_some_and(|entry| entry.id == update_id)
        {
            while cache
                .get(update_cache_index)
                .is_some_and(|entry| entry.remaining > 0)
            {
                update_cache_index += 1;
                internal += 1;
            }
        }
        if command == "G28" {
            exported_g1_lines += 1;
            continue;
        }
        // Caller :1458-1471 probes before advancing its physical counter;
        // its progress-cache iterator is separate from update's cursor.
        let lookup_id = exported_g1_lines + internal;
        exported_g1_lines += 1 + internal;
        while cache_index < cache.len() && cache[cache_index].id < lookup_id {
            cache_index += 1;
        }
        let hit = cache
            .get(cache_index)
            .is_some_and(|entry| entry.id == lookup_id);
        if hit {
            elapsed[index + 1] = Some(Snapshot {
                elapsed: cache[cache_index].elapsed,
                next: cache.get(cache_index + 1).map(|entry| entry.elapsed),
            });
        }
    }
    elapsed
}
/// Widen each sum for the addition (`GCodeProcessor.cpp:649–655`).
pub(super) fn stage_times(times: &[f64], stages: Vec<(bool, bool)>) -> (f32, f32) {
    times.iter().zip(stages).fold(
        (0.0_f32, 0.0_f32),
        |(prepare, first), (&time, (is_prepare, is_first))| {
            let add = |sum, enabled| {
                if enabled {
                    (f64::from(sum) + time) as f32
                } else {
                    sum
                }
            };
            (add(prepare, is_prepare), add(first, is_first))
        },
    )
}
