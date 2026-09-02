//! OrcaSlicer 2.4.2 `Fill/Fill3DHoneycomb.cpp` truncated-octahedron slice.

use super::{
    checked_rotate::{rotate_points, rotate_points_with_trig},
    connect::{FillConnectionParams, connect_infill},
    multiline_offset,
};
use crate::geometry::{
    BoundingBox, ClipperError, CoordinateScale, ExPolygon, Point, Polygon, Polyline,
    douglas_peucker, intersection_open_polylines,
};

#[derive(Clone, Copy)]
pub(crate) struct Params {
    pub(crate) z: f64,
    pub(crate) spacing: f64,
    pub(crate) overlap: f64,
    pub(crate) angle: f32,
    pub(crate) density: f32,
    pub(crate) multiline: i32,
    pub(crate) anchor_length: f32,
    pub(crate) anchor_length_max: f32,
    pub(crate) dont_sort: bool,
}

pub(crate) fn fill_surface(
    surface: &ExPolygon,
    params: Params,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    let delta = ((params.overlap - 0.5 * params.spacing) / scale.factor()) as f32;
    let components =
        crate::geometry::offset_expolygon(surface, delta, crate::geometry::JoinType::Miter, 3.0)?;
    let mut output = Vec::new();
    for component in components {
        output.extend(fill_component(&component, params, scale)?);
    }
    Ok(output)
}

fn fill_component(
    surface: &ExPolygon,
    params: Params,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    let rotate = f64::from(params.angle.abs()) >= 1.0e-4;
    let expolygon = if rotate {
        rotate_expolygon(surface, -f64::from(params.angle))?
    } else {
        surface.clone()
    };
    let mut bounds =
        BoundingBox::from_polygon(expolygon.contour()).ok_or(ClipperError::CoordinateOutOfRange)?;
    let scaled_spacing = params.spacing / scale.factor();
    bounds.offset((5.0 * scaled_spacing) as i64);

    let mut z_scale = 2.0_f64.sqrt();
    let mut grid_size = scaled_spacing * ((z_scale + 1.0) * 0.5) * f64::from(params.multiline)
        / f64::from(params.density);
    let layer_height = 1.0 / scale.factor();
    let mut layers_per_module = ((2.0 * grid_size) / (z_scale * layer_height) + 0.05).floor();
    if params.density > 0.42 {
        layers_per_module = 2.0;
        grid_size = scaled_spacing * 1.1 * f64::from(params.multiline) / f64::from(params.density);
        z_scale = 2.0 * grid_size / (layers_per_module * layer_height);
    } else {
        layers_per_module = layers_per_module.max(2.0);
        z_scale = 2.0 * grid_size / (layers_per_module * layer_height);
        grid_size = scaled_spacing * ((z_scale + 1.0) * 0.5) * f64::from(params.multiline)
            / f64::from(params.density);
        layers_per_module = ((2.0 * grid_size) / (z_scale * layer_height) + 0.05)
            .floor()
            .max(2.0);
        z_scale = 2.0 * grid_size / (layers_per_module * layer_height);
    }
    let module = (4.0 * grid_size).round() as i64;
    let aligned = Point::new(
        bounds.min().x().div_euclid(module) * module,
        bounds.min().y().div_euclid(module) * module,
    );
    let minimum = Point::new(
        bounds.min().x().min(aligned.x()),
        bounds.min().y().min(aligned.y()),
    );
    let width = (bounds.max().x() - minimum.x()) as f64;
    let height = (bounds.max().y() - minimum.y()) as f64;
    let mut polylines = make_grid(
        params.z / scale.factor() * z_scale,
        grid_size,
        width,
        height,
    )?;
    for polyline in &mut polylines {
        translate(polyline, minimum)?;
        let points = std::mem::replace(polyline, Polyline::new(Vec::new())).into_points();
        *polyline = Polyline::new(douglas_peucker(&points, 5.0 * params.spacing));
    }
    polylines = multiline_offset::apply(polylines, params.multiline, params.spacing, scale)?;
    let clip = expolygon_polygons(&expolygon);
    let mut polylines = intersection_open_polylines(&polylines, &clip)?;
    let minimum_length = 0.8 * scaled_spacing;
    polylines.retain(|polyline| polyline_length(polyline) >= minimum_length);
    if polylines.is_empty() {
        return Ok(Vec::new());
    }
    let mut connected = connect_infill(
        polylines,
        &expolygon,
        params.spacing,
        FillConnectionParams {
            anchor_length: params.anchor_length,
            anchor_length_max: params.anchor_length_max,
            multiline: params.multiline,
            dont_sort: params.dont_sort,
        },
        scale,
    )?;
    if rotate {
        rotate_polylines(&mut connected, f64::from(params.angle))?;
    }
    Ok(connected)
}

