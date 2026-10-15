//! `FillLine` port (`src/libslic3r/Fill/FillLine.cpp`): oscillating vertical
//! line infill. Each scan column produces a `Line` slanted by `_line_oscillation`
//! alternating sign; the lines are clipped to the surface, extended past the
//! boundary, chained, connected where the gap allows, and rotated back.

use crate::geometry::{
    BoundingBox, ClipperError, CoordinateScale, ExPolygon, JoinType, Line, Point, Polygon,
    Polyline, difference_open_polylines, intersection_open_polylines, offset_expolygon,
};

#[derive(Clone, Copy)]
pub(crate) struct LineFillParams {
    /// Regional runtime flag from Fill.cpp:1235-1242 / FillBase.cpp:311-313.
    pub(crate) dont_alternate_fill_direction: bool,
    /// Line spacing in mm (unscaled).
    pub(crate) spacing: f64,
    pub(crate) overlap: f64,
    /// Infill density fraction (0..1).
    pub(crate) density: f32,
    /// Infill direction angle (radians) BEFORE the `Fill::_infill_direction`
    /// adjustments (`out_angle += pi/2`, layer alternation).
    pub(crate) angle: f32,
    /// `layer_id` — drives the `+_layer_angle` alternation.
    pub(crate) layer_index: usize,
    /// `thickness_layers` — divisor for the alternation index.
    pub(crate) thickness_layers: u16,
    /// `fixed_angle` — skips the layer alternation when true.
    pub(crate) fixed_angle: bool,
    /// Bridge angle override (`surface->bridge_angle >= 0`).
    pub(crate) bridge_angle: Option<f32>,
    /// Reference point (object bbox center) — `direction.second`.
    pub(crate) reference: Point,
    /// `dont_adjust` — skip the solid-spacing adjustment.
    pub(crate) dont_adjust: bool,
}

#[cfg(test)]
mod tests;

const EPSILON: f64 = 1e-4;

fn rotate_polygon_origin(polygon: &Polygon, angle: f64) -> Result<Polygon, ClipperError> {
    Ok(Polygon::new(crate::fill::checked_rotate::rotate_points(
        polygon.points().to_vec(),
        angle,
    )?))
}

fn rotate_expolygon_origin(expolygon: &ExPolygon, angle: f64) -> Result<ExPolygon, ClipperError> {
    let (contour, holes) = (
        rotate_polygon_origin(expolygon.contour(), angle)?,
        expolygon
            .holes()
            .iter()
            .map(|h| rotate_polygon_origin(h, angle))
            .collect::<Result<Vec<_>, _>>()?,
    );
    Ok(ExPolygon::new(contour, holes))
}

fn polygon_bbox(polygon: &Polygon) -> BoundingBox {
    let mut it = polygon.points().iter();
    let first = *it.next().unwrap_or(&Point::new(0, 0));
    it.fold(BoundingBox::new(first, first), |bb, p| {
        BoundingBox::new(
            Point::new(bb.min().x().min(p.x()), bb.min().y().min(p.y())),
            Point::new(bb.max().x().max(p.x()), bb.max().y().max(p.y())),
        )
    })
}

fn align_scalar(coord: i64, spacing: i64, base: i64) -> i64 {
    let rel = coord - base;
    let aligned = if rel < 0 {
        ((rel - spacing + 1) / spacing) * spacing
    } else {
        (rel / spacing) * spacing
    };
    base + aligned
}

/// `Fill::fill_surface` (`FillBase.cpp:105-118`), followed by FillLine.
pub(crate) fn fill_line(
    expolygon: &ExPolygon,
    params: LineFillParams,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    let delta = ((params.overlap - 0.5 * params.spacing) / scale.factor()) as f32;
    let mut output = Vec::new();
    for domain in offset_expolygon(expolygon, delta, JoinType::Miter, 3.0)? {
        output.extend(fill_surface_single(&domain, params, scale)?);
    }
    Ok(output)
}

