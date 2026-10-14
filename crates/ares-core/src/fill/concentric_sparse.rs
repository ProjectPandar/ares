//! The sub-100%-density branch of `FillConcentric::_fill_surface_single`
//! (`src/libslic3r/Fill/FillConcentric.cpp:9-64`): plain `offset2_ex` shrinking
//! rings ordered outside-in, each split at the point nearest the previous
//! ring's end, then `multiline_fill`, `clip_end(loop_clipping)`, and a
//! shortest-traverse reorder.
//!
//! Shared by the emitted sparse concentric fill and the bridge-over-infill
//! anchor projection — upstream `Fill::fill_surface` runs this same routine in
//! both contexts (the emitted fill goes through `fill_surface`, the anchor
//! projection through `Fill::new_from_type` + `fill_surface`).

use crate::{
    fill::line::adjust_solid_spacing,
    geometry::{
        ClipperError, CoordinateScale, ExPolygon, JoinType, Point, Polygon, Polyline,
        offset_expolygons, offset2_ex, union_contours,
    },
};

/// `Point::nearest_point_index` (`Point.cpp:199-220`): returns the LAST index
/// achieving the minimal squared distance, with an X-then-Y early-exit prune.
/// Distances are computed in `double` (i64 squared would overflow).
pub(crate) fn nearest_point_index(last_pos: Point, points: &[Point]) -> usize {
    let mut idx = 0usize;
    let mut best = -1.0f64;
    for (i, p) in points.iter().enumerate() {
        let dx = (last_pos.x() - p.x()) as f64;
        let mut d = dx * dx;
        if best != -1.0 && d > best {
            continue;
        }
        let dy = (last_pos.y() - p.y()) as f64;
        d += dy * dy;
        if best != -1.0 && d > best {
            continue;
        }
        idx = i;
        best = d;
        if d < f64::EPSILON {
            break;
        }
    }
    idx
}

/// `FillConcentric::_fill_surface_single` — `density` is a 0..1 fraction,
/// `loop_clipping` is `scale_(seam_gap.get_abs_value(nozzle))`.
pub(crate) fn fill_concentric_sparse(
    expolygon: &ExPolygon,
    spacing: f64,
    params: &super::FillParams,
    loop_clipping: i64,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    let density = params.density;
    let multiline = params.multiline;
    let dont_adjust = params.dont_adjust;
    // All of `min_spacing`, `distance`, `contract` are already in scaled coord
    // units (spacing is divided by scale.factor() once here).
    let mut effective_spacing = spacing;
    let scaled_spacing = spacing / scale.factor();
    let min_spacing = scaled_spacing * f64::from(multiline);
    // `coord_t(min_spacing / params.density)` — upstream truncates the
    // ring pitch to integer scaled units (`FillConcentric.cpp:23`), so
    // the i64 cast (toward zero) is load-bearing: sub-micron remainder
    // shifts every subsequent `offset2_ex` ring by accumulating drift.
    let mut distance = (min_spacing / f64::from(density)).trunc();
    // `FillConcentric.cpp:25-27`: ironing retains nominal spacing;
    // ordinary full-density fills may adjust it to the contour's X extent.
    if density > 0.9999 && !dont_adjust {
        let (x0, x1) = {
            let (mut x0, mut x1) = (i64::MAX, i64::MIN);
            for point in expolygon.contour().points() {
                x0 = x0.min(point.x());
                x1 = x1.max(point.x());
            }
            (x0, x1)
        };
        distance = adjust_solid_spacing(x1 - x0, distance as i64) as f64;
        // FillConcentric.cpp:25-27: only solid adjustment rewrites mm spacing.
        effective_spacing = distance * scale.factor();
    }
    // FillConcentric.cpp:31: contract using nominal/adjusted mm spacing,
    // not the scaled, density-dependent ring pitch. Narrow after scaling.
    let contract = 0.5 * f64::from(multiline - 1) * effective_spacing / scale.factor();
    let mut last = if contract.abs() > f64::EPSILON {
        offset_expolygons(
            std::slice::from_ref(expolygon),
            -contract as f32,
            JoinType::Miter,
            3.0,
        )?
    } else {
        vec![expolygon.clone()]
    };
    let mut loops: Vec<Polygon> = last
        .iter()
        .flat_map(|e| {
            let (c, h) = (e.contour().clone(), e.holes().to_vec());
            std::iter::once(c).chain(h)
        })
        .collect();
    while !last.is_empty() {
        last = offset2_ex(
            &last,
            -(distance + min_spacing / 2.0) as f32,
            (min_spacing / 2.0) as f32,
            JoinType::Miter,
            3.0,
        )?;
        loops.extend(last.iter().flat_map(|e| {
            let (c, h) = (e.contour().clone(), e.holes().to_vec());
            std::iter::once(c).chain(h)
        }));
    }
    // outside-in ordering via the union contour walk
    // (`union_pt_chained_outside_in`).
    let contours = union_contours(&loops)?;
    // split each ring at the point nearest the previous ring's end
    // (`loop.split_at_index(last_pos.nearest_point_index(loop.points))`),
    // then `last_pos = polyline.last_point()` (the re-appended split point).
    let mut polylines: Vec<Polyline> = Vec::with_capacity(contours.len());
    let mut last_pos = Point::new(0, 0);
    for points in contours {
        if points.len() < 2 {
            continue;
        }
        let nearest = nearest_point_index(last_pos, &points);
        let mut ring = points[nearest..].to_vec();
        ring.extend_from_slice(&points[..nearest]);
        ring.push(points[nearest]);
        polylines.push(Polyline::new(ring));
        last_pos = polylines
            .last()
            .and_then(|p| p.back())
            .unwrap_or(points[nearest]);
    }
    // `multiline_fill` — expand by the multiline overlap count.
    let mut polylines = if multiline > 1 {
        // FillConcentric.cpp:54 / FillBase.cpp:2712: multiline_fill takes
        // mm spacing; its offset implementation performs the scaling.
        crate::fill::multiline_offset::apply(polylines, multiline, effective_spacing, scale)?
    } else {
        polylines
    };
    // `clip_end(loop_clipping)` then keep only valid paths (>= 2 points).
    polylines.retain_mut(|polyline| {
        polyline.clip_end(loop_clipping as f64);
        polyline.is_valid()
    });
    Ok(polylines)
}
