use std::collections::VecDeque;

use super::super::motion::{KlipperConfig, MotionBlock, Trapezoid};
use super::super::moves::MoveRecorder;
use super::super::schedule::{PendingDelay, TimeMachine};
use super::super::stop_times::StopTimes;
use super::custom_time::CustomTimes;
use super::export_cache::Entry;
use crate::GCodeFlavor;
use crate::gcode_processor::{CustomGCodeTime, CustomGCodeType};

pub(super) struct Metadata {
    pub(super) id: usize,
    pub(super) remaining: usize,
    pub(super) prepare: bool,
    pub(super) eligible: bool,
}

pub(super) struct Timing {
    machine: TimeMachine,
    pending: VecDeque<Metadata>,
    pub(super) total: f64,
    pub(super) prepare: f32,
    pub(super) cache: Vec<Entry>,
    pub(super) stop_times: StopTimes,
    custom: CustomTimes,
    pub(super) custom_gcode_times: Vec<CustomGCodeTime>,
}

impl Timing {
    pub(super) fn new(flavor: GCodeFlavor) -> Self {
        Self {
            machine: TimeMachine::new(flavor),
            pending: VecDeque::new(),
            total: 0.0,
            prepare: 0.0,
            cache: Vec::new(),
            stop_times: StopTimes::default(),
            custom: CustomTimes::default(),
            custom_gcode_times: Vec::new(),
        }
    }

    pub(super) fn push(
        &mut self,
        block: &MotionBlock,
        metadata: Metadata,
        recorder: &mut MoveRecorder,
    ) {
        self.pending.push_back(metadata);
        let batch = self.machine.push(block);
        self.commit(&batch, recorder, true);
    }

    pub(super) fn flush(
        &mut self,
        config: KlipperConfig,
        delay: Option<PendingDelay>,
        recorder: &mut MoveRecorder,
    ) {
        let batch = self.machine.flush(config, delay);
        self.commit(&batch, recorder, true);
    }

    pub(super) fn custom(
        &mut self,
        id: usize,
        kind: CustomGCodeType,
        config: KlipperConfig,
        recorder: &mut MoveRecorder,
    ) {
        self.stop_times.register(id);
        self.flush(config, None, recorder);
        self.custom.register(kind);
    }

    pub(super) fn finish(&mut self, recorder: &mut MoveRecorder) {
        let (batch, trailing) = self.machine.finish();
        self.commit(&batch, recorder, true);
        // A final Marlin block below the planner minimum keeps zero-time
        // lookup metadata, without registering a processed stop time.
        let unprocessed = vec![(0.0, Trapezoid::default()); self.pending.len()];
        self.commit(&unprocessed, recorder, false);
        self.total += trailing;
        self.custom_gcode_times = std::mem::take(&mut self.custom).finish(trailing, self.total);
    }

    fn commit(&mut self, batch: &[(f64, Trapezoid)], recorder: &mut MoveRecorder, processed: bool) {
        if batch.is_empty() {
            return;
        }
        let eligible = recorder.apply_batch(batch);
        for (&(time, _), override_eligible) in batch.iter().zip(eligible) {
            let metadata = self.pending.pop_front().unwrap();
            self.total += time;
            self.custom.add(time);
            if metadata.prepare {
                self.prepare = (f64::from(self.prepare) + time) as f32;
            }
            if !override_eligible.unwrap_or(metadata.eligible) {
                continue;
            }
            if processed {
                self.stop_times.record(metadata.id, self.total as f32);
            }
            self.cache.push(Entry {
                id: metadata.id,
                remaining: metadata.remaining,
                elapsed: self.total as f32,
            });
        }
    }
}
