use super::super::filament_change::unloads_filament;
use super::{IndexedMotion, MoveRecorder, RecordedMotion, SeamDetector, word};

/// The `Seam` vertex each arc segment's `process_G1` stores, if any; a
/// seam-carrying or non-cacheable segment stays out of `g1_times_cache`.
pub(super) fn arc_seams(
    seams: &mut SeamDetector,
    motion_blocks: &[IndexedMotion],
    arc_start: [f64; 3],
    wiping: bool,
    cache_eligible: &mut Vec<bool>,
) -> Vec<Option<[f32; 3]>> {
    let mut position = arc_start;
    motion_blocks
        .iter()
        .map(|IndexedMotion { block, .. }| {
            let end = std::array::from_fn::<_, 3, _>(|i| position[i] + block.delta[i]);
            let extruding =
                !wiping && block.delta[3] > 0.0 && (block.delta[0] != 0.0 || block.delta[1] != 0.0);
            position = end;
            let seam = seams.associate(extruding, end);
            cache_eligible.push((!block.e_only || wiping) && seam.is_none());
            seam
        })
        .collect()
}

/// Records one command's `process_G1` vertices; each arc segment starts
/// where the previous one ended. Returns the move id of every block.
pub(super) fn record_motions(
    recorder: &mut MoveRecorder,
    mut motion: RecordedMotion,
    motions: &[IndexedMotion],
    seams: &[Option<[f32; 3]>],
) -> Vec<u32> {
    motions
        .iter()
        .enumerate()
        .map(|(index, indexed)| {
            motion.end = indexed.end;
            motion.seam = seams.get(index).copied().flatten();
            let move_id = recorder.motion(motion);
            motion.start = indexed.end;
            move_id
        })
        .collect()
}

pub(super) fn synchronizes_planner(code: &str, measured_g29: bool) -> bool {
    let command = code.split_whitespace().next().unwrap_or_default();
    matches!(command, "M0" | "M1")
        || matches!(command, "G4" | "M400")
            && (word(code, 'S').is_some() || word(code, 'P').is_some())
        || measured_g29
        || command == "M191" && word(code, 'S').unwrap_or(0.0) > 40.0
        || command == "G92" && word(code, 'E').is_none()
        || unloads_filament(code)
        || command == "SYNC" && word(code, 'T').is_some()
}
