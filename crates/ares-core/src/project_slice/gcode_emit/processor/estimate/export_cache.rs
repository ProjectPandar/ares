//! Normal `ExportLines::update` cache cursor (`GCodeProcessor.cpp:771–810`).

use super::super::export_lines::reader::command;
use super::Estimate;

impl Estimate {
    pub(in crate::project_slice::gcode_emit::processor) fn export_elapsed_at(
        &self,
        line: usize,
    ) -> Option<f32> {
        self.export_elapsed[line]
    }
}

pub(super) struct Entry {
    pub(super) id: usize,
    pub(super) remaining: usize,
    pub(super) elapsed: f32,
}

pub(super) fn snapshots(lines: &[String], cache: &[Entry]) -> Vec<Option<f32>> {
    let mut snapshots = vec![None; lines.len() + 1];
    let mut cache_index = 0;
    let mut counter = 0;
    for (index, line) in lines.iter().enumerate() {
        let command = command(line);
        if !matches!(command, "G0" | "G1" | "G2" | "G3" | "G28") {
            continue;
        }
        let mut lookup = counter + 1;
        while cache_index < cache.len() && cache[cache_index].id < lookup {
            cache_index += 1;
        }
        let mut internal = 0;
        if cache
            .get(cache_index)
            .is_none_or(|entry| entry.id != lookup)
        {
            counter += 1;
            continue;
        }
        if matches!(command, "G2" | "G3") {
            while cache
                .get(cache_index)
                .is_some_and(|entry| entry.remaining > 0)
            {
                cache_index += 1;
                lookup += 1;
                internal += 1;
            }
        }
        if let Some(entry) = cache.get(cache_index)
            && entry.id == lookup
        {
            snapshots[index + 1] = Some(entry.elapsed);
        }
        counter += 1 + internal;
    }
    snapshots
}
