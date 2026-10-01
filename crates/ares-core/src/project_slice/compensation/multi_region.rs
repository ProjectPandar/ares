use crate::{
    SliceError,
    geometry::{
        ClipperError, CoordinateScale, ExPolygon, FillRule, JoinType, Polygon,
        difference_ex_polygons, intersection_ex, offset_expolygon_refs_paths, offset_expolygons,
        union_ex,
    },
};

use super::{
    super::{
        elephant_foot::compensate_expolygons,
        region_slices::{PostRegion, RegionSurface},
    },
    PreparedObjectCompensation, geometry_error, xy,
};

// PrintObjectSlice.cpp:1314-1365: compensate the merged outline, then split it.
pub(super) fn apply(
    regions: &mut [PostRegion],
    prepared: &PreparedObjectCompensation,
    scale: CoordinateScale,
    closing_epsilon: &mut Option<f32>,
) -> Result<Vec<Vec<ExPolygon>>, SliceError> {
    let max_growth = prepared.xy_contour.max(prepared.xy_hole);
    let min_growth = prepared.xy_contour.min(prepared.xy_hole);
    let scaled_epsilon = scale.scaled_delta(0.0001) as f32;
    let mut backups = vec![Vec::new(); prepared.backup_len];
    for (layer_index, compensation) in prepared.layers.iter().enumerate() {
        if max_growth > 0.0 {
            grow_regions(regions, layer_index, prepared, scaled_epsilon)?;
        }
        if min_growth < 0.0 || compensation.is_some() {
            // Native function-static eps initializes even for negative-growth-only calls.
            let eps = *closing_epsilon.get_or_insert_with(|| {
                (scale.scaled_delta(prepared.closing_radius_mm) * 1.5) as f32
            });
            let mut trimming = if let Some(compensation) = compensation {
                let outline = merged(regions, layer_index, eps)
                    .and_then(|outline| offset_expolygons(&outline, -eps, JoinType::Miter, 3.0))
                    .map_err(|_| geometry_error())?;
                backups[layer_index] = outline.clone();
                compensate_expolygons(
                    &outline,
                    compensation.minimum_width_mm,
                    compensation.compensation_mm,
                    scale,
                )
                .map_err(|_| geometry_error())?
            } else {
                merged(regions, layer_index, scaled_epsilon).map_err(|_| geometry_error())?
            };
            if min_growth < 0.0 {
                trimming = xy::apply(
                    &trimming,
                    prepared.xy_contour.min(0.0),
                    prepared.xy_hole.min(0.0),
                )
                .map_err(|_| geometry_error())?;
            }
            for region in regions.iter_mut() {
                let clipped = intersection_ex(&slices(region, layer_index), &trimming)
                    .map_err(|_| geometry_error())?;
                set_slices(region, layer_index, clipped);
            }
        }
    }
    Ok(backups)
}

fn grow_regions(
    regions: &mut [PostRegion],
    layer_index: usize,
    prepared: &PreparedObjectCompensation,
    scaled_epsilon: f32,
) -> Result<(), SliceError> {
    let outline = merged(regions, layer_index, scaled_epsilon)
        .and_then(|outline| union_ex(&paths(&outline), FillRule::NonZero))
        .and_then(|outline| {
            xy::apply(
                &outline,
                prepared.xy_contour.max(0.0),
                prepared.xy_hole.max(0.0),
            )
        })
        .map_err(|_| geometry_error())?;
    let max_growth = prepared.xy_contour.max(prepared.xy_hole);
    let mut processed = Vec::new();
    let region_count = regions.len();
    for (region_index, region) in regions.iter_mut().enumerate() {
        let mut slices = offset_expolygons(
            &slices(region, layer_index),
            max_growth,
            JoinType::Miter,
            3.0,
        )
        .and_then(|grown| intersection_ex(&grown, &outline))
        .map_err(|_| geometry_error())?;
        if region_index > 0 {
            slices = difference_ex_polygons(&slices, &processed).map_err(|_| geometry_error())?;
        }
        if region_index + 1 < region_count {
            processed.extend(paths(&slices));
        }
        set_slices(region, layer_index, slices);
    }
    Ok(())
}

fn slices(region: &PostRegion, layer_index: usize) -> Vec<ExPolygon> {
    region.layers[layer_index]
        .surfaces
        .iter()
        .map(|surface| surface.as_parts().1.clone())
        .collect()
}

fn set_slices(region: &mut PostRegion, layer_index: usize, slices: Vec<ExPolygon>) {
    region.layers[layer_index].surfaces = slices.into_iter().map(RegionSurface::internal).collect();
}

fn paths(slices: &[ExPolygon]) -> Vec<Polygon> {
    slices
        .iter()
        .flat_map(|polygon| std::iter::once(polygon.contour()).chain(polygon.holes()))
        .cloned()
        .collect()
}

// Layer.cpp:119-138::merged excludes subtractor regions with no extrusion.
fn merged(
    regions: &[PostRegion],
    layer_index: usize,
    offset: f32,
) -> Result<Vec<ExPolygon>, ClipperError> {
    let delta = if offset == 0.0 { 0.0001_f32 } else { offset };
    let mut polygons = Vec::new();
    for region in regions {
        let options = &region.options;
        if options.bottom_shell_layers.0 > 0
            || options.top_shell_layers.0 > 0
            || options.sparse_infill_density.0 > 0.0
            || options.wall_loops.0 > 0
        {
            let slices = region.layers[layer_index]
                .surfaces
                .iter()
                .map(|surface| surface.as_parts().1)
                .collect::<Vec<_>>();
            polygons.extend(offset_expolygon_refs_paths(
                &slices,
                delta,
                JoinType::Miter,
                3.0,
            )?);
        }
    }
    let outline = union_ex(&polygons, FillRule::NonZero)?;
    if offset == 0.0 {
        offset_expolygons(&outline, -0.0001_f32, JoinType::Miter, 3.0)
    } else {
        Ok(outline)
    }
}
