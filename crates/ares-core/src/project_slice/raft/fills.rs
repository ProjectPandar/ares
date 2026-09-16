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

use crate::fill::base_support::connect_base_support;
use crate::fill::checked_rotate::rotate_point;
use crate::fill::rectilinear::segments::{
    IntersectionKind, SegmentIntersection, SegmentedLine, populate_vertical_lines,
    prepare_rectilinear_contours,
};
use crate::geometry::{BoundingBox, ClipperError, CoordinateScale, Point, Polygon, Polyline};
use crate::geometry::{FillRule, union_ex};

#[derive(Clone, Copy, Debug)]
pub(crate) struct RaftFillSpec {
    /// Infill angle in radians.
    pub(crate) angle: f64,
    /// Flow spacing in mm.
    pub(crate) spacing: f64,
    /// Density in [0, 1].
    pub(crate) density: f32,
}

/// Fill one raft layer (`FillSupportBase::fill_surface`).
#[expect(dead_code, reason = "wired by the raft emission slice")]
pub(crate) fn raft_layer_fill(
    layer_polygons: &[Polygon],
    spec: RaftFillSpec,
    reference: Point,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    let expolygons = union_ex(layer_polygons, FillRule::NonZero)?;
    let mut out = Vec::new();
    for expolygon in expolygons {
        let inner_offset = checked_scale_f32(scale, -0.5 * spec.spacing)?;
        let line_spacing = scale
            .checked_scale(spec.spacing / f64::from(spec.density))
            .ok_or(ClipperError::CoordinateOutOfRange)?;
        if line_spacing <= 0 {
            continue;
        }
        let mut slice = prepare_rectilinear_contours(&expolygon, -(spec.angle + std::f64::consts::FRAC_PI_2), 0.0, inner_offset)?;
        let boundary: Vec<Polygon> = slice
            .contours
            .iter()
            .filter(|contour| !contour.inner)
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
        let reference = rotate_point(reference, (-spec.angle).cos(), (-spec.angle).sin())?;
        let aligned_min_x = align_to_grid(source_bounds.min().x(), line_spacing, reference.x())?;
        let width = source_bounds.max().x() - aligned_min_x;
        let count = usize::try_from((width + line_spacing - 1) / line_spacing).unwrap_or(0);
        populate_vertical_lines(&mut slice, count, aligned_min_x, line_spacing)?;

        let mut fill_lines = Vec::new();
        for line in &slice.lines {
            if line.x < source_bounds.min().x() {
                continue;
            }
            if line.x > source_bounds.max().x() {
                break;
            }
            fill_lines.extend(vertical_segments(line));
        }
        if fill_lines.is_empty() {
            continue;
        }

        let bbox =
            BoundingBox::from_polygons(&boundary).ok_or(ClipperError::CoordinateOutOfRange)?;
        out.extend(connect_base_support(
            fill_lines,
            &boundary,
            bbox,
            spec.spacing,
            spec.density,
            scale,
        )?);
    }
    Ok(out)
}

/// Emit OuterLow→OuterHigh pairs as 2-point polylines
/// (`make_fill_lines:2948-2960`); the slice frame already carries the
/// back-rotated points.
fn vertical_segments(line: &SegmentedLine) -> Vec<Polyline> {
    let mut out = Vec::new();
    let mut index = 0;
    while index < line.intersections.len() {
        let low = &line.intersections[index];
        if low.kind != IntersectionKind::OuterLow {
            index += 1;
            continue;
        }
        let Some(high) = line.intersections.get(index + 1) else {
            break;
        };
        if high.kind == IntersectionKind::OuterHigh {
            out.push(Polyline::new(vec![low.point, high.point]));
            index += 2;
        } else {
            index += 1;
        }
    }
    out
}

fn align_to_grid(coordinate: i64, spacing: i64, base: i64) -> Result<i64, ClipperError> {
    let spacing = i128::from(spacing);
    let delta = i128::from(coordinate) - i128::from(base);
    i64::try_from(i128::from(base) + delta.div_euclid(spacing) * spacing)
        .map_err(|_| ClipperError::CoordinateOutOfRange)
}

fn checked_scale_f32(scale: CoordinateScale, value: f64) -> Result<f32, ClipperError> {
    let scaled = scale
        .checked_scale(value)
        .ok_or(ClipperError::CoordinateOutOfRange)?;
    Ok(scaled as f32)
}
