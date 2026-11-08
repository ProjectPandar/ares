//! `FillSupportBase::fill_surface` (`FillRectilinear.cpp:3610-3634`).

use super::geom::{align_to_grid, vertical_segments};
use crate::fill::base_support::connect_base_support;
use crate::fill::checked_rotate::rotate_point;
use crate::fill::rectilinear::segments::{populate_vertical_lines, prepare_rectilinear_contours};
use crate::geometry::{
    BoundingBox, ClipperError, CoordinateScale, ExPolygon, Point, Polygon, Polyline,
};

pub(in crate::project_slice) struct SupportBaseFillParams {
    pub(in crate::project_slice) spacing: f64,
    pub(in crate::project_slice) density: f32,
    pub(in crate::project_slice) angle: f32,
    pub(in crate::project_slice) reference: Point,
    pub(in crate::project_slice) reference_angle: f32,
    pub(in crate::project_slice) outer_offset: f32,
}

pub(in crate::project_slice) fn fill_surface(
    expolygon: &ExPolygon,
    params: SupportBaseFillParams,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    let scaled_spacing = f64::from(params.spacing as f32) / scale.factor();
    let line_spacing = (scaled_spacing / f64::from(params.density)) as i64;
    if line_spacing <= 0 {
        return Ok(Vec::new());
    }
    let rotate_by = f64::from(-(params.angle + std::f32::consts::FRAC_PI_2));
    // ExPolygonWithOffset receives coord_t after the caller's float narrowing.
    let outer_offset = params.outer_offset as i64 as f32;
    let mut slice = prepare_rectilinear_contours(expolygon, rotate_by, outer_offset, 0.0)?;
    let boundary: Vec<Polygon> = slice
        .contours
        .iter()
        .filter(|contour| !contour.inner)
        .map(|contour| contour.polygon.clone())
        .collect();
    if boundary.is_empty() {
        return Ok(Vec::new());
    }
    let source_bounds =
        BoundingBox::from_expolygon(&slice.source).ok_or(ClipperError::CoordinateOutOfRange)?;
    let neg_angle = f64::from(-params.reference_angle);
    let reference = rotate_point(params.reference, neg_angle.cos(), neg_angle.sin())?;
    let aligned_min_x = align_to_grid(source_bounds.min().x(), line_spacing, reference.x())?;
    let width = source_bounds.max().x() - aligned_min_x;
    let count = usize::try_from((width + line_spacing - 1) / line_spacing).unwrap_or(0);
    populate_vertical_lines(&mut slice, count, aligned_min_x, line_spacing)?;
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
    let bbox = BoundingBox::from_polygons(&boundary).ok_or(ClipperError::CoordinateOutOfRange)?;
    let rotate_back = params.angle + std::f32::consts::FRAC_PI_2;
    let (cos_a, sin_a) = (f64::from(rotate_back.cos()), f64::from(rotate_back.sin()));
    let connected = connect_base_support(
        fill_lines,
        crate::fill::base_support::BaseSupportGeometry {
            boundary_source: &boundary,
            bounding_box: bbox,
            scale,
        },
        params.spacing,
        &crate::fill::FillParams {
            density: params.density,
            multiline: 1,
            dont_adjust: true,
        },
    )?;
    Ok(connected
        .into_iter()
        .map(|polyline| {
            Polyline::new(
                polyline
                    .points()
                    .iter()
                    .map(|point| {
                        let (x, y) = (point.x() as f64, point.y() as f64);
                        Point::new(
                            (x * cos_a - y * sin_a).round() as i64,
                            (x * sin_a + y * cos_a).round() as i64,
                        )
                    })
                    .collect(),
            )
        })
        .collect())
}
