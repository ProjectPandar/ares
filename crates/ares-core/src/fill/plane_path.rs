//! Source-cited rewrite of OrcaSlicer 2.4.2 `Fill/FillPlanePath.cpp/.hpp`.

mod bounds;
mod classic_clip;
mod generate;
mod ordering;
mod output;
#[cfg(test)]
mod tests;

use bounds::Bounds;
use output::InfillPolylineOutput;

use super::{
    checked_rotate::{rotate_points, rotate_points_with_trig},
    connect::{FillConnectionParams, connect_infill},
    multiline_offset,
};
use crate::geometry::{
    BoundingBox, ClipperError, CoordinateScale, ExPolygon, JoinType, Point, Polygon, Polyline,
    intersection_open_polylines, offset_expolygon,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlanePathPattern {
    HilbertCurve,
    ArchimedeanChords,
    OctagramSpiral,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PlanePathFillParams {
    pub(crate) spacing: f64,
    pub(crate) overlap: f64,
    pub(crate) density: f32,
    pub(crate) angle: f32,
    pub(crate) multiline: i32,
    pub(crate) resolution: f64,
    pub(crate) anchor_length: f32,
    pub(crate) anchor_length_max: f32,
    pub(crate) object_bounding_box: BoundingBox,
    pub(crate) calibration_order: bool,
}

pub(crate) fn fill_surface(
    surface: &ExPolygon,
    pattern: PlanePathPattern,
    params: PlanePathFillParams,
    scale: CoordinateScale,
) -> Result<Vec<Vec<Polyline>>, ClipperError> {
    let offset = ((params.overlap - 0.5 * params.spacing) / scale.factor()) as f32;
    let components = offset_expolygon(surface, offset, JoinType::Miter, 3.0)?;
    let mut output = Vec::new();
    for component in components {
        let polylines = fill_component(&component, pattern, params, scale)?;
        if !polylines.is_empty() {
            output.push(polylines);
        }
    }
    Ok(output)
}

fn fill_component(
    surface: &ExPolygon,
    pattern: PlanePathPattern,
    params: PlanePathFillParams,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    // `Fill::_infill_direction` adds pi/2 in float precision; plane-path
    // `_layer_angle()` is always zero (`FillPlanePath.hpp:48`).
    let direction = f64::from(params.angle + std::f32::consts::FRAC_PI_2);
    let rotated = rotate_expolygon(surface, -direction)?;
    let mut snug_bounds = Bounds::from_expolygon(&rotated);
    snug_bounds.offset(scaled_offset_coordinate(1.0e-4, scale)?)?;
    snug_bounds.offset(scaled_offset_coordinate(
        params.spacing * f64::from(params.multiline),
        scale,
    )?)?;

    let align = params.density < 0.995;
    let mut generation_bounds = if align {
        Bounds::from_bounding_box(params.object_bounding_box).rotated(-direction)?
    } else {
        snug_bounds
    };
    let shift = if pattern.centered() {
        generation_bounds.center()
    } else {
        generation_bounds.minimum()
    };
    let inverse_shift = Point::new(
        shift
            .x()
            .checked_neg()
            .ok_or(ClipperError::CoordinateOutOfRange)?,
        shift
            .y()
            .checked_neg()
            .ok_or(ClipperError::CoordinateOutOfRange)?,
    );
    let rotated = translate_expolygon(rotated, inverse_shift)?;
    generation_bounds.translate(inverse_shift)?;
    if align {
        snug_bounds.translate(inverse_shift)?;
    }

    let distance =
        params.spacing / scale.factor() * f64::from(params.multiline) / f64::from(params.density);
    if !distance.is_finite() || distance <= 0.0 {
        return Err(ClipperError::CoordinateOutOfRange);
    }
    let min_x = ceil_coordinate(generation_bounds.minimum().x() as f64 / distance)?;
    let min_y = ceil_coordinate(generation_bounds.minimum().y() as f64 / distance)?;
    let max_x = ceil_coordinate(generation_bounds.maximum().x() as f64 / distance)?;
    let max_y = ceil_coordinate(generation_bounds.maximum().y() as f64 / distance)?;
    let resolution = params.resolution / scale.factor() / distance;
    let mut generated = if align {
        InfillPolylineOutput::clipped(snug_bounds, distance)
    } else {
        InfillPolylineOutput::plain(distance)
    };
    generate::generate(
        pattern,
        min_x,
        min_y,
        max_x,
        max_y,
        resolution,
        &mut generated,
    )?;
    let points = generated.result();
    if points.len() < 2 {
        return Ok(Vec::new());
    }

    let polylines = multiline_offset::apply(
        vec![Polyline::new(points)],
        params.multiline,
        params.spacing,
        scale,
    )?;
    let clip = expolygon_polygons(&rotated);
    if let Ok(path) = std::env::var("ARES_DUMP_PLANEBOUND") {
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(file, "D {direction}");
            for polygon in &clip {
                for point in polygon.points() {
                    let _ = writeln!(
                        file,
                        "B {:.6} {:.6}",
                        scale.unscale(point.x()),
                        scale.unscale(point.y())
                    );
                }
            }
        }
    }
    let clipped = if pattern == PlanePathPattern::OctagramSpiral {
        classic_clip::intersect(&polylines, &clip)?
    } else {
        intersection_open_polylines(&polylines, &clip)?
    };
    if clipped.is_empty() {
        return Ok(Vec::new());
    }
    let mut chained = if params.anchor_length_max < 0.05 || params.density > 0.5 {
        let mut clipped = clipped;
        ordering::chain(&mut clipped, pattern, params.calibration_order);
        clipped
    } else {
        connect_infill(
            clipped,
            &rotated,
            params.spacing,
            FillConnectionParams {
                anchor_length: params.anchor_length,
                anchor_length_max: params.anchor_length_max,
                multiline: params.multiline,
                dont_sort: false,
            },
            scale,
        )?
    };
    translate_polylines(&mut chained, shift)?;
    rotate_polylines(&mut chained, direction)?;
    Ok(chained)
}

impl PlanePathPattern {
    const fn centered(self) -> bool {
        matches!(self, Self::ArchimedeanChords | Self::OctagramSpiral)
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

fn translate_expolygon(expolygon: ExPolygon, delta: Point) -> Result<ExPolygon, ClipperError> {
    let (contour, holes) = expolygon.into_parts();
    Ok(ExPolygon::new(
        translate_polygon(contour, delta)?,
        holes
            .into_iter()
            .map(|hole| translate_polygon(hole, delta))
            .collect::<Result<_, _>>()?,
    ))
}

fn translate_polygon(polygon: Polygon, delta: Point) -> Result<Polygon, ClipperError> {
    Ok(Polygon::new(
        polygon
            .into_points()
            .into_iter()
            .map(|point| add(point, delta))
            .collect::<Result<_, _>>()?,
    ))
}

fn translate_polylines(polylines: &mut [Polyline], delta: Point) -> Result<(), ClipperError> {
    for polyline in polylines {
        let points = std::mem::replace(polyline, Polyline::new(Vec::new())).into_points();
        *polyline = Polyline::new(
            points
                .into_iter()
                .map(|point| add(point, delta))
                .collect::<Result<_, _>>()?,
        );
    }
    Ok(())
}

fn rotate_polylines(polylines: &mut [Polyline], angle: f64) -> Result<(), ClipperError> {
    let (cosine, sine) = (angle.cos(), angle.sin());
    for polyline in polylines {
        let points = std::mem::replace(polyline, Polyline::new(Vec::new())).into_points();
        *polyline = Polyline::new(rotate_points_with_trig(points, cosine, sine)?);
    }
    Ok(())
}

fn add(point: Point, delta: Point) -> Result<Point, ClipperError> {
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
}

fn expolygon_polygons(expolygon: &ExPolygon) -> Vec<Polygon> {
    std::iter::once(expolygon.contour().clone())
        .chain(expolygon.holes().iter().cloned())
        .collect()
}

fn scaled_offset_coordinate(value: f64, scale: CoordinateScale) -> Result<i64, ClipperError> {
    scale
        .checked_scale(value)
        .map(|_| (value / scale.factor()).round() as i64)
        .ok_or(ClipperError::CoordinateOutOfRange)
}

fn ceil_coordinate(value: f64) -> Result<i64, ClipperError> {
    let value = value.ceil();
    if value.is_finite() && (i64::MIN as f64..-(i64::MIN as f64)).contains(&value) {
        Ok(value as i64)
    } else {
        Err(ClipperError::CoordinateOutOfRange)
    }
}
