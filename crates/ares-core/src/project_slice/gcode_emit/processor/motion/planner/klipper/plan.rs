//! `TimeMachine::plan_klipper` (`GCodeProcessor.cpp:505-563`), v² passes.
use super::Block;

pub(super) fn plan(blocks: &mut [Block], lazy: bool) -> usize {
    let mut flush_count = blocks.len();
    let mut update_flush_count = lazy;
    let mut next_start_v2 = 0.0_f32;
    let mut next_mcr_start_v2 = 0.0_f32;
    let mut peak_cruise_v2 = 0.0_f32;
    let mut pending = 0;
    for (index, block) in blocks.iter_mut().enumerate().rev() {
        let delta_v2 = 2.0 * block.distance * block.acceleration;
        let reachable_start_v2 = next_start_v2 + delta_v2;
        let start_v2 = (block.max_entry * block.max_entry).min(reachable_start_v2);
        let reachable_mcr_v2 = next_mcr_start_v2 + block.mcr_delta_v2;
        let mcr_start_v2 = block.max_mcr_entry_sqr.min(reachable_mcr_v2);
        let mut cruise_v2 = -1.0;
        pending += 1;
        if mcr_start_v2 < reachable_mcr_v2
            && (mcr_start_v2 + block.mcr_delta_v2 > next_mcr_start_v2 || pending > 1)
        {
            if update_flush_count && peak_cruise_v2 > 0.0 {
                flush_count = index;
                update_flush_count = false;
            }
            peak_cruise_v2 =
                (block.cruise * block.cruise).min(0.5 * (mcr_start_v2 + reachable_mcr_v2));
        }
        if mcr_start_v2 < reachable_mcr_v2 {
            cruise_v2 = (0.5 * (start_v2 + reachable_start_v2))
                .min(block.cruise * block.cruise)
                .min(peak_cruise_v2);
            pending = 0;
        }
        block.entry = start_v2;
        block.exit = next_start_v2;
        block.planned_cruise = cruise_v2;
        block.peak_cruise_v2 = peak_cruise_v2;
        next_start_v2 = start_v2;
        next_mcr_start_v2 = mcr_start_v2;
    }
    if update_flush_count {
        return 0;
    }
    let mut prev_cruise_v2 = 0.0_f32;
    for block in &mut blocks[..flush_count] {
        if block.planned_cruise < 0.0 {
            block.planned_cruise = prev_cruise_v2.min(block.entry);
            prev_cruise_v2 = block.planned_cruise;
        } else {
            prev_cruise_v2 = block.peak_cruise_v2;
        }
    }
    flush_count
}
