//! Incremental `TimeMachine::calculate_time` admission and delay attribution.

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

pub(super) struct TimeMachine {
    planner: Planner,
    kinds: VecDeque<MotionKind>,
    pending: VecDeque<PendingDelay>,
}

impl TimeMachine {
    pub(super) fn new(flavor: GCodeFlavor) -> Self {
        Self {
            planner: Planner::new(flavor),
            kinds: VecDeque::new(),
            pending: VecDeque::new(),
        }
    }

    /// The flavor-specific planner flushes its committed prefix and keeps
    /// only the upstream look-ahead queue (`GCodeProcessor.cpp:5685-5692`).
    pub(super) fn push(&mut self, block: &MotionBlock) -> Vec<(f64, Trapezoid)> {
        self.kinds.push_back(block.kind);
        let batch = self.planner.push(block);
        self.attribute(batch)
    }

    pub(super) fn flush(
        &mut self,
        config: KlipperConfig,
        delay: Option<PendingDelay>,
    ) -> Vec<(f64, Trapezoid)> {
        if let Some(delay) = delay.filter(|delay| delay.seconds > 0.0) {
            if let Some(last) = self
                .pending
                .back_mut()
                .filter(|last| last.target == delay.target)
            {
                last.seconds += delay.seconds;
            } else {
                self.pending.push_back(delay);
            }
        }
        let batch = self.planner.flush(config);
        self.attribute(batch)
    }

    /// Final drain, including delays with no later matching move
    /// (`GCodeProcessor.cpp:4000-4002,760-775`).
    pub(super) fn finish(&mut self) -> (Vec<(f64, Trapezoid)>, f64) {
        let batch = self.planner.finish(!self.pending.is_empty());
        let batch = self.attribute(batch);
        let trailing = self
            .pending
            .drain(..)
            .fold(0.0_f32, |sum, delay| sum + delay.seconds);
        (batch, f64::from(trailing))
    }

    fn attribute(&mut self, mut batch: Vec<(f64, Trapezoid)>) -> Vec<(f64, Trapezoid)> {
        for (time, _) in &mut batch {
            let kind = self.kinds.pop_front().unwrap();
            if let Some(delay) = self.pending.front()
                && (delay.target == DelayTarget::Any
                    || delay.target == DelayTarget::ToolChange && kind == MotionKind::ToolChange)
            {
                *time += f64::from(delay.seconds);
                self.pending.pop_front();
            }
        }
        batch
    }
}
