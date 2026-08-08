mod merge;
mod propagate;
mod types;
mod wave_seeds;

pub(crate) use merge::{expand_merge_expolygons, merge_expansions_into_expolygons};
pub(crate) use propagate::{
    expand_expolygons, propagate_waves, propagate_waves_ex,
    propagate_waves_ex_from_sources_with_steps, propagate_waves_from_sources,
    propagate_waves_from_sources_with_steps,
};
#[cfg(test)]
pub(in crate::geometry) use propagate::{wavefront_counter_clockwise, wavefront_step_for_test};
pub(crate) use types::{RegionExpansion, RegionExpansionEx, RegionExpansionParameters, WaveSeed};
#[cfg(all(test, debug_assertions))]
pub(in crate::geometry) use wave_seeds::assert_source_topology_for_test;
pub(crate) use wave_seeds::wave_seeds;
#[cfg(test)]
pub(in crate::geometry) use wave_seeds::{
    bbox_contains_for_test, centroid_for_test, expanded_source_paths_for_test,
    longest_axis_for_test, merge_path_for_test, partition_for_test, reconcile_for_test,
    recover_path_for_test, sample_for_test, sort_seeds_for_test, split_registry_for_test,
};

use super::{ClipperError, CoordinateScale, ExPolygon, Polygon};

type PropagateWavesFn = fn(
    &[WaveSeed],
    &[ExPolygon],
    &RegionExpansionParameters,
) -> Result<Vec<RegionExpansion>, ClipperError>;
type PropagateWavesExFn = fn(
    &[WaveSeed],
    &[ExPolygon],
    &RegionExpansionParameters,
) -> Result<Vec<RegionExpansionEx>, ClipperError>;
type PropagateWavesExFromSourcesWithStepsFn = fn(
    &[ExPolygon],
    &[ExPolygon],
    f32,
    f32,
    usize,
    CoordinateScale,
) -> Result<Vec<RegionExpansionEx>, ClipperError>;
type WaveSeedsFn = fn(
    &[ExPolygon],
    &[ExPolygon],
    f32,
    bool,
    CoordinateScale,
) -> Result<Vec<WaveSeed>, ClipperError>;
type PropagateWavesFromSourcesFn = fn(
    &[ExPolygon],
    &[ExPolygon],
    &RegionExpansionParameters,
    CoordinateScale,
) -> Result<Vec<RegionExpansion>, ClipperError>;
type PropagateWavesFromSourcesWithStepsFn = fn(
    &[ExPolygon],
    &[ExPolygon],
    f32,
    f32,
    usize,
    CoordinateScale,
) -> Result<Vec<RegionExpansion>, ClipperError>;
type ExpandExPolygonsFn = fn(
    &[ExPolygon],
    &[ExPolygon],
    f32,
    f32,
    usize,
    CoordinateScale,
) -> Result<Vec<Vec<Polygon>>, ClipperError>;
type ExpandMergeFn = fn(
    Vec<ExPolygon>,
    &[ExPolygon],
    &RegionExpansionParameters,
    CoordinateScale,
) -> Result<Vec<ExPolygon>, ClipperError>;
type MergeExpansionsFn = fn(
    Vec<ExPolygon>,
    Vec<RegionExpansion>,
    CoordinateScale,
) -> Result<Vec<ExPolygon>, ClipperError>;

const _: fn(f32, f32, usize, CoordinateScale) -> RegionExpansionParameters =
    RegionExpansionParameters::build;
const _: PropagateWavesFn = propagate_waves;
const _: PropagateWavesExFn = propagate_waves_ex;
const _: PropagateWavesExFromSourcesWithStepsFn = propagate_waves_ex_from_sources_with_steps;
const _: WaveSeedsFn = wave_seeds;
const _: PropagateWavesFromSourcesFn = propagate_waves_from_sources;
const _: PropagateWavesFromSourcesWithStepsFn = propagate_waves_from_sources_with_steps;
const _: ExpandExPolygonsFn = expand_expolygons;
const _: MergeExpansionsFn = merge_expansions_into_expolygons;
const _: ExpandMergeFn = expand_merge_expolygons;