/// `FillLine::_fill_surface_single` (`FillLine.cpp:10-115`).
fn fill_surface_single(
    expolygon: &ExPolygon,
    params: LineFillParams,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    // `Fill::_infill_direction` (`FillBase.cpp:275-323`): `out_angle =
    // bridge_angle.unwrap_or(angle)`, `+ _layer_angle` alternation when
    // `!fixed_angle && !dont_alternate_fill_direction`, then `+ pi/2`.
    let mut angle = params.bridge_angle.unwrap_or(params.angle);
    if params.bridge_angle.is_none()
        && !params.fixed_angle
        && !params.dont_alternate_fill_direction
        && (params.layer_index / usize::from(params.thickness_layers)) & 1 == 1
    {
        angle += std::f32::consts::FRAC_PI_2;
    }
    let direction = f64::from(angle + std::f32::consts::FRAC_PI_2);
    // `expolygon.rotate(-direction.first)` — rotate about origin so columns are vertical.
    let rotated = rotate_expolygon_origin(expolygon, -direction)?;
    let reference =
        crate::fill::checked_rotate::rotate_points(vec![params.reference], -direction)?[0];

    let min_spacing = (params.spacing / scale.factor()) as i64;
    let mut line_spacing = (min_spacing as f64 / f64::from(params.density)) as i64;
    let diagonal_distance = line_spacing * 2;
    let line_oscillation = line_spacing - min_spacing;
    let mut bounding_box = polygon_bbox(&Polygon::new(rotated.contour().points().to_vec()));

    if f64::from(params.density) > 0.9999 && !params.dont_adjust {
        // `_adjust_solid_spacing(bounding_box.size()(0), line_spacing)` — snap the
        // line spacing so a whole number of lines spans the bounding-box width.
        line_spacing = adjust_solid_spacing(
            bounding_box.max().x() - bounding_box.min().x(),
            line_spacing,
        );
    } else {
        // extend bounding box to align the pattern with other layers.
        let aligned_min = Point::new(
            align_scalar(bounding_box.min().x(), line_spacing, reference.x()),
            align_scalar(bounding_box.min().y(), line_spacing, reference.y()),
        );
        bounding_box = BoundingBox::new(
            Point::new(
                bounding_box.min().x().min(aligned_min.x()),
                bounding_box.min().y().min(aligned_min.y()),
            ),
            bounding_box.max(),
        );
    }

    // Generate the vertical oscillating lines (`FillLine::_line`, FillLine.hpp:34).
    let mut lines: Vec<Line> = Vec::new();
    let x_max = bounding_box.max().x() + (EPSILON / scale.factor()) as i64;
    let mut x = bounding_box.min().x();
    while x <= x_max {
        let i = lines.len();
        let osc = if i & 1 != 0 { line_oscillation } else { 0 };
        lines.push(Line::new(
            Point::new(x - osc, bounding_box.min().y()),
            Point::new(x + osc, bounding_box.max().y()),
        ));
        x += line_spacing;
    }

    // Clip against a slightly larger expolygon (`offset(expolygon, scale_(0.02))`).
    let clip_off = offset_expolygon(
        &rotated,
        (0.02 / scale.factor()) as f32,
        JoinType::Miter,
        3.0,
    )?;
    let mut clip: Vec<Polygon> = Vec::new();
    for e in &clip_off {
        clip.push(e.contour().clone());
        clip.extend(e.holes().iter().cloned());
    }
    let src: Vec<Polyline> = lines
        .iter()
        .map(|l| Polyline::new(vec![l.a, l.b]))
        .collect();
    let mut polylines = intersection_open_polylines(&src, &clip)?;

    // Extend each path `extra` past the boundary in Y (`INFILL_OVERLAP_OVER_SPACING = 0.3`).
    let extra = (min_spacing as f32 * 0.3_f32 + 0.5_f32).floor() as i64;
    for polyline in &mut polylines {
        let mut pts = polyline.points().to_vec();
        if pts.len() < 2 {
            continue;
        }
        // Upstream swaps the point *references* so the lower-Y endpoint is
        // extended down and the higher-Y endpoint up (FillLine.cpp:66-73).
        let (lo, hi) = if pts[0].y() <= pts[pts.len() - 1].y() {
            (0usize, pts.len() - 1)
        } else {
            (pts.len() - 1, 0usize)
        };
        pts[lo] = Point::new(pts[lo].x(), pts[lo].y() - extra);
        pts[hi] = Point::new(pts[hi].x(), pts[hi].y() + extra);
        *polyline = Polyline::new(pts);
    }

    // Connect the lines (`chain_polylines` + `_can_connect` + `contains(Line)`).
    let mut polylines_out: Vec<Polyline> = Vec::new();
    if !polylines.is_empty() {
        // `expolygon_off = offset_ex(expolygon, min_spacing/2)` — one island per
        // input island.
        let expolygon_off =
            offset_expolygon(&rotated, (min_spacing / 2) as f32, JoinType::Miter, 3.0)?
                .into_iter()
                .next();
        let tolerance = (10.0 * EPSILON / scale.factor()) as i64;
        let mut ordered = polylines;
        crate::project_slice::chain_polylines(&mut ordered, scale);
        let mut first = true;
        for polyline in ordered {
            if first {
                polylines_out.push(polyline);
                first = false;
                continue;
            }
            let last_point = polylines_out.last().and_then(|p| p.back()).unwrap();
            let first_point = polyline.points()[0];
            let dx = (last_point.x() - first_point.x()).abs();
            let dy = (last_point.y() - first_point.y()).abs();
            let can_connect = dx >= (line_spacing - line_oscillation) - tolerance
                && dx <= (line_spacing + line_oscillation) + tolerance
                && dy <= diagonal_distance;
            let contained = match &expolygon_off {
                Some(off) => difference_open_polylines(
                    &[Polyline::new(vec![last_point, first_point])],
                    &expolygon_to_polygons(off),
                )?
                .is_empty(),
                None => false,
            };
            if can_connect
                && contained
                && let Some(back) = polylines_out.last_mut()
            {
                back.append_points(polyline.points().iter().copied());
                continue;
            }
            polylines_out.push(polyline);
        }
    }

    // Rotate back (`it->rotate(direction.first)` — origin rotation).
    let mut out = Vec::with_capacity(polylines_out.len());
    for polyline in &polylines_out {
        out.push(Polyline::new(crate::fill::checked_rotate::rotate_points(
            polyline.points().to_vec(),
            direction,
        )?));
    }
    Ok(out)
}

fn expolygon_to_polygons(expolygon: &ExPolygon) -> Vec<Polygon> {
    let mut out = Vec::with_capacity(expolygon.holes().len() + 1);
    out.push(expolygon.contour().clone());
    out.extend(expolygon.holes().iter().cloned());
    out
}

/// `Fill::_adjust_solid_spacing` (`FillBase.cpp:255-271`): snap `distance` so
/// the bounding width is a near-integer multiple, capping the inflation at 20%.
pub(crate) fn adjust_solid_spacing(width: i64, distance: i64) -> i64 {
    if distance <= 0 {
        return distance;
    }
    let epsilon = EPSILON as i64;
    // `coord_t((width - EPSILON) / distance)` — truncating division.
    let number_of_intervals = (width - epsilon) / distance;
    let mut distance_new = if number_of_intervals == 0 {
        distance
    } else {
        (width - epsilon) / number_of_intervals
    };
    let factor = distance_new as f64 / distance as f64;
    const FACTOR_MAX: f64 = 1.2;
    if factor > FACTOR_MAX {
        distance_new = ((distance as f64) * FACTOR_MAX + 0.5).floor() as i64;
    }
    distance_new
}
