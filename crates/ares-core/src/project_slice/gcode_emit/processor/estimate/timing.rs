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
    pub(super) first_layer: bool,
    pub(super) eligible: bool,
}

pub(super) struct Timing {
    machine: TimeMachine,
    pending: VecDeque<Metadata>,
    batch: Vec<(f64, Trapezoid)>,
    eligible: Vec<Option<bool>>,
    pub(super) total: f64,
    pub(super) prepare: f32,
    pub(super) initial_layer_time: f32,
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
            batch: Vec::new(),
            eligible: Vec::new(),
            total: 0.0,
            prepare: 0.0,
            initial_layer_time: 0.0,
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
        self.batch.clear();
        self.machine.push(block, &mut self.batch);
        self.commit(recorder, true);
    }

    pub(super) fn flush(
        &mut self,
        config: KlipperConfig,
        delay: Option<PendingDelay>,
        recorder: &mut MoveRecorder,
    ) {
        self.batch.clear();
        self.machine.flush(config, delay, &mut self.batch);
        self.commit(recorder, true);
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
        self.batch.clear();
        let trailing = self.machine.finish(&mut self.batch);
        self.commit(recorder, true);
        // A final Marlin block below the planner minimum keeps zero-time
        // lookup metadata, without registering a processed stop time.
        self.batch.clear();
        self.batch
            .resize(self.pending.len(), (0.0, Trapezoid::default()));
        self.commit(recorder, false);
        self.total += trailing;
        self.custom_gcode_times = std::mem::take(&mut self.custom).finish(trailing, self.total);
    }

    fn commit(&mut self, recorder: &mut MoveRecorder, processed: bool) {
        if self.batch.is_empty() {
            return;
        }
        let tracks_eligibility = recorder.tracks_eligibility();
        if tracks_eligibility {
            recorder.apply_batch(&self.batch, &mut self.eligible);
        }
        for (index, &(time, _)) in self.batch.iter().enumerate() {
            let metadata = self.pending.pop_front().unwrap();
            self.total += time;
            self.custom.add(time);
            if metadata.prepare {
                self.prepare = (f64::from(self.prepare) + time) as f32;
            }
            if metadata.first_layer {
                self.initial_layer_time = (f64::from(self.initial_layer_time) + time) as f32;
            }
            let override_eligible = if tracks_eligibility {
                self.eligible[index]
            } else {
                None
            };
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