fn make_grid(z: f64, grid: f64, width: f64, height: f64) -> Result<Vec<Polyline>, ClipperError> {
    let critical = critical_points(z, grid);
    let cycle = (z + 0.5 * grid) % (2.0 * grid) / (2.0 * grid);
    let mut output = Vec::new();
    if cycle < 0.5 {
        let mut direction = -1.0;
        let mut x = 0.0;
        while x <= width {
            let xs = perpendicular_points(
                &critical,
                PerpendicularRequest {
                    z,
                    grid,
                    length: height,
                    base: x,
                    direction,
                },
            );
            let ys = collinear_points(grid, &critical, height);
            output.push(points(xs, ys, direction > 0.0)?);
            direction = -direction;
            x += grid;
        }
    } else {
        let mut direction = 1.0;
        let mut y = grid;
        while y <= height {
            let xs = collinear_points(grid, &critical, width);
            let ys = perpendicular_points(
                &critical,
                PerpendicularRequest {
                    z,
                    grid,
                    length: width,
                    base: y,
                    direction,
                },
            );
            output.push(points(xs, ys, direction < 0.0)?);
            direction = -direction;
            y += grid;
        }
    }
    Ok(output)
}

fn points(x: Vec<f64>, y: Vec<f64>, reverse: bool) -> Result<Polyline, ClipperError> {
    if x.len() != y.len() {
        return Err(ClipperError::CoordinateOutOfRange);
    }
    let mut points = x
        .into_iter()
        .zip(y)
        .map(|(x, y)| Point::new(x as i64, y as i64))
        .collect::<Vec<_>>();
    if reverse {
        points.reverse();
    }
    Ok(Polyline::new(points))
}

fn critical_points(z: f64, grid: f64) -> Vec<f64> {
    let mut output = vec![0.0];
    let normalized = (tri_wave(z, grid) * 0.5).abs() / grid;
    if normalized > 0.0 {
        output.extend([
            grid * normalized,
            grid * (1.0 - normalized),
            grid * (1.0 + normalized),
            grid * (2.0 - normalized),
        ]);
    }
    output
}

fn collinear_points(grid: f64, critical: &[f64], length: f64) -> Vec<f64> {
    let mut output = vec![0.0];
    let mut location = 0.0;
    while location < length {
        output.extend(critical.iter().map(|critical| location + critical));
        location += 2.0 * grid;
    }
    output.push(length);
    output
}

struct PerpendicularRequest {
    z: f64,
    grid: f64,
    length: f64,
    base: f64,
    direction: f64,
}

fn perpendicular_points(critical: &[f64], request: PerpendicularRequest) -> Vec<f64> {
    let PerpendicularRequest {
        z,
        grid,
        length,
        base,
        direction,
    } = request;
    let mut output = vec![base];
    let mut location = 0.0;
    while location < length {
        output.extend(
            critical
                .iter()
                .map(|critical| base + troct_wave(*critical, grid, z) * direction),
        );
        location += 2.0 * grid;
    }
    output.push(base);
    output
}

// Upstream `triWave` keeps the phase in a C `float` (f32) — the storage
// rounding and fractional extraction happen in single precision
// (`Fill3DHoneycomb.cpp:29-35`); f64 here shifts cut points by one
// scaled unit after truncation.
fn tri_wave(position: f64, grid: f64) -> f64 {
    let mut t = (position / (2.0 * grid) + 0.25) as f32;
    t -= t as i64 as f32;
    (1.0 - (f64::from(t) * 8.0 - 4.0).abs()) * (grid * 0.25) + grid * 0.25
}

fn troct_wave(position: f64, grid: f64, z: f64) -> f64 {
    let offset = tri_wave(z, grid) * 0.5;
    let y = tri_wave(position, grid);
    if y.abs() > offset.abs() {
        y.signum() * offset
    } else {
        y * offset.signum()
    }
}

fn rotate_expolygon(expolygon: &ExPolygon, angle: f64) -> Result<ExPolygon, ClipperError> {
    let rotate =
        |polygon: &Polygon| rotate_points(polygon.points().to_vec(), angle).map(Polygon::new);
    Ok(ExPolygon::new(
        rotate(expolygon.contour())?,
        expolygon
            .holes()
            .iter()
            .map(rotate)
            .collect::<Result<_, _>>()?,
    ))
}

fn rotate_polylines(polylines: &mut [Polyline], angle: f64) -> Result<(), ClipperError> {
    let (cosine, sine) = (angle.cos(), angle.sin());
    for polyline in polylines {
        let points = std::mem::replace(polyline, Polyline::new(Vec::new())).into_points();
        *polyline = Polyline::new(rotate_points_with_trig(points, cosine, sine)?);
    }
    Ok(())
}

fn translate(polyline: &mut Polyline, delta: Point) -> Result<(), ClipperError> {
    let points = std::mem::replace(polyline, Polyline::new(Vec::new())).into_points();
    *polyline = Polyline::new(
        points
            .into_iter()
            .map(|point| {
                Ok(Point::new(
                    point
                        .x()
                        .checked_add(delta.x())
                        .ok_or(ClipperError::CoordinateOutOfRange)?,
                    point
                        .y()
                        .checked_add(delta.y())
                        .ok_or(ClipperError::CoordinateOutOfRange)?,
                ))
            })
            .collect::<Result<_, _>>()?,
    );
    Ok(())
}

fn expolygon_polygons(expolygon: &ExPolygon) -> Vec<Polygon> {
    std::iter::once(expolygon.contour().clone())
        .chain(expolygon.holes().iter().cloned())
        .collect()
}

fn polyline_length(polyline: &Polyline) -> f64 {
    polyline
        .points()
        .windows(2)
        .map(|points| {
            let x = (points[1].x() - points[0].x()) as f64;
            let y = (points[1].y() - points[0].y()) as f64;
            x.hypot(y)
        })
        .sum()
}

#[cfg(test)]
mod tests;
