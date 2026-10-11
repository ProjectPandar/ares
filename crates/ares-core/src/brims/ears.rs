//! Source-owned Brim.cpp automatic ear area shared by both Rust adapters.
use crate::geometry::{
    CoordinateScale, ExPolygon, JoinType, Point, Polygon, difference_ex, intersection_ex,
    offset_expolygons,
};
use crate::{BrimOptions, BrimPath, LayerContours, Point2, SliceError};

mod auto;
pub(crate) mod paths;

pub(crate) struct AreaParameters {
    pub(crate) gap: f32,
    pub(crate) width: f32,
    pub(crate) spacing: f32,
    pub(crate) resolution: f64,
    pub(crate) detection_length: i64,
    pub(crate) max_angle: f64,
}

pub(crate) fn area(
    source: &[ExPolygon],
    parameters: AreaParameters,
) -> Result<(Vec<ExPolygon>, Vec<ExPolygon>), SliceError> {
    let AreaParameters {
        gap,
        width,
        spacing,
        resolution,
        detection_length,
        max_angle,
    } = parameters;
    let mut result = Vec::new();
    for poly in source {
        let contour = ExPolygon::new(poly.contour().clone(), Vec::new());
        let inner = offset_expolygons(&[contour], gap, JoinType::Round, resolution)
            .map_err(geometry_error)?;
        let radius = (width - gap - spacing) as i64;
        let outer = auto::make_auto(&inner, radius, detection_length, max_angle, true);
        result.extend(difference_ex(&outer, &inner).map_err(geometry_error)?);
        let holes: Vec<_> = poly
            .holes()
            .iter()
            .map(|hole| {
                let mut reversed = hole.clone();
                reversed.reverse();
                ExPolygon::new(reversed, Vec::new())
            })
            .collect();
        let inset = offset_expolygons(&holes, -width - gap, JoinType::Round, resolution)
            .map_err(geometry_error)?;
        let ear_source =
            offset_expolygons(&holes, -gap, JoinType::Round, resolution).map_err(geometry_error)?;
        let ears = auto::make_auto(&ear_source, radius, detection_length, max_angle, false);
        let inner_ears = difference_ex(&ears, &inset).map_err(geometry_error)?;
        result.extend(intersection_ex(&inner_ears, &holes).map_err(geometry_error)?);
    }
    let footprint =
        offset_expolygons(source, gap, JoinType::Round, resolution).map_err(geometry_error)?;
    Ok((result, footprint))
}

pub(super) fn brim_ear_paths(
    layer: &LayerContours,
    options: BrimOptions,
    effective_line_width: f64,
) -> Result<Vec<BrimPath>, SliceError> {
    let scale = CoordinateScale::Normal;
    let mut source = Vec::new();
    for contour in layer
        .contours()
        .iter()
        .filter(|c| layer.is_outer_contour(c))
    {
        let polygon = scaled_contour(contour, scale)?;
        let holes = layer
            .contours()
            .iter()
            .filter(|c| !layer.is_outer_contour(c))
            .map(|c| scaled_contour(c, scale))
            .collect::<Result<Vec<_>, _>>()?;
        let mut holes = holes
            .into_iter()
            .filter(|h| polygon.contains(&h.points()[0]))
            .collect::<Vec<_>>();
        for hole in &mut holes {
            if hole.is_counter_clockwise() {
                hole.reverse();
            }
        }
        let mut polygon = polygon;
        if !polygon.is_counter_clockwise() {
            polygon.reverse();
        }
        source.push(ExPolygon::new(polygon, holes));
    }
    let spacing = scaled(scale, effective_line_width)? as f32;
    let spacing_mm = f64::from(spacing) * scale.factor();
    let width = (options.width_mm() / spacing_mm / 2.0).floor() * spacing_mm * 2.0;
    let gap = scaled(scale, options.object_gap_mm())? as f32;
    let resolution = scale.scaled_delta(0.0125);
    let (area, footprint) = area(
        &source,
        AreaParameters {
            gap,
            width: scaled(scale, width)? as f32,
            spacing,
            resolution,
            detection_length: scaled(scale, options.brim_ears_detection_length_mm())?,
            max_angle: options.brim_ears_max_angle_degrees(),
        },
    )?;
    let area = difference_ex(&area, &footprint).map_err(geometry_error)?;
    paths::from_area(area, spacing, resolution)?
        .into_iter()
        .map(|points| {
            BrimPath::new(
                points
                    .into_iter()
                    .map(|p| Point2::new(scale.unscale(p.x()), scale.unscale(p.y())))
                    .collect(),
            )
        })
        .collect()
}

fn scaled_contour(contour: &crate::Contour, scale: CoordinateScale) -> Result<Polygon, SliceError> {
    Ok(Polygon::new(
        contour
            .points()
            .iter()
            .map(|p| Ok(Point::new(scaled(scale, p.x())?, scaled(scale, p.y())?)))
            .collect::<Result<Vec<_>, SliceError>>()?,
    ))
}
fn scaled(scale: CoordinateScale, value: f64) -> Result<i64, SliceError> {
    scale
        .checked_scale(value)
        .ok_or_else(|| SliceError::InvalidInput("brim coordinate is out of range".into()))
}
fn geometry_error(_: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput("brim generation failed".into())
}
