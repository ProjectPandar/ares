//! `GCodeProcessor::calculate_time` planner flush and delay accounting.

use super::motion::{KlipperConfig, MotionBlock, MotionKind, Planner, Trapezoid};
use crate::GCodeFlavor;
use std::collections::VecDeque;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum DelayTarget {
    Any,
    ToolChange,
}

#[derive(Clone, Copy)]
pub(super) struct PendingDelay {
    pub(super) target: DelayTarget,
    pub(super) seconds: f32,
}

pub(super) struct FlushEvent {
    pub(super) block_count: usize,
    pub(super) delay: Option<PendingDelay>,
    pub(super) klipper: KlipperConfig,
}

pub(super) struct Schedule {
    pub(super) times: Vec<f64>,
    /// Trapezoid of each processed block; blocks the final pass drops keep
    /// the zero default.
    pub(super) trapezoids: Vec<Trapezoid>,
    pub(super) trailing_delay: f64,
    /// Processed-block count after each explicit synchronization.
    pub(super) flush_ends: Vec<usize>,
    /// Observation for stop registration, before each explicit flush.
    pub(super) flush_starts: Vec<usize>,
    /// Processed-block count once each block is queued.
    pub(super) pushed: Vec<usize>,
    pub(super) processed_blocks: usize,
}

pub(super) fn scheduled_times(
    blocks: &[MotionBlock],
    events: &[FlushEvent],
    flavor: GCodeFlavor,
) -> Schedule {
    let mut planner = Planner::new(flavor);
    let klipper = flavor == GCodeFlavor::Klipper;
    // Blocks the final pass drops keep zero time and the zero trapezoid.
    let mut timed = vec![(0.0, Trapezoid::default()); blocks.len()];
    let mut pending = VecDeque::new();
    let mut pushed = 0;
    let mut emitted = 0;
    let mut flush_ends = Vec::with_capacity(events.len());
    let mut flush_starts = Vec::with_capacity(events.len());
    let mut pushed_processed = Vec::with_capacity(blocks.len());

    for event in events {
        while pushed < event.block_count {
            let batch = planner.push(&blocks[pushed]);
            pushed += 1;
            record_batch(
                blocks,
                (&mut timed, &mut emitted),
                batch,
                &mut pending,
                klipper,
            );
            pushed_processed.push(emitted);
        }
        flush_starts.push(emitted);
        if let Some(delay) = event.delay.filter(|delay| delay.seconds > 0.0) {
            if let Some(last) = pending
                .back_mut()
                .filter(|last| last.target == delay.target)
            {
                last.seconds += delay.seconds;
            } else {
                pending.push_back(delay);
            }
        }
        let batch = planner.flush(event.klipper);
        record_batch(
            blocks,
            (&mut timed, &mut emitted),
            batch,
            &mut pending,
            klipper,
        );
        flush_ends.push(emitted);
    }
    while pushed < blocks.len() {
        let batch = planner.push(&blocks[pushed]);
        pushed += 1;
        record_batch(
            blocks,
            (&mut timed, &mut emitted),
            batch,
            &mut pending,
            klipper,
        );
        pushed_processed.push(emitted);
    }
    let batch = planner.finish(!pending.is_empty());
    record_batch(
        blocks,
        (&mut timed, &mut emitted),
        batch,
        &mut pending,
        klipper,
    );

    let trailing_delay = pending
        .into_iter()
        .fold(0.0_f32, |total, delay| total + delay.seconds);
    let (times, trapezoids) = timed.into_iter().unzip();
    Schedule {
        times,
        trapezoids,
        trailing_delay: f64::from(trailing_delay),
        flush_ends,
        flush_starts,
        pushed: pushed_processed,
        processed_blocks: emitted,
    }
}

fn record_batch(
    blocks: &[MotionBlock],
    (timed, emitted): (&mut [(f64, Trapezoid)], &mut usize),
    mut batch: Vec<(f64, Trapezoid)>,
    pending: &mut VecDeque<PendingDelay>,
    klipper: bool,
) {
    for (block, (time, _)) in blocks[*emitted..].iter().zip(&mut batch) {
        let Some(delay) = pending.front() else {
            break;
        };
        if delay.target == DelayTarget::Any
            || delay.target == DelayTarget::ToolChange && block.kind == MotionKind::ToolChange
        {
            if klipper {
                *time += f64::from(delay.seconds);
            } else {
                *time = f64::from(*time as f32 + delay.seconds);
            }
            pending.pop_front();
        }
    }
    let end = *emitted + batch.len();
    timed[*emitted..end].copy_from_slice(&batch);
    *emitted = end;
}
