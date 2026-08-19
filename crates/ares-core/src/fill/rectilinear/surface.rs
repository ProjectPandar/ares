use crate::geometry::{ClipperError, CoordinateScale, ExPolygon, Point, Polyline};

use super::segments::{populate_vertical_lines, prepare_rectilinear_contours};
use super::{
    chain_monotonic_regions, compute_region_costs, connect_contours, connect_region_neighbors,
    emit_monotonic_polylines, fast_round_up, generate_monotonic_regions, insert_phony_outer_pairs,
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

    let (minimum_x, maximum_x) = x_bounds(&slice.source);
    let width = maximum_x - minimum_x;
    let nominal = checked_scale(scale, params.spacing)?;
    let mut line_spacing = (nominal as f32 / params.density) as i64;
    if params.density > 0.9999 && !params.dont_adjust {
        line_spacing = adjust_solid_spacing(width, line_spacing);
    }
    let actual_spacing = (line_spacing as f64 * scale.factor()) as f32;
    let count = usize::try_from(
        (i128::from(width) + i128::from(line_spacing) - 1) / i128::from(line_spacing),
    )
    .map_err(|_| ClipperError::CoordinateOutOfRange)?;
    let scaled_epsilon = checked_scale(scale, 1.0e-4)?;
    let x0 = if params.density > 0.9999 {
        minimum_x
            .checked_add((line_spacing + scaled_epsilon) / 2)
            .ok_or(ClipperError::CoordinateOutOfRange)?
    } else {
        minimum_x
    };
    populate_vertical_lines(&mut slice, count, x0, line_spacing)?;
    let link_max_length = checked_scale(scale, params.link_max_length)? as f64;
    connect_contours(&mut slice, params.anchor_length_max < 0.05, link_max_length);
    insert_phony_outer_pairs(&mut slice.lines);
    let mut regions = generate_monotonic_regions(&slice.lines);
    connect_region_neighbors(&mut regions, &slice.lines);
    compute_region_costs(&mut regions, &slice, scale);
    let path = chain_monotonic_regions(&regions, &slice, scale);
    let polylines: Vec<Polyline> = emit_monotonic_polylines(&path, &regions, &slice, scale)
        .into_iter()
        .map(|polyline| rotate_polyline(polyline, f64::from(direction)))
        .collect::<Result<_, _>>()?;
    Ok(MonotonicFillOutput {
        polylines,
        spacing: actual_spacing,
    })
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

fn checked_scaled_f32(scale: CoordinateScale, value: f64) -> Result<f32, ClipperError> {
    let scaled = (value / scale.factor()) as f32;
    if !scaled.is_finite() || !(i64::MIN as f32..-(i64::MIN as f32)).contains(&scaled) {
        return Err(ClipperError::CoordinateOutOfRange);
    }
    Ok((scaled as i64) as f32)
}

fn adjust_solid_spacing(width: i64, distance: i64) -> i64 {
    let intervals = (width - 1) / distance;
    let mut adjusted = if intervals == 0 {
        distance
    } else {
        (width - 1) / intervals
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

fn checked_scale(scale: CoordinateScale, value: f64) -> Result<i64, ClipperError> {
    scale
        .checked_scale(value)
        .ok_or(ClipperError::CoordinateOutOfRange)
}

fn rotate_polyline(polyline: Polyline, angle: f64) -> Result<Polyline, ClipperError> {
    let cosine = angle.cos();
    let sine = angle.sin();
    polyline
        .into_points()
        .into_iter()
        .map(|point| {
            checked_point(
                cosine * point.x() as f64 - sine * point.y() as f64,
                sine * point.x() as f64 + cosine * point.y() as f64,
            )
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Polyline::new)
}

fn checked_point(x: f64, y: f64) -> Result<Point, ClipperError> {
    let x = fast_round_up(x);
    let y = fast_round_up(y);
    if !x.is_finite()
        || !y.is_finite()
        || !(i64::MIN as f64..-(i64::MIN as f64)).contains(&x)
        || !(i64::MIN as f64..-(i64::MIN as f64)).contains(&y)
    {
        return Err(ClipperError::CoordinateOutOfRange);
    }
    Ok(Point::new(x as i64, y as i64))
}
