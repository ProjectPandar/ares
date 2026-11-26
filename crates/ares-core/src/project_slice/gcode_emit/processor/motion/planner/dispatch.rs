//! `GCodeProcessor.cpp:8058`: dispatch by firmware flavor, without fallback.
use super::klipper::KlipperPlanner;
use super::{RollingPlanner, Trapezoid};
use crate::GCodeFlavor;
use crate::project_slice::gcode_emit::processor::motion::{KlipperConfig, MotionBlock};

pub(in crate::project_slice::gcode_emit::processor) enum Planner {
    Marlin(RollingPlanner),
    Klipper(KlipperPlanner),
}

impl Planner {
    pub(in crate::project_slice::gcode_emit::processor) fn new(flavor: GCodeFlavor) -> Self {
        if flavor == GCodeFlavor::Klipper {
            Self::Klipper(KlipperPlanner::new())
        } else {
            Self::Marlin(RollingPlanner::new())
        }
    }

    pub(in crate::project_slice::gcode_emit::processor) fn push(
        &mut self,
        block: &MotionBlock,
        out: &mut Vec<(f64, Trapezoid)>,
    ) {
        match self {
            Self::Marlin(planner) => planner.push(block, out),
            Self::Klipper(planner) => planner.push(block, out),
        }
    }

    pub(in crate::project_slice::gcode_emit::processor) fn flush(
        &mut self,
        config: KlipperConfig,
        out: &mut Vec<(f64, Trapezoid)>,
    ) {
        match self {
            Self::Marlin(planner) => planner.flush(out),
            Self::Klipper(planner) => planner.flush(config, out),
        }
    }

    pub(in crate::project_slice::gcode_emit::processor) fn finish(
        &mut self,
        delay_buffered: bool,
        out: &mut Vec<(f64, Trapezoid)>,
    ) {
        match self {
            Self::Marlin(planner) => planner.finish(delay_buffered, out),
            Self::Klipper(planner) => planner.finish(out),
        }
    }
}
