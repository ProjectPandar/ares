//! `ExportLines::update` M73 lookup over `g1_times_cache` (`GCodeProcessor.cpp:771-810`, `:1241-1252`, `:1458-1471`).

use super::export_cache;

#[derive(Clone, Copy)]
pub(super) struct Snapshot {
    pub(super) elapsed: f32,
    pub(super) next: Option<f32>,
}

pub(super) fn elapsed_snapshots(
    lines: &[String],
    cache: &[export_cache::Entry],
    arc_segment_counts: &[usize],
    arc_seam_excluded: &[bool],
) -> Vec<Option<Snapshot>> {
    let mut elapsed = vec![None; lines.len() + 1];
    let mut cache_index = 0;
    let mut exported_g1_lines = 0;
    for (index, line) in lines.iter().enumerate() {
        let command = line
            .split(';')
            .next()
            .unwrap_or_default()
            .split_whitespace()
            .next()
            .unwrap_or_default();
        let lookup_id = match command {
            "G0" | "G1" => {
                // `process_line_move(g1_lines_counter++)` (:1460) probes
                // `g1_times_cache_it->id == g1_lines_counter` at the
                // PRE-increment caller counter (:1246-1248) — the line
                // reads the cumulative through the previous eligible
                // cache entry.
                let id = exported_g1_lines;
                exported_g1_lines += 1;
                Some(id)
            }
            "G2" | "G3" => {
                // `ret` = `internal_g1_lines_counter` = the
                // `remaining_internal_g1_lines > 0` walk count in
                // `ExportLines::update` (:793-799): `arc_internal - 1`
                // eligible internal entries for a fully-cached arc
                // (the `it->id == counter` probe lands on the
                // second-to-last internal segment), 0 when the first
                // block was seam-excluded (`it->id > counter`).
                // `process_line_move(g1_lines_counter + ret)` (:1466),
                // `g1_lines_counter += 1 + ret` (:1467). The `it->id`
                // probe is `exported + 1 + ret` (`++` at :778).
                let internal = if arc_seam_excluded[index] {
                    0
                } else {
                    arc_segment_counts[index]
                };
                // `process_line_move(g1_lines_counter + internal)` (:1466)
                // probes `g1_times_cache_it->id == g1_lines_counter +
                // internal` (:1246-1248) — `internal` is `update`'s
                // `ret` = the `remaining>0` walk count = `arc_internal`
                // for a fully-cached arc, 0 when the first block was
                // seam-excluded. `g1_lines_counter += 1 + internal` (:1467).
                let lookup = exported_g1_lines + internal;
                exported_g1_lines += 1 + internal;
                Some(lookup)
            }
            "G28" => {
                exported_g1_lines += 1;
                None
            }
            _ => None,
        };
        let Some(lookup_id) = lookup_id else {
            continue;
        };
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
