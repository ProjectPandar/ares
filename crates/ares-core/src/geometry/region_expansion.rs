mod propagate;
mod types;

pub(crate) use propagate::propagate_waves;
#[cfg(test)]
pub(in crate::geometry) use propagate::{wavefront_counter_clockwise, wavefront_step_for_test};
pub(crate) use types::{RegionExpansion, RegionExpansionParameters, WaveSeed};

use super::{ClipperError, CoordinateScale, ExPolygon};

type PropagateWavesFn = fn(
    &[WaveSeed],
    &[ExPolygon],
    &RegionExpansionParameters,
) -> Result<Vec<RegionExpansion>, ClipperError>;

const _: fn(f32, f32, usize, CoordinateScale) -> RegionExpansionParameters =
    RegionExpansionParameters::build;
const _: PropagateWavesFn = propagate_waves;
