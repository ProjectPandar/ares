//! PrintObjectSlice.cpp874–1030: consume shared painted-region identities and replace real slices.
pub(super) mod fuzzy;

use super::super::range_lookup;
use super::super::{PostRegionPrintObject, RegionSurface};
use super::{clip_error, math::paths};
use crate::geometry::difference_polygons_paths;
use crate::project::region_graph::{FuzzyRegion, PaintedRegion, PaintedRegions};
use crate::project_slice::volume_regions::{VolumeRegionGraph, VolumeRegionRange};
use crate::{
    SliceError,
    geometry::{
        BoundingBox, ClipperError, CoordinateScale, ExPolygon, FillRule, JoinType, closing_ex,
        intersection_ex, opening_ex, union_ex,
    },
};

pub(in crate::project_slice) struct ObjectRegions {
    pub(in crate::project_slice) active: bool,
    pub(super) ranges: Vec<VolumeRegionRange>,
    painted: Vec<Vec<PaintedRegion>>,
    pub(super) fuzzy: Vec<Vec<FuzzyRegion>>,
}
impl ObjectRegions {
    pub(in crate::project_slice) fn new(
        graph: &VolumeRegionGraph,
        painted: PaintedRegions,
        active: bool,
    ) -> Self {
        Self {
            active,
            ranges: if active || painted.fuzzy.iter().any(|regions| !regions.is_empty()) {
                graph.ranges.clone()
            } else {
                Vec::new()
            },
            painted: painted.mmu,
            fuzzy: painted.fuzzy,
        }
    }
}

#[derive(Clone, Default)]
struct ByRegion {
    polygons: Vec<ExPolygon>,
    needs_merge: bool,
}
impl ByRegion {
    fn append(&mut self, mut polygons: Vec<ExPolygon>) {
        if polygons.is_empty() {
            return;
        }
        if self.polygons.is_empty() {
            self.polygons = polygons;
        } else {
            self.polygons.append(&mut polygons);
            self.needs_merge = true;
        }
    }
}
fn bounds(polygons: &[ExPolygon]) -> Option<BoundingBox> {
    BoundingBox::from_points(
        polygons
            .iter()
            .flat_map(|p| p.contour().points().iter().copied()),
    )
}

fn split_layer(
    object: &mut PostRegionPrintObject,
    layer: usize,
    (ranges, painted): (&VolumeRegionRange, &[PaintedRegion]),
    colors: Vec<Vec<ExPolygon>>,
    scale: CoordinateScale,
) -> Result<(), ClipperError> {
    let boxes = colors.iter().map(|p| bounds(p)).collect::<Vec<_>>();
    if boxes.iter().all(Option::is_none) {
        return Ok(());
    }
    let mut output = vec![ByRegion::default(); object.regions.len()];
    let mut first = 0;
    for parent in &mut object.regions {
        if first == painted.len() || parent.layers[layer].surfaces.is_empty() {
            continue;
        }
        let Some(found) = painted[first..]
            .iter()
            .position(|p| ranges.volume_regions[p.parent].region_id == Some(parent.id))
        else {
            continue;
        };
        first += found;
        let original = std::mem::take(&mut parent.layers[layer].surfaces)
            .into_iter()
            .map(|s| s.into_parts().1)
            .collect::<Vec<_>>();
        let parent_box = bounds(&original).unwrap();
        let mut self_color = None;
        for (index, (color, bbox)) in colors.iter().zip(&boxes).enumerate() {
            if !bbox.is_some_and(|bbox| parent_box.overlap(bbox)) {
                continue;
            }
            first += painted[first..]
                .iter()
                .position(|p| p.extruder > index)
                .unwrap();
            let target = &painted[first];
            debug_assert_eq!(
                ranges.volume_regions[target.parent].region_id,
                Some(parent.id)
            );
            debug_assert_eq!(target.extruder, index + 1);
            if target.region_id == parent.id {
                self_color = Some(index);
                continue;
            }
            output[target.region_id].append(intersection_ex(&original, color)?);
        }
        let mut mine = paths(original);
        for (index, (color, bbox)) in colors.iter().zip(&boxes).enumerate() {
            if self_color == Some(index) || !bbox.is_some_and(|bbox| parent_box.overlap(bbox)) {
                continue;
            }
            mine = difference_polygons_paths(&mine, &paths(color.iter().cloned()))?;
            if mine.is_empty() {
                break;
            }
        }
        if !mine.is_empty() {
            let delta = scale.scaled_delta(5.0 * 1e-4) as f32;
            mine = paths(opening_ex(
                &union_ex(&mine, FillRule::NonZero)?,
                delta,
                JoinType::Miter,
                3.0,
            )?);
        }
        if !mine.is_empty() {
            output[parent.id].append(union_ex(&mine, FillRule::NonZero)?);
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
pub(super) fn apply(
    object: &mut PostRegionPrintObject,
    metadata: &ObjectRegions,
    segmentation: Vec<Vec<Vec<ExPolygon>>>,
    extruders: usize,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    let mut range = range_lookup::first(&metadata.ranges, object.plan.layers[0].slice_z);
    for (index, mut colors) in segmentation.into_iter().enumerate() {
        range = range_lookup::next(&metadata.ranges, range, object.plan.layers[index].slice_z);
        let colors = colors.drain(..extruders).collect();
        split_layer(
            object,
            index,
            (&metadata.ranges[range], &metadata.painted[range]),
            colors,
            scale,
        )
        .map_err(clip_error)?;
    }
    Ok(())
}
