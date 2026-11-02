use super::super::filament_change::unloads_filament;
use super::super::motion::{KlipperConfig, MotionBlock, MotionKind};
use super::{IndexedMotion, MoveRecorder, RecordedMotion, SeamDetector, word};

pub(super) fn tool_change_block(klipper: KlipperConfig) -> MotionBlock {
    MotionBlock {
        klipper,
        virtual_motion: false,
        distance: 0.0,
        delta: [0.0; 4],
        speed: 0.0,
        acceleration: 0.0,
        centripetal_acceleration: 0.0,
        max_feedrate: [0.0; 4],
        max_acceleration: [0.0; 4],
        jerk: [0.0; 4],
        extrude_factor: 1.0,
        // Tool_change never enters g1_times_cache or advances
        // the G1 ID, including zero-duration transitions.
        e_only: true,
        kind: MotionKind::ToolChange,
    }
}

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
/// where the previous one ended.
pub(super) fn record_motions(
    recorder: &mut MoveRecorder,
    mut motion: RecordedMotion,
    motions: &[IndexedMotion],
    seams: &[Option<[f32; 3]>],
) {
    for (index, indexed) in motions.iter().enumerate() {
        motion.end = indexed.end;
        motion.seam = seams.get(index).copied().flatten();
        let vertex = recorder.options_z.next_vertex();
        recorder.motion(motion);
        recorder.options_z.block(vertex);
        motion.start = indexed.end;
    }
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
