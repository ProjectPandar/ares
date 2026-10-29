use crate::geometry::{ClipperError, CoordinateScale, ExPolygon, Point, Polyline};

use super::segments::{populate_vertical_lines, prepare_rectilinear_contours};
use super::{
    chain_monotonic_regions, compute_region_costs, connect_contours, connect_region_neighbors,
    emit_monotonic_polylines, generate_monotonic_regions, insert_phony_outer_pairs,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MonotonicFillParams {
    pub(crate) spacing: f64,
    pub(crate) overlap: f64,
    pub(crate) density: f32,
    pub(crate) angle: f32,
    pub(crate) layer_index: usize,
    pub(crate) thickness_layers: u16,
    pub(crate) fixed_angle: bool,
    /// `Fill::_layer_angle` override returning 0 for this pattern
    /// (`FillRectilinear.hpp:44-46` etc.): suppresses the `(idx & 1) ? PI/2 : 0`
    /// alternation independently of `fixed_angle` (`FillBase.cpp:300-316`).
    pub(crate) layer_angle_fixed: bool,
    /// FillBase.hpp:147 / FillBase.cpp:311-313: separate from fixed_angle.
    pub(crate) dont_alternate_fill_direction: bool,
    pub(crate) bridge_angle: Option<f32>,
    pub(crate) reference_point: Point,
    pub(crate) dont_adjust: bool,
    pub(crate) anchor_length_max: f32,
    pub(crate) link_max_length: f64,
    /// `has_consistent_pattern` (`FillRectilinear.cpp:2874` passes it to
    /// `traverse_graph_generate_polylines`; `FillZigZag`/`FillCrossZag`/
    /// `FillLockedZag` override it to `true`, `FillRectilinear`/`FillMonotonic`
    /// default `false`). It makes `next_start` alternate the per-line
    /// direction so the zig-zag serpentine stays consistent across layers.
    pub(crate) consistent_pattern: bool,
    /// `params.horiz_move` (`FillBase.hpp:103`, applied `FillRectilinear.cpp:
    /// 2811-2816`): the crosszag/lockedzag per-layer horizontal grid shift.
    pub(crate) horiz_move: f32,
    /// `extended_object_bounding_box()` x-bounds (`FillBase.cpp:1569-1578`) in
    /// scaled coordinates: the object bounding box transpose-merged and
    /// sqrt(2)-scaled so it covers every rotation. `has_consistent_pattern`
    /// (`FillRectilinear.cpp:2785`) swaps the per-expolygon grid bounds for
    /// these so the vertical-line grid (and the `i_vline2 % 2` serpentine
    /// parity) is anchored to the OBJECT, not to each surface's own bbox.
    /// `None` when `consistent_pattern` is false.
    pub(crate) extended_object_min_x: Option<i64>,
    pub(crate) extended_object_max_x: Option<i64>,
    /// `FillParams::locked_zag` (`FillBase.hpp:106`, `FillRectilinear.cpp:
    /// 2763-2765`): the LockedZag skeleton pass rotates the infill direction
    /// by an additional 90°. The skin pass clears it (`:3919`).
    pub(crate) locked_zag: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MonotonicFillOutput {
    pub(crate) polylines: Vec<Polyline>,
    pub(crate) spacing: f32,
}

pub(crate) fn fill_monotonic_surface(
    expolygon: &ExPolygon,
    params: MonotonicFillParams,
    scale: CoordinateScale,
) -> Result<MonotonicFillOutput, ClipperError> {
    fill_surface(expolygon, params, scale, true)
}

pub(crate) fn fill_rectilinear_surface(
    expolygon: &ExPolygon,
    params: MonotonicFillParams,
    scale: CoordinateScale,
) -> Result<MonotonicFillOutput, ClipperError> {
    fill_surface(expolygon, params, scale, false)
}

fn fill_surface(
    expolygon: &ExPolygon,
    params: MonotonicFillParams,
    scale: CoordinateScale,
    monotonic: bool,
) -> Result<MonotonicFillOutput, ClipperError> {
    let mut direction = infill_direction(params);
    if params.locked_zag {
        direction += std::f32::consts::FRAC_PI_2;
    }
    let (outer_offset, inner_offset) = scaled_offsets(scale, params.overlap, params.spacing)?;
    let mut slice =
        prepare_rectilinear_contours(expolygon, -f64::from(direction), outer_offset, inner_offset)?;
    if !slice.contours.iter().any(|contour| contour.inner) {
        return Ok(MonotonicFillOutput {
            polylines: Vec::new(),
            spacing: params.spacing as f32,
        });
    }
    let (source_minimum_x, source_maximum_x) = x_bounds(&slice.source);
    let source_width = source_maximum_x - source_minimum_x;
    // `bounding_box = has_consistent_pattern ? extended_object_bounding_box()
    // : bounding_box_src` (FillRectilinear.cpp:2785) — for zig-zag/locked-zag
    // the vertical-line grid (and its `i_vline2 % 2` serpentine parity) spans
    // the whole object, not each surface's own bbox.
    let (grid_source_min_x, grid_maximum_x) =
        match (params.extended_object_min_x, params.extended_object_max_x) {
            (Some(min_x), Some(max_x)) if params.consistent_pattern => (min_x, max_x),
            _ => (source_minimum_x, source_maximum_x),
        };
    let mut line_spacing = scaled_line_spacing(params.spacing, params.density, scale);
    let full_infill = params.density > 0.9999;
    // `x0`/`n_vlines` are anchored on `bounding_box.min(0)` — the extended
    // object bbox for consistent patterns, the surface bbox otherwise.
    let grid_minimum_x = if params.consistent_pattern {
        grid_source_min_x
    } else {
        source_minimum_x
    };
    let minimum_x = if full_infill && !params.dont_adjust {
        line_spacing = adjust_solid_spacing(source_width, line_spacing);
        grid_minimum_x
    } else {
        let reference = rotate_point(params.reference_point, -f64::from(direction))?;
        // `bounding_box.merge(align_to_grid(bounding_box.min, spacing, refpt))`
        // — snap the (possibly object-wide) grid minimum onto the reference grid.
        align_to_grid(grid_source_min_x, line_spacing, reference.x())?
    };
    let grid_spacing = (line_spacing as f64 * scale.factor()) as f32;
    // `n_vlines` spans `bounding_box` (object-wide for consistent patterns,
    // per-surface otherwise) — leading/trailing empty lines are kept so the
    // serpentine parity stays anchored.
    let width = grid_maximum_x - minimum_x;
    let flow_spacing = if full_infill && !params.dont_adjust {
        grid_spacing
    } else {
        params.spacing as f32
    };
    let count = usize::try_from(
        (i128::from(width) + i128::from(line_spacing) - 1) / i128::from(line_spacing),
    )
    .map_err(|_| ClipperError::CoordinateOutOfRange)?;
    let scaled_epsilon = checked_scale(scale, 1.0e-4)?;
    let mut x0 = if full_infill {
        minimum_x
            .checked_add((line_spacing + scaled_epsilon) / 2)
            .ok_or(ClipperError::CoordinateOutOfRange)?
    } else {
        minimum_x
    };
    // `FillRectilinear.cpp:2811-2816`: the crosszag/lockedzag `horiz_move`
    // shifts the grid start, adding one column when the shift lands on an
    // odd multiple of the spacing.
    let mut count = count;
    // The upstream compound addition rounds x0 through float even at zero shift.
    let gap_line = (params.horiz_move / line_spacing as f32) as i32;
    if gap_line % 2 == 0 {
        x0 = (x0 as f32 + (params.horiz_move - (i64::from(gap_line) * line_spacing) as f32)) as i64;
    } else {
        x0 = (x0 as f32 + (params.horiz_move - (i64::from(gap_line - 1) * line_spacing) as f32))
            as i64;
        count += 1;
    }
    populate_vertical_lines(&mut slice, count, x0, line_spacing)?;
    let link_max_length = checked_scale(scale, params.link_max_length)? as f64;
    connect_contours(&mut slice, params.anchor_length_max < 0.05, link_max_length);
    let generated = if monotonic {
        insert_phony_outer_pairs(&mut slice.lines);
        let mut regions = generate_monotonic_regions(&slice.lines);
        connect_region_neighbors(&mut regions, &slice.lines);
        compute_region_costs(&mut regions, &slice, scale);
        let path = chain_monotonic_regions(&regions, &slice, scale);
        emit_monotonic_polylines(&path, &regions, &slice, scale)
    } else {
        super::traverse::generate(&slice, params.consistent_pattern)
    };
    let polylines: Vec<Polyline> = generated
        .into_iter()
        .map(|polyline| {
            let rotated = rotate_polyline(polyline, f64::from(direction))?;
            // `fill_surface_by_lines` (FillRectilinear.cpp:2899) drops
            // consecutive points that collide after the rotate-back.
            Ok(remove_duplicate_consecutive(rotated))
        })
        .collect::<Result<_, _>>()?;
    Ok(MonotonicFillOutput {
        polylines,
        spacing: flow_spacing,
    })
}

pub(super) fn scaled_line_spacing(spacing: f64, density: f32, scale: CoordinateScale) -> i64 {
    (spacing / scale.factor() / f64::from(density)) as i64
}

fn infill_direction(params: MonotonicFillParams) -> f32 {
    let mut direction = params.bridge_angle.unwrap_or(params.angle);
    if params.bridge_angle.is_none()
        && !params.fixed_angle
        && !params.layer_angle_fixed
        && !params.dont_alternate_fill_direction
        && (params.layer_index / usize::from(params.thickness_layers)) & 1 == 1
    {
        direction += std::f32::consts::FRAC_PI_2;
    }
    direction + std::f32::consts::FRAC_PI_2
}

pub(super) fn scaled_offsets(
    scale: CoordinateScale,
    overlap: f64,
    spacing: f64,
) -> Result<(f32, f32), ClipperError> {
    Ok((
        checked_scaled_f32(scale, overlap - (0.5 - 0.45) * spacing)?,
        checked_scaled_f32(scale, overlap - 0.5 * spacing)?,
    ))
}

// ExPolygonWithOffset receives coord_t parameters: the float offsets truncate to
// integer microns before ClipperOffset sees them (FillRectilinear.cpp:391-397).
fn checked_scaled_f32(scale: CoordinateScale, value: f64) -> Result<f32, ClipperError> {
    let scaled = value / scale.factor();
    if !scaled.is_finite() || !(i64::MIN as f64..-(i64::MIN as f64)).contains(&scaled) {
        return Err(ClipperError::CoordinateOutOfRange);
    }
    Ok(scaled as f32 as i64 as f32)
}

fn adjust_solid_spacing(width: i64, distance: i64) -> i64 {
    // FillBase.cpp uses `(width - EPSILON)` with EPSILON = 1e-4 in double
    // arithmetic before the integer truncations.
    let width_f = width as f64 - 1.0e-4;
    let number_of_intervals = (width_f / distance as f64) as i64;
    let mut adjusted = if number_of_intervals == 0 {
        distance
    } else {
        (width_f / number_of_intervals as f64) as i64
    };
    if adjusted as f64 / distance as f64 > 1.2 {
        adjusted = (distance as f64 * 1.2 + 0.5).floor() as i64;
    }
    adjusted
}

fn x_bounds(expolygon: &ExPolygon) -> (i64, i64) {
    expolygon
        .contour()
        .points()
        .iter()
        .chain(expolygon.holes().iter().flat_map(|hole| hole.points()))
        .fold((i64::MAX, i64::MIN), |(minimum, maximum), point| {
            (minimum.min(point.x()), maximum.max(point.x()))
        })
}

fn align_to_grid(coordinate: i64, spacing: i64, base: i64) -> Result<i64, ClipperError> {
    let spacing = i128::from(spacing);
    let delta = i128::from(coordinate) - i128::from(base);
    i64::try_from(i128::from(base) + delta.div_euclid(spacing) * spacing)
        .map_err(|_| ClipperError::CoordinateOutOfRange)
}

fn rotate_point(point: Point, angle: f64) -> Result<Point, ClipperError> {
    crate::fill::checked_rotate::rotate_point(point, angle.cos(), angle.sin())
}

fn checked_scale(scale: CoordinateScale, value: f64) -> Result<i64, ClipperError> {
    scale
        .checked_scale(value)
        .ok_or(ClipperError::CoordinateOutOfRange)
}

fn rotate_polyline(polyline: Polyline, angle: f64) -> Result<Polyline, ClipperError> {
    Ok(Polyline::new(crate::fill::checked_rotate::rotate_points(
        polyline.into_points(),
        angle,
    )?))
}

/// `Polyline::remove_duplicate_points` after the rotate-back: consecutive
/// points that round onto the same integer coordinate are dropped.
fn remove_duplicate_consecutive(polyline: Polyline) -> Polyline {
    let mut points = polyline.into_points();
    let mut write = 0;
    for read in 0..points.len() {
        if write == 0 || points[read] != points[write - 1] {
            points.swap(write, read);
            write += 1;
        }
    }
    points.truncate(write);
    Polyline::new(points)
}
