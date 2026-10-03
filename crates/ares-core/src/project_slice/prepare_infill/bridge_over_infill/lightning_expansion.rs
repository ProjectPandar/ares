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
        ClipperError, CoordinateScale, ExPolygon, JoinType, Polygon, difference_ex,
        intersection_ex_polygons, offset_paths, union_safety_offset_ex,
    },
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

const MITER_LIMIT: f64 = 3.0;

/// Phase-2 expansion (`PrintObject.cpp:2666-2707`) for one region's
/// `fill_surfaces`: `expansion = intersection_ex(sparse, expand(new_polys,
/// scaled(3)))`; `solid_infill += expansion`; `sparse = diff_ex(sparse,
/// union_safety_offset_ex(solid))`. Non-(Internal|InternalSolid) surfaces pass
/// through; the rewritten `stInternalSolid`/`stInternal` pair is appended.
pub(super) fn expand_region<'a>(
    fill_surfaces: &[RegionSurface],
    candidate_new_polygons: impl IntoIterator<Item = &'a [Polygon]>,
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
    // `expand(surface.new_polys, scaled<float>(3.0))`.
    let expand_delta = scale.scaled_delta(3.0) as f32;
    for polygons in candidate_new_polygons {
        let expanded = offset_paths(polygons, expand_delta, JoinType::Miter, MITER_LIMIT)?;
        solid_infill.extend(intersection_ex_polygons(&sparse_infill, &expanded)?);
    }

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
    for ep in solid {
        out.push(RegionSurface::new(RegionSurfaceKind::InternalSolid, ep));
    }
    for ep in sparse {
        out.push(RegionSurface::internal(ep));
    }
    Ok(out)
}
