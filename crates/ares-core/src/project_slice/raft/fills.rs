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
    /// `no_sort` (`:1517`): each sheath region is one ordered collection.
    pub(crate) no_sort: bool,
    /// `tree_branch_diameter_double_wall_area_scaled`
    /// (`SupportParameters.hpp:186-188`), lattice units².
    pub(crate) tree_double_wall_area: f64,
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

/// One entry of a raft layer's `support_fills.entities`; `extrude_support`
/// chains the top-level entries (`GCode.cpp:6199-6201`).
#[derive(Debug, PartialEq)]
pub(crate) enum RaftEntity {
    /// `ExtrusionPath` (`extrusion_entities_append_paths`,
    /// `ExtrusionEntity.hpp:581-590`).
    Path(Polyline),
    /// `ExtrusionPathOriented`: appended with `can_reverse = false`.
    Oriented(Polyline),
    /// `ExtrusionEntityCollection` with `no_sort = true`, printed in order.
    NoSort(Vec<Polyline>),
}

impl RaftEntity {
    /// The entity's polylines in print order.
    pub(crate) fn polylines(&self) -> &[Polyline] {
        match self {
            Self::Path(polyline) | Self::Oriented(polyline) => std::slice::from_ref(polyline),
            Self::NoSort(polylines) => polylines,
        }
    }
}

/// The raft layer's `support_fills.entities` (`SupportCommon.cpp:1476-1520`):
/// `fill_expolygons_with_sheath_generate_paths` (`:705-757`) with a sheath,
/// otherwise the plain fill. `Print.cpp:2693` collects their points for
/// the skirt hull.
pub(crate) fn raft_layer_entities(
    layer_polygons: &[Polygon],
    spec: RaftFillSpec,
    scale: CoordinateScale,
) -> Result<Vec<RaftEntity>, ClipperError> {
    if !spec.sheath {
        return Ok(
            raft_layer_fill(layer_polygons, spec, Point::new(0, 0), scale)?
                .into_iter()
                .map(RaftEntity::Path)
                .collect(),
        );
    }
    debug_assert!(matches!(spec.preprocess, RaftFillPreprocess::Closing));
    if spec.density == 0.0 {
        return tree::tree_supports_generate_paths(layer_polygons, spec, scale);
    }
    sheath::fill_with_sheath(layer_polygons, spec, scale)
}

mod sheath;
mod tree;

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
use geom::{align_to_grid, checked_scale_f32, vertical_segments};

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
