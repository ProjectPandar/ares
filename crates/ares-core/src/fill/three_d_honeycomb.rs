//! OrcaSlicer beta `Fill/Fill3DHoneycomb.cpp` truncated-octahedron slice.

mod grid;
mod tops;
mod wave;

use super::{
    checked_rotate::{rotate_points, rotate_points_with_trig},
    connect::{FillConnectionParams, connect_infill},
    corner_smoothing,
};
use crate::geometry::{
    BoundingBox, ClipperError, CoordinateScale, ExPolygon, Point, Polygon, Polyline,
    intersection_open_polylines,
};

const EPSILON: f64 = 1.0e-4;

#[derive(Clone, Copy)]
pub(crate) struct Params {
    pub(crate) z: f64,
    pub(crate) layer_height: f64,
    pub(crate) thickness_layers: u16,
    pub(crate) spacing: f64,
    pub(crate) overlap: f64,
    pub(crate) angle: f32,
    pub(crate) density: f32,
    pub(crate) multiline: i32,
    pub(crate) complete_top: bool,
    pub(crate) smooth_factor: f64,
    pub(crate) resolution: f64,
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
    let rotate = f64::from(params.angle.abs()) >= EPSILON;
    let expolygon = if rotate {
        rotate_expolygon(surface, -f64::from(params.angle))?
    } else {
        surface.clone()
    };
    let mut bounds =
        BoundingBox::from_polygon(expolygon.contour()).ok_or(ClipperError::CoordinateOutOfRange)?;
    let spacing = params.spacing / scale.factor();
    bounds.offset((5.0 * spacing) as i64);
    let mut request = grid_request(params, scale);
    let module = (request.size * 2.0).round() as i64;
    let minimum = Point::new(
        bounds.min().x().div_euclid(module) * module,
        bounds.min().y().div_euclid(module) * module,
    );
    request.width = (bounds.max().x() - minimum.x()) as f64;
    request.height = (bounds.max().y() - minimum.y()) as f64;
    let mut polylines = grid::make_grid(request);
    for polyline in &mut polylines {
        translate(polyline, minimum)?;
        corner_smoothing::smooth(
            polyline,
            params.smooth_factor,
            params.resolution / scale.factor(),
        );
    }
    let clip = expolygon_polygons(&expolygon);
    let mut polylines = intersection_open_polylines(&polylines, &clip)?;
    let minimum_length = (0.8 * params.spacing) / scale.factor();
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

fn grid_request(params: Params, scale: CoordinateScale) -> grid::Grid {
    let mut z_scale = 2.0_f64.sqrt();
    let multiline = params.multiline as usize;
    let layers = usize::from(params.thickness_layers.max(1));
    let mut size = (params.spacing / scale.factor()) * ((z_scale + 1.0) / 2.0) * multiline as f64
        / f64::from(params.density);
    let layer_height = params.layer_height / scale.factor();
    let layers_per_module = ((size * 2.0) / (z_scale * layer_height) + 0.05).floor();
    if f64::from(params.density) > 0.42 || layers_per_module < 2.0 {
        size =
            (params.spacing / scale.factor()) * 1.1 * multiline as f64 / f64::from(params.density);
        z_scale = (size * 2.0) / (2.0 * layer_height);
    }
    let start = size / 2.0 + (params.layer_height / 2.0) / scale.factor();
    grid::Grid {
        z: (params.z / scale.factor()) * z_scale + start,
        z_last: ((params.z - params.layer_height * multiline as f64 * layers as f64)
            / scale.factor())
            * z_scale
            + start,
        size,
        width: 0.0,
        height: 0.0,
        complete_top: params.complete_top,
        spacing: params.spacing / scale.factor(),
        multiline,
        layers,
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
            (x * x + y * y).sqrt()
        })
        .sum()
}
