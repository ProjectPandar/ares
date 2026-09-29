//! `PrintObject::bridge_over_infill` "LIGHTNING INFILL SECTION"
//! (`PrintObject.cpp:2593-2721`): when lightning sparse infill is used, layers
//! whose `lower_layer` has `stInternal` inside a lightning region get their
//! `stInternal` sparse carved into `stInternalSolid` near bridge candidates, so
//! bridge extrusions anchor onto solid instead of lightning. The expanded
//! surfaces feed the lightning `Generator` (built on the expanded set, like
//! `prepare_lightning_infill_data` after the expansion). Upstream runs a
//! two-phase collect-then-apply (ORCA de-race); ares runs serially so Phase 1
//! naturally reads original state.

use crate::{
    geometry::{
        ClipperError, CoordinateScale, ExPolygon, FillRule, JoinType, Polygon, difference_ex,
        intersection_polygons_paths, offset_paths, union_safety_offset_ex,
    },
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

const MITER_LIMIT: f64 = 3.0;

/// Phase-2 expansion (`PrintObject.cpp:2666-2707`) for one layer's
/// `fill_surfaces`: `expansion = intersection_ex(sparse, expand(new_polys,
/// scaled(3)))`; `solid_infill += expansion`; `sparse = diff_ex(sparse,
/// union_safety_offset_ex(solid))`. Non-(Internal|InternalSolid) surfaces pass
/// through; the rewritten `stInternalSolid`/`stInternal` pair is appended.
pub(super) fn expand_layer(
    fill_surfaces: &[RegionSurface],
    candidate_new_polygons: &[Polygon],
    scale: CoordinateScale,
) -> Result<Vec<RegionSurface>, ClipperError> {
    let mut passthrough: Vec<RegionSurface> = Vec::new();
    let mut sparse_infill: Vec<ExPolygon> = Vec::new();
    let mut solid_infill: Vec<ExPolygon> = Vec::new();
    for surface in fill_surfaces {
        match surface.as_parts().0 {
            RegionSurfaceKind::Internal => sparse_infill.push(surface.as_parts().1.clone()),
            RegionSurfaceKind::InternalSolid => solid_infill.push(surface.as_parts().1.clone()),
            _ => passthrough.push(surface.clone()),
        }
    }
    if sparse_infill.is_empty() {
        return Ok(fill_surfaces.to_vec());
    }
    // `expand(surface.new_polys, scaled<float>(3.0))`.
    let expand_delta = scale.scaled_delta(3.0) as f32;
    let expanded = offset_paths(
        candidate_new_polygons,
        expand_delta,
        JoinType::Miter,
        MITER_LIMIT,
    )?;
    let sparse_polys: Vec<Polygon> = sparse_infill
        .iter()
        .flat_map(|e| {
            let mut v = vec![e.contour().clone()];
            v.extend(e.holes().iter().cloned());
            v
        })
        .collect();
    let expansion = intersection_polygons_paths(&sparse_polys, &expanded)?;
    solid_infill.extend(expansion.into_iter().map(|p| ExPolygon::new(p, Vec::new())));

    let solid = union_safety_offset_ex(
        &solid_infill
            .iter()
            .flat_map(|e| {
                let mut v = vec![e.contour().clone()];
                v.extend(e.holes().iter().cloned());
                v
            })
            .collect::<Vec<_>>(),
    )?;
    let sparse = difference_ex(&sparse_infill, &solid)?;

    let mut out = passthrough;
    for ep in &solid {
        out.push(RegionSurface::new(
            RegionSurfaceKind::InternalSolid,
            ep.clone(),
        ));
    }
    for ep in &sparse {
        out.push(RegionSurface::internal(ep.clone()));
    }
    let _ = FillRule::EvenOdd; // silence unused import if FillRule unused
    Ok(out)
}
