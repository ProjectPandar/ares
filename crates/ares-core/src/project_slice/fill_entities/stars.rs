//! Stars sparse infill — `FillStars::fill_surface`
//! (`FillRectilinear.cpp:3487-3504`): three 60° scanline families; the third
//! is shifted by `(3/2) * spacing * multiline / density` so the stars nest.
//!
//! Only the `multiline == 1` `fill_surface_by_multilines` path is ported; the
//! `multiline > 1` `fill_surface_trapezoidal` branch is deferred.

use crate::{
    SliceError,
    fill::multiline::{MultilineFillParams, fill_surface},
    geometry::CoordinateScale,
    project_slice::group_fills::SurfaceFill,
};

use super::{FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath, LayerFillEntities};

pub(super) fn append(
    output: &mut LayerFillEntities,
    fill: SurfaceFill,
    scale: CoordinateScale,
    object_reference: crate::geometry::Point,
) -> Result<(), SliceError> {
    let density = (0.01_f64 * f64::from(fill.params.density)) as f32;
    let sweeps =
        crate::fill::multiline::stars_sweeps(fill.params.spacing, fill.params.multiline, density);
    let params = MultilineFillParams {
        spacing: fill.params.spacing,
        overlap: fill.params.overlap,
        angle: fill.params.angle + std::f32::consts::FRAC_PI_2,
        reference: object_reference,
        density,
        multiline: fill.params.multiline,
        anchor_length: fill.params.anchor_length,
        anchor_length_max: fill.params.anchor_length_max,
        dont_sort: false,
    };
    let mut polylines = Vec::new();
    for expolygon in &fill.expolygons {
        polylines.extend(fill_surface(expolygon, params, &sweeps, scale).map_err(stars_error)?);
    }
    if polylines.is_empty() {
        return Ok(());
    }
    let flow = super::materialized_flow(fill.params, fill.params.spacing as f32);
    output.collections.push(FillExtrusionCollection {
        entities: polylines
            .into_iter()
            .map(|polyline| {
                FillExtrusionEntity::Path(FillExtrusionPath {
                    polyline,
                    fitting: Vec::new(),
                    role: fill.params.extrusion_role,
                    mm3_per_mm: flow.mm3_per_mm,
                    width: flow.width,
                    height: flow.height,
                })
            })
            .collect(),
        no_sort: false,
        simplify_reversed: false,
    });
    Ok(())
}

fn stars_error(error: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput(format!("stars infill generation failed: {error:?}"))
}
