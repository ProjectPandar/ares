//! Raft layer fills, ported from `FillSupportBase::fill_surface`
//! (`OrcaSlicer/src/libslic3r/Fill/FillRectilinear.cpp:3610-3634`) and
//! `make_fill_lines` (`:2920-2961`), wired to the ported
//! `connect_base_support`.
//!
//! Grid anchor: the object bounding-box center rotated by `-angle`
//! (`Fill::_infill_direction`, `FillBase.cpp:288-290`; the filler's
//! bbox is the object bbox, `SupportCommon.cpp:1454-1455`). Only
//! OuterLow→OuterHigh intersection pairs become vertical fill lines;
//! the connection boundary is the `-0.5·spacing` inset outer contour.

use super::fill_params::RaftFillPattern;
use crate::fill::base_support::connect_base_support;
use crate::fill::checked_rotate::rotate_point;
use crate::fill::rectilinear::segments::{populate_vertical_lines, prepare_rectilinear_contours};
use crate::geometry::{
    BoundingBox, ClipperError, CoordinateScale, ExPolygon, Point, Polygon, Polyline,
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct RaftFillSpec {
    /// Infill angle in radians.
    pub(crate) angle: f64,
    /// Flow spacing in mm.
    pub(crate) spacing: f64,
    /// `flow.spacing()` of the layer's own extrusion flow, mm: the sheath
    /// offsets use it (`SupportCommon.cpp:733`), while the filler keeps
    /// `spacing`. Raft base layers are taller than the object layer the
    /// filler spacing is derived from (`:1476` vs `:1479`).
    pub(crate) sheath_spacing: f64,
    /// Density in [0, 1].
    pub(crate) density: f32,
    /// `with_sheath` (`SupportCommon.cpp:1516`): the flange (raft layer 0)
    /// draws a perimeter loop before the fill.
    pub(crate) sheath: bool,
    /// `first_layer_flow.width()` for the flange `closing_ex` inflate
    /// (`0.5·width`), mm.
    pub(crate) width_mm: f64,
    /// `Fill::new_from_type(base_fill_pattern / raft_interface_fill_pattern)`
    /// (`SupportCommon.cpp:1450-1451`).
    pub(crate) pattern: RaftFillPattern,
    /// Region preprocessing before the fill
    /// (`fill_expolygons_with_sheath_generate_paths` vs
    /// `extrude_interface`, `SupportCommon.cpp:725-727,1729`).
    pub(crate) preprocess: RaftFillPreprocess,
}

/// `SupportCommon.cpp:725-727` applies `closing_ex(polygons, ε)` to the
/// non-sheath raft base/interface layers, while the contact layer
/// (`extrude_interface`, `:1729`) wraps its polygons in
/// `union_safety_offset_ex` (+10 units, `ClipperUtils.hpp:17`).
#[derive(Clone, Copy, Debug)]
pub(crate) enum RaftFillPreprocess {
    /// `closing_ex(polygons, float(SCALED_EPSILON))`.
    Closing,
    /// `union_safety_offset_ex` (Clipper +10 lattice units).
    SafetyOffset,
}

/// The emitted raft-layer polylines (sheath + fill) in source order
/// (`fill_expolygons_with_sheath_generate_paths`,
/// `SupportCommon.cpp:705-757`). With a sheath, each region of
/// `closing_ex(polygons, ε, ε + 0.5·width)` emits its contour and hole
/// loops closed and clipped by `0.15·spacing`, then the fill on
/// `offset_ex(-0.4·spacing)` of that region; other layers just take the
/// fill. `Print.cpp:2693` collects these points for the skirt hull.
pub(crate) fn raft_layer_polylines(
    layer_polygons: &[Polygon],
    spec: RaftFillSpec,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    if !spec.sheath {
        return raft_layer_fill(layer_polygons, spec, Point::new(0, 0), scale);
    }
    debug_assert!(matches!(spec.preprocess, RaftFillPreprocess::Closing));
    let units = 1.0 / scale.factor();
    // `closing_ex(polygons, SCALED_EPSILON, SCALED_EPSILON + 0.5·width)`
    // (`SupportCommon.cpp:728`): dilate by `ε` then erode by
    // `ε + 0.5·width` — net shrink of `0.5·width`, so the sheath sits
    // just inside the polygon boundary. `SCALED_EPSILON = scale_(1e-4)`
    // and `float(SCALED_EPSILON)` keeps the fractional 0.4096 units
    // (`libslic3r.h:52,96`).
    let epsilon = scale.scaled_delta(1e-4) as f32;
    // `SCALED_EPSILON + 0.5*flow.scaled_width()`
    // (`SupportCommon.cpp:738`, `Flow.hpp:62`): `scaled_width` is the
    // integer `coord_t(scale_(width))` — `Flow` stores the width as
    // `float` (`Flow.cpp:129-143`), so narrow through f32 before the
    // scale/trunc, then halve the integer.
    let erode = (0.5 * ((f64::from(spec.width_mm as f32) * units) as i64) as f64) as f32;
    // `spacing = flow.scaled_spacing()` (`:733`): `Flow` spacing is
    // stored as `float`, so narrow through f32 before the trunc.
    let spacing = (f64::from(spec.sheath_spacing as f32) * units) as i64 as f64;
    let mut out = Vec::new();
    for expolygon in closing_polygons(layer_polygons, epsilon, epsilon + erode)? {
        // `draw_perimeters(expoly, spacing * 0.15)` (`:459-475,735`): the
        // contour, then each hole reversed, each closed and clipped.
        let loops = std::iter::once((expolygon.contour(), false))
            .chain(expolygon.holes().iter().map(|hole| (hole, true)));
        for (polygon, hole) in loops {
            let mut points = polygon.points().to_vec();
            points.push(points[0]);
            if hole {
                points.reverse();
            }
            let mut polyline = Polyline::new(points);
            clip_polyline_end(&mut polyline, 0.15 * spacing);
            if polyline.points().len() > 1 {
                out.push(polyline);
            }
        }
        // `offset_ex(expoly, float(-0.4 * spacing))` (`:749`).
        let fill_region = crate::geometry::offset_expolygon(
            &expolygon,
            (-0.4 * spacing) as f32,
            crate::geometry::JoinType::Miter,
            3.0,
        )?;
        out.extend(fill_expolygons(fill_region, spec, Point::new(0, 0), scale)?);
    }
    Ok(out)
}

/// `closing_ex(const Polygons&, delta1, delta2)` (`ClipperUtils.cpp:599-604`):
/// `shrink_paths<PolyTree>(expand_paths(polygons, delta1), delta2)`, which
/// keeps clockwise hole polygons as holes.
fn closing_polygons(
    polygons: &[Polygon],
    expand: f32,
    shrink: f32,
) -> Result<Vec<ExPolygon>, ClipperError> {
    use crate::geometry::{JoinType, offset_paths, offset_paths_tree};
    let expanded = offset_paths(polygons, expand, JoinType::Miter, 3.0)?;
    Ok(offset_paths_tree(&expanded, -shrink, JoinType::Miter, 3.0)?.into_expolygons())
}

/// `Polyline::clip_end` (`Polyline.cpp`): shorten the polyline tail by
/// `distance`, dropping exhausted segments.
mod geom;
use geom::{align_to_grid, checked_scale_f32, clip_polyline_end, vertical_segments};

mod patterns;

/// Fill one raft layer with the filler `Fill::new_from_type(pattern)` builds
/// (`SupportCommon.cpp:1450-1451`); the inline body is
/// `FillSupportBase::fill_surface`.
pub(crate) fn raft_layer_fill(
    layer_polygons: &[Polygon],
    spec: RaftFillSpec,
    reference: Point,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    let expolygons = match spec.preprocess {
        // `union_safety_offset_ex` = `offset_ex(polygons, 10.f)`
        // (`ClipperUtils.hpp:17,364`).
        RaftFillPreprocess::SafetyOffset => {
            crate::geometry::union_safety_offset_ex(layer_polygons)?
        }
        // `closing_ex(polygons, float(SCALED_EPSILON))`
        // (`SupportCommon.cpp:727`): `SCALED_EPSILON = scale_(1e-4)`.
        RaftFillPreprocess::Closing => {
            let epsilon = scale.scaled_delta(1e-4) as f32;
            closing_polygons(layer_polygons, epsilon, epsilon)?
        }
    };
    fill_expolygons(expolygons, spec, reference, scale)
}

/// `fill_expolygons_generate_paths` (`SupportCommon.cpp:432-443`); the
/// `SupportBase` body is `FillSupportBase::fill_surface`.
fn fill_expolygons(
    expolygons: Vec<ExPolygon>,
    spec: RaftFillSpec,
    reference: Point,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    let mut out = Vec::new();
    for expolygon in expolygons {
        match spec.pattern {
            RaftFillPattern::SupportBase => {}
            RaftFillPattern::Rectilinear => {
                out.extend(patterns::rectilinear(&expolygon, spec, reference, scale)?);
                continue;
            }
            RaftFillPattern::Honeycomb => {
                out.extend(patterns::honeycomb(&expolygon, spec, scale)?);
                continue;
            }
        }
        // `aoffset1 = coord_t(scale_(overlap − 0.5·spacing))`
        // (`FillRectilinear.cpp:3613`, `FillBase.cpp:394`) truncates to an
        // integer lattice unit, matching `checked_scale`.
        let inner_offset = checked_scale_f32(scale, -0.5 * spec.spacing)?;
        // `coord_t(scale_(this->spacing) / params.density)`
        // (`FillRectilinear.cpp:3619`): the spacing narrows to f32
        // (`Flow` stores it as `float`) and truncates to `coord_t`
        // BEFORE the density division; the quotient truncates again.
        // `coord_t(scale_(this->spacing) / params.density)`
        // (`FillRectilinear.cpp:3619`): `this->spacing` is `float`
        // (`Flow::spacing()` field is float, `Flow.hpp:62`), scaled to
        // double lattice units, divided by density, then truncated.
        let scaled_spacing = f64::from(spec.spacing as f32) / scale.factor();
        let line_spacing = (scaled_spacing / f64::from(spec.density)) as i64;
        if line_spacing <= 0 {
            continue;
        }
        // Upstream `ExPolygonWithOffset(expoly, -rotate_vector.first, ...)`
        // rotates by the f32 `float` angle; the extra FRAC_PI_2 below is
        // ares' vertical-line convention translation. Round the combined
        // argument through f32 so the trig matches upstream's `float`
        // precision bit-for-bit.
        let rotate_by = f64::from(-((spec.angle as f32) + std::f32::consts::FRAC_PI_2));
        let mut slice = prepare_rectilinear_contours(&expolygon, rotate_by, 0.0, inner_offset)?;
        // Upstream `connect_base_support(..., poly_with_offset.polygons_outer,
        // ...)` — `polygons_outer` there is the DELTA-shrunk fill boundary
        // (where the infill endpoints sit), which is ares' `inner`
        // contour (`ExPolygonWithOffset` naming is inverted vs ares').
        let boundary: Vec<Polygon> = slice
            .contours
            .iter()
            .filter(|contour| contour.inner)
            .map(|contour| contour.polygon.clone())
            .collect();
        if boundary.is_empty() {
            continue;
        }

        // Grid anchor (`make_fill_lines:2928-2934`): align the source
        // bbox minimum to the grid passing through the rotated
        // reference point.
        let source_bounds =
            BoundingBox::from_expolygon(&slice.source).ok_or(ClipperError::CoordinateOutOfRange)?;
        // `-rotate_vector.first` in upstream is the negated f32 angle.
        let neg_angle = f64::from(-(spec.angle as f32));
        let reference = rotate_point(reference, neg_angle.cos(), neg_angle.sin())?;
        let aligned_min_x = align_to_grid(source_bounds.min().x(), line_spacing, reference.x())?;
        let width = source_bounds.max().x() - aligned_min_x;
        let count = usize::try_from((width + line_spacing - 1) / line_spacing).unwrap_or(0);
        populate_vertical_lines(&mut slice, count, aligned_min_x, line_spacing)?;

        // `FillSupportBase::fill_surface` passes `x_margin = 0`
        // (`FillRectilinear.cpp:3621`): keep every line inside the source
        // bbox (`make_fill_lines:2924-2925, 2937-2939`).
        let (x_min, x_max) = (source_bounds.min().x(), source_bounds.max().x());
        let mut fill_lines = Vec::new();
        for line in &slice.lines {
            if line.x < x_min {
                continue;
            }
            if line.x > x_max {
                break;
            }
            fill_lines.extend(vertical_segments(line));
        }
        if fill_lines.is_empty() {
            continue;
        }

        let bbox =
            BoundingBox::from_polygons(&boundary).ok_or(ClipperError::CoordinateOutOfRange)?;
        // Connect in the SLICE frame (upstream connects before the
        // rotate-back), then rotate the connected output back by
        // +(angle+π/2) (`FillSupportBase::fill_surface:3627-3633`).
        // `pt.rotate(cos_a, sin_a)` with `cos_a = cos(rotate_vector.first)`
        // — f64 cos of the f32 angle (`FillRectilinear.cpp:3627-3633`).
        let rotate_back = f64::from((spec.angle as f32) + std::f32::consts::FRAC_PI_2);
        let (cos_a, sin_a) = (rotate_back.cos(), rotate_back.sin());
        let connected = connect_base_support(
            fill_lines,
            &boundary,
            bbox,
            spec.spacing,
            spec.density,
            scale,
        )?;
        out.extend(connected.into_iter().map(|polyline| {
            let rotated = polyline
                .points()
                .iter()
                .map(|point| {
                    let (x, y) = (point.x() as f64, point.y() as f64);
                    crate::geometry::Point::new(
                        (x * cos_a - y * sin_a).round() as i64,
                        (x * sin_a + y * cos_a).round() as i64,
                    )
                })
                .collect::<Vec<_>>();
            crate::geometry::Polyline::new(rotated)
        }));
    }
    Ok(out)
}

#[cfg(test)]
mod tests;
