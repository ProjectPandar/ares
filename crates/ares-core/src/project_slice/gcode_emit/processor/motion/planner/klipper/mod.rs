//! Klipper `TimeMachine` queue and toolhead windows (`GCodeProcessor.cpp:505-637,
//! 5665-5671,8136-8146`). Marlin's fixed-block queue remains separate.
mod plan;
mod prepare;

use super::super::{KlipperConfig, MotionBlock, MotionKind};
use super::Trapezoid;

#[derive(Clone, Copy, Default)]
struct Block {
    distance: f32,
    acceleration: f32,
    cruise: f32,
    max_entry: f32,
    max_mcr_entry_sqr: f32,
    mcr_delta_v2: f32,
    junction_deviation: f32,
    axis_feedrate: [f64; 4],
    direction: [f32; 4],
    entry: f32,
    exit: f32,
    planned_cruise: f32,
    peak_cruise_v2: f32,
}

pub(in crate::project_slice::gcode_emit::processor) struct KlipperPlanner {
    blocks: Vec<Block>,
    previous: Option<Block>,
    config: KlipperConfig,
    junction_flush: f32,
    priming: bool,
}

impl KlipperPlanner {
    pub(super) fn new() -> Self {
        Self {
            blocks: Vec::new(),
            previous: None,
            config: KlipperConfig::default(),
            junction_flush: 1.0,
            priming: true,
        }
    }

    fn configure(&mut self, config: KlipperConfig) {
        // Between pushes/synchronizations there is no automatic flush. While
        // priming, legacy/modern transitions telescope to this net adjustment.
        if self.priming && config.legacy() != self.config.legacy() {
            self.junction_flush += if config.legacy() { 1.0 } else { -1.0 };
        }
        self.config = config;
    }

    pub(super) fn push(&mut self, motion: &MotionBlock) -> Vec<(f64, Trapezoid)> {
        self.configure(motion.klipper);
        let block = if motion.kind == MotionKind::ToolChange {
            Block::default()
        } else {
            let block = prepare::block(motion, self.previous, self.blocks.last().copied());
            self.previous = Some(block);
            block
        };
        self.blocks.push(block);
        if motion.kind != MotionKind::ToolChange && !motion.virtual_motion && self.blocks.len() > 1
        {
            self.junction_flush -= block.distance / block.cruise;
        }
        if motion.kind != MotionKind::ToolChange
            && !motion.virtual_motion
            && self.junction_flush <= 0.0
        {
            self.process(true)
        } else {
            Vec::new()
        }
    }

    pub(super) fn flush(&mut self, config: KlipperConfig) -> Vec<(f64, Trapezoid)> {
        self.configure(config);
        let batch = self.process(false);
        self.junction_flush = if self.config.legacy() { 2.0 } else { 1.0 };
        self.priming = true;
        batch
    }

    pub(super) fn finish(&mut self) -> Vec<(f64, Trapezoid)> {
        self.process(false)
    }

    fn process(&mut self, lazy: bool) -> Vec<(f64, Trapezoid)> {
        if self.blocks.is_empty() {
            return Vec::new();
        }
        self.junction_flush = if self.config.legacy() { 0.250 } else { 0.150 };
        self.priming = false;
        let count = plan::plan(&mut self.blocks, lazy);
        self.blocks
            .drain(..count)
            .map(|block| {
                let start_v2 = block.entry.min(block.planned_cruise);
                let end_v2 = block.exit.min(block.planned_cruise);
                let inv_two_accel = if block.acceleration > 0.0 {
                    0.5 / block.acceleration
                } else {
                    0.0
                };
                let trapezoid = Trapezoid {
                    distance: block.distance,
                    entry: start_v2.sqrt(),
                    exit: end_v2.sqrt(),
                    cruise_feedrate: block.planned_cruise.sqrt(),
                    accelerate_until: (block.planned_cruise - start_v2) * inv_two_accel,
                    decelerate_after: block.distance
                        - (block.planned_cruise - end_v2) * inv_two_accel,
                };
                let cruise = f64::from(trapezoid.cruise_feedrate);
                let mut time = if cruise > 0.0 {
                    f64::from(block.distance) / cruise
                } else {
                    0.0
                };
                if cruise > 0.0 && block.acceleration > 0.0 {
                    let accel = cruise - f64::from(trapezoid.entry);
                    let decel = cruise - f64::from(trapezoid.exit);
                    time += (accel * accel + decel * decel)
                        / (2.0 * f64::from(block.acceleration) * cruise);
                }
                (time, trapezoid)
            })
            .collect()
    }
}
