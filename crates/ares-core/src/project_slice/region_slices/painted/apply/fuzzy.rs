//! PrintObjectSlice.cpp1033–1141: split actual LayerRegion slices by fuzzy masks.
use super::{ByRegion, ObjectRegions, bounds, clip_error, paths, range_lookup};
use crate::{
    SliceError,
    geometry::{
        ClipperError, CoordinateScale, ExPolygon, FillRule, JoinType, closing_ex,
        difference_polygons_paths, intersection_ex, opening_ex, union_ex,
    },
    project::region_graph::FuzzyRegion,
    project_slice::region_slices::{PostRegionPrintObject, RegionSurface},
};

fn split_layer(
    object: &mut PostRegionPrintObject,
    layer: usize,
    regions: &[FuzzyRegion],
    mask: &[ExPolygon],
    scale: CoordinateScale,
) -> Result<(), ClipperError> {
    let Some(mask_box) = bounds(mask) else {
        return Ok(());
    };
    let mut output = vec![ByRegion::default(); object.regions.len()];
    let mut first = 0;
    for parent in &mut object.regions {
        if first == regions.len() || parent.layers[layer].surfaces.is_empty() {
            continue;
        }
        let Some(index) = regions[first..]
            .iter()
            .position(|region| region.parent_region_id == parent.id)
        else {
            continue;
        };
        first += index;
        let target = &regions[first];
        first += 1;
        let original = std::mem::take(&mut parent.layers[layer].surfaces)
            .into_iter()
            .map(|surface| surface.into_parts().1)
            .collect::<Vec<_>>();
        let parent_box = bounds(&original).unwrap();
        let mut remaining = paths(original.iter().cloned());
        if parent_box.overlap(mask_box) && target.region_id != parent.id {
            output[target.region_id].append(intersection_ex(&original, mask)?);
            remaining = difference_polygons_paths(&remaining, &paths(mask.iter().cloned()))?;
            if !remaining.is_empty() {
                remaining = paths(opening_ex(
                    &union_ex(&remaining, FillRule::NonZero)?,
                    scale.scaled_delta(5.0 * 1e-4) as f32,
                    JoinType::Miter,
                    3.0,
                )?);
            }
        }
        if !remaining.is_empty() {
            output[parent.id].append(union_ex(&remaining, FillRule::NonZero)?);
        }
    }
    for (region, mut data) in object.regions.iter_mut().zip(output) {
        if data.needs_merge {
            data.polygons = closing_ex(
                &data.polygons,
                scale.scaled_delta(10.0 * 1e-4) as f32,
                JoinType::Miter,
                3.0,
            )?;
        }
        region.layers[layer].surfaces = data
            .polygons
            .into_iter()
            .map(RegionSurface::internal)
            .collect();
    }
    Ok(())
}

pub(in crate::project_slice::region_slices::painted) fn apply(
    object: &mut PostRegionPrintObject,
    metadata: &ObjectRegions,
    segmentation: Vec<Vec<Vec<ExPolygon>>>,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    let mut range = range_lookup::first(&metadata.ranges, object.plan.layers[0].slice_z);
    for (layer, masks) in segmentation.into_iter().enumerate() {
        range = range_lookup::next(&metadata.ranges, range, object.plan.layers[layer].slice_z);
        debug_assert_eq!(masks.len(), 1);
        split_layer(object, layer, &metadata.fuzzy[range], &masks[0], scale).map_err(clip_error)?;
    }
    Ok(())
}
