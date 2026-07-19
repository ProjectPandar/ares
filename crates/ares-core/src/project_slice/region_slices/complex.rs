use crate::{
    ProjectVolumeType, SliceError,
    geometry::{
        ClipperError, CoordinateScale, ExPolygon, JoinType, difference_ex, intersection_ex,
        offset2_ex,
    },
};

use super::{
    PendingRegionSlices, PostRegionPrintObject, RegionSurface, VolumeBound, VolumeOccurrenceId,
    VolumeSlices,
};
use crate::project_slice::volume_regions::VolumeRegion;

pub(in crate::project_slice) fn compose_complex_region_slices(
    pending: PendingRegionSlices,
    scale: CoordinateScale,
) -> Result<PostRegionPrintObject, SliceError> {
    let (mut output, mut working, bounds, records, complex_layers) = pending.into_parts();
    for layer_index in complex_layers {
        let mut slices = records
            .iter()
            .map(|record| TempSlice {
                region_id: record.region_id,
                occurrence_id: record.occurrence_id,
                expolygons: take_layer(&mut working, record.occurrence_id, layer_index),
            })
            .collect::<Vec<_>>();
        for index in 0..records.len() {
            if slices[index].expolygons.is_empty() {
                continue;
            }
            if records[index].kind == ProjectVolumeType::ParameterModifier {
                partition_modifier(&records, &mut slices, index).map_err(map_clipper_error)?;
                continue;
            }
            if !matches!(
                records[index].kind,
                ProjectVolumeType::ModelPart | ProjectVolumeType::NegativeVolume
            ) {
                unreachable!("region graph cannot contain support records");
            }
            subtract_from_predecessors(&records, &bounds, &mut slices, index)
                .map_err(map_clipper_error)?;
        }
        write_regions(&mut output, slices, layer_index, scale).map_err(map_clipper_error)?;
    }
    Ok(output)
}

fn subtract_from_predecessors(
    records: &[VolumeRegion],
    bounds: &[VolumeBound],
    slices: &mut [TempSlice],
    index: usize,
) -> Result<(), ClipperError> {
    let (predecessors, current_and_tail) = slices.split_at_mut(index);
    let current = &current_and_tail[0].expolygons;
    for (predecessor, record) in predecessors.iter_mut().zip(records) {
        if predecessor.expolygons.is_empty()
            || record.kind == ProjectVolumeType::NegativeVolume
            || !bounds[records[index].bound_index]
                .bbox()
                .intersects_xy(bounds[record.bound_index].bbox())
        {
            continue;
        }
        predecessor.expolygons = difference_ex(&predecessor.expolygons, current)?;
    }
    Ok(())
}

struct TempSlice {
    region_id: Option<usize>,
    occurrence_id: VolumeOccurrenceId,
    expolygons: Vec<ExPolygon>,
}

impl TempSlice {
    fn valid_key(&self) -> Option<(usize, VolumeOccurrenceId)> {
        (!self.expolygons.is_empty()).then_some((self.region_id?, self.occurrence_id))
    }
}

fn write_regions(
    output: &mut PostRegionPrintObject,
    mut slices: Vec<TempSlice>,
    layer_index: usize,
    scale: CoordinateScale,
) -> Result<(), ClipperError> {
    slices.sort_by(|left, right| match (left.valid_key(), right.valid_key()) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    });
    let mut index = 0;
    while index < slices.len() {
        let Some((region_id, _)) = slices[index].valid_key() else {
            break;
        };
        let mut expolygons = std::mem::take(&mut slices[index].expolygons);
        let mut merged = false;
        let mut end = index + 1;
        while end < slices.len()
            && slices[end]
                .valid_key()
                .is_some_and(|(next_region, _)| next_region == region_id)
        {
            expolygons.append(&mut slices[end].expolygons);
            merged = true;
            end += 1;
        }
        if merged {
            let delta = (1e-4_f64 / scale.factor()) as f32;
            expolygons = offset2_ex(&expolygons, delta, -delta, JoinType::Miter, 3.0)?;
        }
        output.regions[region_id].layers[layer_index]
            .surfaces
            .extend(expolygons.into_iter().map(RegionSurface::internal));
        index = end;
    }
    Ok(())
}

fn partition_modifier(
    records: &[VolumeRegion],
    slices: &mut [TempSlice],
    index: usize,
) -> Result<(), ClipperError> {
    let parent_index = records[index]
        .parent
        .expect("modifier record must retain its parent");
    let source = {
        let (predecessors, current_and_tail) = slices.split_at_mut(index);
        let parent = &mut predecessors[parent_index];
        let current = &mut current_and_tail[0];
        let source = std::mem::take(&mut current.expolygons);
        if !parent.expolygons.is_empty() {
            current.expolygons = intersection_ex(&parent.expolygons, &source)?;
            parent.expolygons = difference_ex(&parent.expolygons, &source)?;
        }
        source
    };
    if records
        .get(index + 1)
        .is_some_and(|next| next.occurrence_id == records[index].occurrence_id)
    {
        slices[index + 1].expolygons = source;
    }
    Ok(())
}

fn map_clipper_error(error: ClipperError) -> SliceError {
    match error {
        ClipperError::CoordinateOutOfRange => SliceError::InvalidInput(
            "project region composition polygon coordinate is outside the supported Clipper range"
                .to_owned(),
        ),
    }
}

fn take_layer(
    working: &mut [VolumeSlices],
    occurrence_id: super::VolumeOccurrenceId,
    layer_index: usize,
) -> Vec<ExPolygon> {
    let index = working
        .binary_search_by_key(&occurrence_id, |volume| volume.occurrence_id)
        .expect("region record must retain a physical carrier");
    std::mem::take(&mut working[index].layers[layer_index])
}

const _: fn(PendingRegionSlices, CoordinateScale) -> Result<PostRegionPrintObject, SliceError> =
    compose_complex_region_slices;
