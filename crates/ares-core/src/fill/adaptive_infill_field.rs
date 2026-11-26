//! `libslic3r/Fill/AdaptiveInfillField.{hpp,cpp}` (`40ecc66f7`): radial and
//! depth adaptive density field for TPMS sparse infill patterns.

mod build;
mod grid;
mod query;
mod reach;
mod scale;
pub(crate) mod tpms;

use std::sync::Arc;

use crate::{
    ProcessAdaptiveInfillGradient, ProcessAdaptiveInfillMode, ProcessInfillPattern,
    geometry::ExPolygon,
};

pub(crate) const MAX_MORPH: usize = 4;
pub(crate) const POLAR: usize = 24;
pub(crate) const AZIMUTH: usize = 48;
pub(crate) const MODE_COUNT: usize = 8;

/// `PrintConfig.hpp:177` `is_adaptive_infill_pattern`.
pub(crate) const fn is_adaptive_infill_pattern(pattern: ProcessInfillPattern) -> bool {
    matches!(
        pattern,
        ProcessInfillPattern::Gyroid | ProcessInfillPattern::TpmsD | ProcessInfillPattern::TpmsFk
    )
}

pub(crate) const fn mode_index(mode: ProcessAdaptiveInfillMode) -> usize {
    match mode {
        ProcessAdaptiveInfillMode::Disabled => 0,
        ProcessAdaptiveInfillMode::DistanceWarp => 1,
        ProcessAdaptiveInfillMode::SmoothBlend => 2,
        ProcessAdaptiveInfillMode::SteppedShells => 3,
        ProcessAdaptiveInfillMode::Lobes => 4,
        ProcessAdaptiveInfillMode::NormalZ => 5,
        ProcessAdaptiveInfillMode::NormalY => 6,
        ProcessAdaptiveInfillMode::NormalX => 7,
    }
}

pub(crate) const fn mode_from_index(index: usize) -> Option<ProcessAdaptiveInfillMode> {
    match index {
        0 => Some(ProcessAdaptiveInfillMode::Disabled),
        1 => Some(ProcessAdaptiveInfillMode::DistanceWarp),
        2 => Some(ProcessAdaptiveInfillMode::SmoothBlend),
        3 => Some(ProcessAdaptiveInfillMode::SteppedShells),
        4 => Some(ProcessAdaptiveInfillMode::Lobes),
        5 => Some(ProcessAdaptiveInfillMode::NormalZ),
        6 => Some(ProcessAdaptiveInfillMode::NormalY),
        7 => Some(ProcessAdaptiveInfillMode::NormalX),
        _ => None,
    }
}

pub(crate) struct Slice<'a> {
    pub(crate) bottom_z: f64,
    pub(crate) top_z: f64,
    pub(crate) expolygons: &'a [ExPolygon],
}

#[derive(Clone, Copy, Default)]
pub(crate) struct Radial {
    pub(crate) center: [f64; 3],
    pub(crate) t: f64,
    pub(crate) weight: f32,
}

pub(crate) type Radials = [Radial; 2 * MAX_MORPH];

pub(super) struct Lobe {
    pub(super) center: [f64; 3],
    pub(super) depth: f64,
    pub(super) reach: Vec<f32>,
    pub(super) mean_depth: Vec<f32>,
}

pub(super) struct Body {
    pub(super) first_lobe: usize,
    pub(super) lobes: usize,
    pub(super) depth: f64,
}

pub(crate) struct AdaptiveInfillField {
    pub(super) mode: ProcessAdaptiveInfillMode,
    pub(super) axis: i32,
    pub(super) origin: [f64; 3],
    pub(super) cell: f64,
    pub(super) size: [i32; 3],
    pub(super) body: Vec<i32>,
    pub(super) bodies: Vec<Body>,
    pub(super) lobes: Vec<Lobe>,
    pub(super) section: Vec<i32>,
    pub(super) depth: Vec<f32>,
}

#[derive(Clone, Default)]
pub(crate) struct AdaptiveInfillFields {
    fields: [Option<Arc<AdaptiveInfillField>>; MODE_COUNT],
}

impl AdaptiveInfillFields {
    pub(crate) fn new(fields: [Option<Arc<AdaptiveInfillField>>; MODE_COUNT]) -> Self {
        Self { fields }
    }

    pub(crate) fn get(&self, mode: ProcessAdaptiveInfillMode) -> Option<&AdaptiveInfillField> {
        self.fields[mode_index(mode)].as_deref()
    }
}

#[derive(Clone, Copy)]
pub(crate) struct AdaptiveTpms {
    pub(crate) equation: fn(f32, f32, f32) -> f32,
    pub(crate) surface_frequency: f64,
    pub(crate) interior_frequency: f64,
    pub(crate) gradient: ProcessAdaptiveInfillGradient,
}
