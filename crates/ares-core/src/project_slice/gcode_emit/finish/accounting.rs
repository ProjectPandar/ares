/// `GCodeProcessor::process_G1` / `UsedFilaments::process_*_cache`.
/// Track the global E
/// position (M82/M83, G90/G91, G92 resets), classify each move
/// (`move_type`, `GCodeProcessor.cpp:3834-3851`) and bucket extruded
/// volume by the active `;TYPE:` role tag — upstream parses the same
/// marker via `reserved_tag(ETags::Role)` +
/// `ExtrusionEntity::string_to_role` (`GCodeProcessor.cpp:3120-3122`,
/// `ExtrusionEntity.cpp:593-651`).
///
/// Role → bucket (`GCodeProcessor.cpp:3902-3911`):
/// `Support`/`Support interface`/`Support transition` → support,
/// `Prime tower` (erWipeTower) → wipe tower, everything else → model.
/// `; FLUSH_START`/`; FLUSH_END` regions (`Flush_Start_Tag`,
/// `GCodeProcessor.cpp:3170-3178`) credit positive-E deltas to
/// `flush_per_filament` + `total` (`update_flush_per_filament`,
/// `GCodeProcessor.cpp:1606-1619, 3949-3961`); upstream additionally
/// caps flush by `nozzle_volume` via `m_remaining_volume` — that cap is
/// deferred since the emitter does not tag flush regions today.
pub(in crate::project_slice::gcode_emit) fn account_statistics(
    gcode: &[u8],
    diameters: &[crate::options::OrcaFloat],
) -> crate::PrintEstimatedStatistics {
    let mut statistics = crate::PrintEstimatedStatistics {
        volumes_per_color_change: Vec::new(),
        model_volumes_per_extruder: vec![0.0; diameters.len()],
        wipe_tower_volumes_per_extruder: vec![0.0; diameters.len()],
        support_volumes_per_extruder: vec![0.0; diameters.len()],
        total_volumes_per_extruder: vec![0.0; diameters.len()],
        flush_per_filament: vec![0.0; diameters.len()],
        total_used_filament: 0.0,
    };
    let mut filament = None;
    let text = std::str::from_utf8(gcode).unwrap_or_default();
    let mut e_position = 0.0_f64;
    let mut x_position = 0.0_f64;
    let mut y_position = 0.0_f64;
    let mut e_local_relative = false;
    let mut global_relative = false;
    let mut role = Bucket::Model;
    let mut flushing = false;
    // `UsedFilaments::model/support/wipe_tower_volume_cache` — one
    // cache per role, flushed to the active filament at every tool
    // change (`process_caches`, `GCodeProcessor.cpp:5931-5944`).
    let mut pending = [0.0_f64; 3];
    let mut used = 0.0_f64;
    for line in text.lines() {
        // `GCodeProcessor::process_tags` reads the role tag from the
        // comment span — `;TYPE:` for compatible printers,
        // `; FEATURE: ` for BBL (`Reserved_Tags` vs
        // `Reserved_Tags_compatible`, `GCodeProcessor.cpp:58-81`).
        if let Some(comment) = line.split_once(';').map(|(_, comment)| comment) {
            if let Some(name) = comment
                .strip_prefix("TYPE:")
                .or_else(|| comment.strip_prefix(" FEATURE: "))
            {
                role = Bucket::from_role(name.trim());
                continue;
            }
            if comment.starts_with(" FLUSH_START") {
                flushing = true;
                continue;
            }
            if comment.starts_with(" FLUSH_END") {
                flushing = false;
                continue;
            }
        }
        let code = line.split_once(';').map_or(line, |(code, _)| code).trim();
        let mut words = code.split_ascii_whitespace();
        let Some(command) = words.next() else {
            continue;
        };
        let letter_value = |letter: char| {
            words.clone().find_map(|word| {
                word.strip_prefix(letter)
                    .and_then(|value| value.parse::<f64>().ok())
                    .map(|value| f64::from(value as f32))
            })
        };
        if let Some(next) =
            super::super::processor::filament_change::selected_filament(code, diameters.len())
        {
            if let Some(previous) = filament
                && previous != next
            {
                // `process_caches` flushes the role caches into the
                // per-filament buckets at every tool change
                // (`GCodeProcessor.cpp:5942-5947`).
                flush_pending(&mut statistics, &mut pending, previous);
                statistics.total_volumes_per_extruder[previous] += used;
                used = 0.0;
            }
            filament = Some(next);
        }
        match command {
            "M82" => e_local_relative = false,
            "M83" => e_local_relative = true,
            "G90" => global_relative = false,
            "G91" => global_relative = true,
            "G92" => {
                if let Some(value) = letter_value('E') {
                    e_position = value;
                }
            }
            "G0" | "G1" | "G2" | "G3" => {
                let x = letter_value('X');
                let y = letter_value('Y');
                let e = letter_value('E');
                let start = (x_position, y_position, e_position);
                let e_relative = global_relative || e_local_relative;
                if let Some(value) = x {
                    x_position = apply_axis(x_position, value, global_relative);
                }
                if let Some(value) = y {
                    y_position = apply_axis(y_position, value, global_relative);
                }
                if let Some(value) = e {
                    e_position = apply_e(e_position, value, e_relative);
                }
                let dx = x_position - start.0;
                let dy = y_position - start.1;
                let delta_e = e_position - start.2;
                if delta_e <= 0.0 {
                    continue;
                }
                let diameter = diameters[filament.unwrap_or(0)].0 as f32;
                let radius = 0.5_f32 * diameter;
                let area = std::f32::consts::PI * (radius * radius);
                // GCodeProcessor::process_G1 stores each extruded volume
                // as float.
                let volume = f64::from((f64::from(area) * delta_e) as f32);
                // `EMoveType::Unretract && m_flushing` → flush + total
                // (`GCodeProcessor.cpp:3949-3961`); extrude moves keep
                // their role bucket.
                if flushing && dx == 0.0 && dy == 0.0 {
                    statistics.flush_per_filament[filament.unwrap_or(0)] += volume;
                    used += volume;
                } else if dx != 0.0 || dy != 0.0 {
                    pending[role as usize] += volume;
                    used += volume;
                }
            }
            _ => {}
        }
    }
    flush_pending(&mut statistics, &mut pending, filament.unwrap_or(0));
    statistics.total_volumes_per_extruder[filament.unwrap_or(0)] += used;
    // `PrintStatistics::total_used_filament` (`GCode.cpp:1914-1937`).
    statistics.total_used_filament = statistics
        .total_volumes_per_extruder
        .iter()
        .zip(diameters)
        .map(|(volume, diameter)| {
            let diameter = f64::from(diameter.0 as f32);
            volume / (std::f64::consts::PI * (0.5 * diameter).powi(2))
        })
        .sum();
    statistics
}

