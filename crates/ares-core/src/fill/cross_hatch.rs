mod pattern;
mod transform;

pub(crate) use transform::line_spacing;

use super::connect::{FillConnectionParams, connect_infill};
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
    debug_assert_eq!(params.multiline, 1);
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

/// `FillRectilinear::fill_surface_by_multilines`: generates one scanline
/// family per sweep angle (density split across the families), then runs a
/// single connector pass over the combined pool (`FillRectilinear.cpp:2996-
/// 3046`).
pub(crate) fn fill_surface_multilines(
    surface: &ExPolygon,
    params: CrossHatchFillParams,
    sweep_angles: &[f32],
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    debug_assert!(!sweep_angles.is_empty());
    let offset_delta = ((params.overlap - 0.5 * params.spacing) / scale.factor()) as f32;
    let components = offset_expolygon(surface, offset_delta, JoinType::Miter, 3.0)?;
    let per_family = params.density / sweep_angles.len() as f32;
    let mut result = Vec::new();

    for component in components {
        let mut component_paths =
            fill_component_multilines(component, params, per_family, sweep_angles, scale)?;
        result.append(&mut component_paths);
    }

    Ok(result)
}

fn fill_component_multilines(
    component: ExPolygon,
    params: CrossHatchFillParams,
    per_family_density: f32,
    sweep_angles: &[f32],
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    let mut polylines = Vec::new();
    for sweep in sweep_angles {
        let angle = params.angle + sweep;
        polylines.extend(generate_family(
            component.clone(),
            params,
            per_family_density,
            angle,
            scale,
        )?);
    }
    let minimum_length = (0.8 * params.spacing) / scale.factor();
    polylines.retain(|polyline| polyline_length(polyline) >= minimum_length);
    if polylines.is_empty() {
        return Ok(Vec::new());
    }
    connect_infill(
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
    )
}

fn generate_family(
    component: ExPolygon,
    params: CrossHatchFillParams,
    density: f32,
    angle: f32,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    let rotate = f64::from(angle.abs()) >= 1e-4_f64;
    let component = if rotate {
        rotate_expolygon(component, f64::from(-angle))?
    } else {
        component
    };
    let (bbox_min, width, height) = transform::aligned_contour_bounds(
        component.contour(),
        params.spacing,
        density,
        params.multiline,
        scale,
    )?;
    let mut polylines = pattern::generate_infill_layers(
        params.z / scale.factor(),
        pattern::repeat_ratio(density),
        line_spacing(params.spacing, density, params.multiline, scale),
        width,
        height,
    )?;
    translate_polylines(&mut polylines, bbox_min)?;

    let (contour, holes) = component.into_parts();
    let mut clip = Vec::with_capacity(holes.len() + 1);
    clip.push(contour);
    clip.extend(holes);
    let mut polylines = intersection_open_polylines(&polylines, &clip)?;
    if rotate {
        rotate_polylines(&mut polylines, f64::from(angle))?;
    }
    Ok(polylines)
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
