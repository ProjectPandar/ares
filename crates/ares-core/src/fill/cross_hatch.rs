mod pattern;
mod transform;

pub(crate) use transform::line_spacing;

use super::{
    connect::{FillConnectionParams, connect_infill},
    multiline_offset,
};
use crate::geometry::{
    ClipperError, CoordinateScale, ExPolygon, JoinType, Polyline, intersection_open_polylines,
    offset_expolygon,
};
use transform::{polyline_length, rotate_expolygon, rotate_polylines, translate_polylines};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CrossHatchFillParams {
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
    params: CrossHatchFillParams,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    debug_assert!(params.anchor_length_max >= 0.05);

    let offset_delta = ((params.overlap - 0.5 * params.spacing) / scale.factor()) as f32;
    let components = offset_expolygon(surface, offset_delta, JoinType::Miter, 3.0)?;
    let mut result = Vec::new();

    for component in components {
        let mut component_paths = fill_component(component, params, scale)?;
        result.append(&mut component_paths);
    }

    Ok(result)
}

fn fill_component(
    component: ExPolygon,
    params: CrossHatchFillParams,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    let rotate = f64::from(params.angle.abs()) >= 1e-4_f64;
    let component = if rotate {
        rotate_expolygon(component, f64::from(-params.angle))?
    } else {
        component
    };

    let (bbox_min, width, height) = transform::aligned_contour_bounds(
        component.contour(),
        params.spacing,
        params.density,
        params.multiline,
        scale,
    )?;
    let mut polylines = pattern::generate_infill_layers(
        params.z / scale.factor(),
        pattern::repeat_ratio(params.density),
        line_spacing(params.spacing, params.density, params.multiline, scale),
        width,
        height,
    )?;
    translate_polylines(&mut polylines, bbox_min)?;
    let polylines = multiline_offset::apply(polylines, params.multiline, params.spacing, scale)?;

    let (contour, holes) = component.into_parts();
    let mut clip = Vec::with_capacity(holes.len() + 1);
    clip.push(contour);
    clip.extend(holes);
    let mut polylines = intersection_open_polylines(&polylines, &clip)?;
    let minimum_length = (0.8 * params.spacing) / scale.factor();
    polylines.retain(|polyline| polyline_length(polyline) >= minimum_length);
    if polylines.is_empty() {
        return Ok(Vec::new());
    }
    let mut clip = clip.into_iter();
    let component = ExPolygon::new(
        clip.next().expect("trusted CrossHatch contour is present"),
        clip.collect(),
    );

    let mut connected = connect_infill(
        polylines,
        &component,
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

#[cfg(test)]
mod tests;