/// `UsedFilaments` role cache ordering — the `pending` array index.
#[derive(Clone, Copy)]
enum Bucket {
    Model = 0,
    Support = 1,
    WipeTower = 2,
}

impl Bucket {
    /// `ExtrusionEntity::string_to_role` restricted to the support /
    /// wipe-tower split `GCodeProcessor.cpp:3902-3911` applies; every
    /// other role counts as model.
    fn from_role(name: &str) -> Bucket {
        match name {
            "Support" | "Support interface" | "Support transition" => Bucket::Support,
            "Prime tower" => Bucket::WipeTower,
            _ => Bucket::Model,
        }
    }
}

fn flush_pending(
    statistics: &mut crate::PrintEstimatedStatistics,
    pending: &mut [f64; 3],
    filament: usize,
) {
    for (bucket, cache) in [
        &mut statistics.model_volumes_per_extruder,
        &mut statistics.support_volumes_per_extruder,
        &mut statistics.wipe_tower_volumes_per_extruder,
    ]
    .iter_mut()
    .zip(pending.iter())
    {
        bucket[filament] += cache;
    }
    *pending = [0.0; 3];
}

fn apply_e(current: f64, value: f64, relative: bool) -> f64 {
    if relative { current + value } else { value }
}

fn apply_axis(current: f64, value: f64, relative: bool) -> f64 {
    if relative { current + value } else { value }
}
