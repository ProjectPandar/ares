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
    pub(crate) bridge_angle: Option<f32>,
    pub(crate) reference_point: Point,
    pub(crate) dont_adjust: bool,
    pub(crate) anchor_length_max: f32,
    pub(crate) link_max_length: f64,
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
    let direction = infill_direction(params);
    let (outer_offset, inner_offset) = scaled_offsets(scale, params.overlap, params.spacing)?;
    let mut slice =
        prepare_rectilinear_contours(expolygon, -f64::from(direction), outer_offset, inner_offset)?;
    if !slice.contours.iter().any(|contour| contour.inner) {
        return Ok(MonotonicFillOutput {
            polylines: Vec::new(),
            spacing: params.spacing as f32,
        });
    }
    let (source_minimum_x, maximum_x) = x_bounds(&slice.source);
    let source_width = maximum_x - source_minimum_x;
    let mut line_spacing = scaled_line_spacing(params.spacing, params.density, scale);
    let full_infill = params.density > 0.9999;
    let minimum_x = if full_infill && !params.dont_adjust {
        line_spacing = adjust_solid_spacing(source_width, line_spacing);
        source_minimum_x
    } else {
        let reference = rotate_point(params.reference_point, -f64::from(direction))?;
        let minimum_x = align_to_grid(source_minimum_x, line_spacing, reference.x())?;
        if let Ok(path) = std::env::var("ARES_DUMP_MONO") {
            use std::io::Write;
            if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
                let _ = writeln!(
                    file,
                    "MONO dir={direction} min_x={minimum_x} spacing={line_spacing} ref={}",
                    reference.x()
                );
            }
        }
        minimum_x
    };
    let grid_spacing = (line_spacing as f64 * scale.factor()) as f32;
    let width = maximum_x - minimum_x;
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
    let x0 = if full_infill {
        minimum_x
            .checked_add((line_spacing + scaled_epsilon) / 2)
            .ok_or(ClipperError::CoordinateOutOfRange)?
    } else {
        minimum_x
    };
    // Orca's generic horizontal-shift path applies `coord_t += float` even when
    // the monotonic shift is zero, so large scan origins round through f32.
    let x0 = (x0 as f32) as i64;
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
        super::traverse::generate(&slice, false)
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
fn remove_duplicate_consecutive(mut polyline: Polyline) -> Polyline {
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
