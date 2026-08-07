mod propagate;
mod types;
mod wave_seeds;

pub(crate) use propagate::propagate_waves;
#[cfg(test)]
pub(in crate::geometry) use propagate::{wavefront_counter_clockwise, wavefront_step_for_test};
pub(crate) use types::{RegionExpansion, RegionExpansionParameters, WaveSeed};
#[cfg(all(test, debug_assertions))]
pub(in crate::geometry) use wave_seeds::assert_source_topology_for_test;
pub(crate) use wave_seeds::wave_seeds;
#[cfg(test)]
pub(in crate::geometry) use wave_seeds::{
    bbox_contains_for_test, centroid_for_test, expanded_source_paths_for_test,
    longest_axis_for_test, merge_path_for_test, partition_for_test, reconcile_for_test,
    recover_path_for_test, sample_for_test, sort_seeds_for_test, split_registry_for_test,
};

use super::{ClipperError, CoordinateScale, ExPolygon};

type PropagateWavesFn = fn(
    &[WaveSeed],
    &[ExPolygon],
    &RegionExpansionParameters,
) -> Result<Vec<RegionExpansion>, ClipperError>;
type WaveSeedsFn = fn(
    &[ExPolygon],
    &[ExPolygon],
    f32,
    bool,
    CoordinateScale,
) -> Result<Vec<WaveSeed>, ClipperError>;

const _: fn(f32, f32, usize, CoordinateScale) -> RegionExpansionParameters =
    RegionExpansionParameters::build;
const _: PropagateWavesFn = propagate_waves;
const _: WaveSeedsFn = wave_seeds;
