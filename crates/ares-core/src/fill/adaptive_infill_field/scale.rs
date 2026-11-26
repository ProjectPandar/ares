//! Gradient scaling helpers (`AdaptiveInfillField.cpp:149-211`):
//! `target_scale`, `DensityLevels`, and `RadialScale`.

use crate::ProcessAdaptiveInfillGradient;

use super::AdaptiveTpms;

const EPSILON: f64 = 1.0e-4;

pub(super) fn target_scale(ratio: f64, gradient: ProcessAdaptiveInfillGradient, depth: f64) -> f64 {
    match gradient {
        ProcessAdaptiveInfillGradient::Quadratic => 1.0 + (ratio - 1.0) * depth * depth,
        ProcessAdaptiveInfillGradient::Exponential => ratio.powf(depth),
        ProcessAdaptiveInfillGradient::Linear => 1.0 + (ratio - 1.0) * depth,
    }
}

pub(super) struct DensityLevels {
    pub(super) ratio: f64,
    pub(super) gradient: ProcessAdaptiveInfillGradient,
    pub(super) count: i32,
}

impl DensityLevels {
    pub(super) fn new(ratio: f64, step: f64, gradient: ProcessAdaptiveInfillGradient) -> Self {
        let count = (ratio.ln().abs() / step.ln() - EPSILON).ceil() as i32;
        Self {
            ratio,
            gradient,
            count,
        }
    }

    pub(super) fn scale(&self, level: i32) -> f64 {
        if self.count == 0 {
            1.0
        } else {
            self.ratio.powf(f64::from(level) / f64::from(self.count))
        }
    }

    pub(super) fn level(&self, depth: f64) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            f64::from(self.count) * target_scale(self.ratio, self.gradient, depth).ln()
                / self.ratio.ln()
        }
    }
}

pub(super) struct RadialScale {
    dimensions: i32,
    scale: [f64; 257],
}

impl RadialScale {
    pub(super) fn new(tpms: AdaptiveTpms, dimensions: i32) -> Self {
        let ratio = (tpms.interior_frequency / tpms.surface_frequency).max(1e-3);
        let target = |depth: f64| target_scale(ratio, tpms.gradient, depth);
        let mut scale = [0.0_f64; 257];
        scale[0] = target(1.0);
        let mut volume = 0.0_f64;
        let last = (scale.len() - 1) as f64;
        for (i, slot) in scale.iter_mut().enumerate().skip(1) {
            let t0 = (i - 1) as f64 / last;
            let t1 = i as f64 / last;
            let v1 = pow_dim(t1, dimensions);
            let v0 = pow_dim(t0, dimensions);
            volume += (v1 - v0) * target(1.0 - 0.5 * (t0 + t1));
            *slot = volume / v1;
        }
        Self { dimensions, scale }
    }

    pub(super) fn evaluate(&self, t: f64) -> f64 {
        if t >= 1.0 {
            let volume = pow_dim(t, self.dimensions);
            return (self.scale[self.scale.len() - 1] + volume - 1.0) / volume;
        }
        let x = t * (self.scale.len() - 1) as f64;
        let i = (x as usize).min(self.scale.len() - 2);
        self.scale[i] + (self.scale[i + 1] - self.scale[i]) * (x - i as f64)
    }
}

fn pow_dim(t: f64, dimensions: i32) -> f64 {
    if dimensions == 3 { t * t * t } else { t * t }
}
