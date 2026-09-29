//! `FillLockedZag::fill_surface_locked_zag`
//! (`src/libslic3r/Fill/FillRectilinear.cpp:3866-3934`): the locked-zag sparse
//! infill splits the surface into a skin ring (inset `skin_infill_depth`) and
//! an inner skeleton, fills each at its own density with the zig-zag /
//! cross-zag rectilinear fill, then re-intersects the generated lines with the
//! per-`Flow` regions (`skin_flow_params` / `skeleton_flow_params`) so each
//! emitted polyline carries the flow of the region it lands in.
//!
//! The `LockRegionParam` maps are built per layer in `group_fills`
//! (`Fill.cpp:998-1008`): every fill's `expolygon` is bucketed by its skin /
//! skeleton density and by its skin / skeleton `Flow`.

use crate::geometry::{
    ClipperError, CoordinateScale, ExPolygon, JoinType, Polygon, Polyline, difference_ex,
    intersection_ex, intersection_open_polylines, offset_expolygons,
    union_safety_offset_expolygons,
};

use super::rectilinear::MonotonicFillParams;

/// `LockDensityParam` mirrors the upstream `std::map<density, ExPolygons>`
/// entry — one density key per bucket of `ExPolygon`s.
pub(crate) struct LockDensityParam {
    pub(crate) density: f32,
    pub(crate) expolygons: Vec<ExPolygon>,
}

/// `LockFlowParam` mirrors the upstream `std::map<Flow, ExPolygons>` entry.
pub(crate) struct LockFlowParam<F> {
    pub(crate) flow: F,
    pub(crate) expolygons: Vec<ExPolygon>,
}

/// `LockRegionParam` (`FillRectilinear.hpp:215`).
pub(crate) struct LockRegionParam<F> {
    pub(crate) skin_density_params: Vec<LockDensityParam>,
    pub(crate) skeleton_density_params: Vec<LockDensityParam>,
    pub(crate) skin_flow_params: Vec<LockFlowParam<F>>,
    pub(crate) skeleton_flow_params: Vec<LockFlowParam<F>>,
}

/// `fill_surface_locked_zag` — returns `(polyline, flow)` pairs; `F` is the
/// caller's flow type, cloned into each emitted entity.
///
/// `fill_sub` is the rectilinear zig-zag/cross-zag generator
/// (`fill_rectilinear_surface`) — injected so the caller owns the per-pattern
/// `MonotonicFillParams` (angle/alternation) while this supplies the
/// density + `horiz_move` overrides (`zig_params.horiz_move = 0` for the
/// skeleton, `cross_params` keeps `params.horiz_move` for the skin).
pub(crate) fn fill_surface_locked_zag<F, G>(
    expolygon: &ExPolygon,
    params: &MonotonicFillParams,
    lock: &LockRegionParam<F>,
    scale: CoordinateScale,
    skin_infill_depth: i64,
    infill_lock_depth: i64,
    mut fill_sub: G,
) -> Result<Vec<(Polyline, F)>, ClipperError>
where
    F: Clone,
    G: FnMut(&ExPolygon, f32, f32, bool) -> Result<Vec<Polyline>, ClipperError>,
{
    // `zig_expas` = inner skeleton region (expolygon shrunk by skin depth);
    // `cross_expas` = the skin ring between the two (`diff_ex`).
    let zig_expas = offset_expolygons(
        std::slice::from_ref(expolygon),
        -skin_infill_depth as f32,
        JoinType::Miter,
        3.0,
    )?;
    let cross_expas = difference_ex(std::slice::from_ref(expolygon), &zig_expas)?;

    let mut skeleton_lines: Vec<Polyline> = Vec::new();
    let mut skin_lines: Vec<Polyline> = Vec::new();

    // Skeleton: each density bucket ∩ zig region, expanded by the lock depth
    // and re-clipped to the surface, filled at that density, horiz_move=0.
    for density_param in &lock.skeleton_density_params {
        let region_exp = union_safety_offset_expolygons(&density_param.expolygons)?;
        let mut exps = intersection_ex(&region_exp, &zig_expas)?;
        exps = offset_expolygons(&exps, infill_lock_depth as f32, JoinType::Miter, 3.0)?;
        let exps = intersection_ex(&exps, std::slice::from_ref(expolygon))?;
        for exp in exps {
            skeleton_lines.extend(fill_sub(&exp, density_param.density, 0.0, true)?);
        }
    }

    // Skin: each density bucket ∩ cross ring, filled at that density with the
    // caller's `horiz_move` (the cross-zag shift).
    for density_param in &lock.skin_density_params {
        let region_exp = union_safety_offset_expolygons(&density_param.expolygons)?;
        let exps = intersection_ex(&region_exp, &cross_expas)?;
        for exp in exps {
            skin_lines.extend(fill_sub(
                &exp,
                density_param.density,
                params.horiz_move,
                false,
            )?);
        }
    }

    // `generate_for_different_flow` (FillRectilinear.cpp:3887-3897): intersect
    // the collected lines with each flow's region; each surviving polyline
    // takes that region's flow.
    let mut out: Vec<(Polyline, F)> = Vec::new();
    for flow_param in &lock.skeleton_flow_params {
        let region = union_safety_offset_expolygons(&flow_param.expolygons)?;
        let region_polygons = expolygons_to_polygons(&region);
        for polyline in intersection_open_polylines(&skeleton_lines, &region_polygons)? {
            out.push((polyline, flow_param.flow.clone()));
        }
    }
    for flow_param in &lock.skin_flow_params {
        let region = union_safety_offset_expolygons(&flow_param.expolygons)?;
        let region_polygons = expolygons_to_polygons(&region);
        for polyline in intersection_open_polylines(&skin_lines, &region_polygons)? {
            out.push((polyline, flow_param.flow.clone()));
        }
    }
    let _ = scale;
    Ok(out)
}

fn expolygons_to_polygons(expolygons: &[ExPolygon]) -> Vec<Polygon> {
    expolygons
        .iter()
        .flat_map(|e| std::iter::once(e.contour().clone()).chain(e.holes().iter().cloned()))
        .collect()
}
