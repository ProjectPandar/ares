//! Standard parametric Gyroid sparse infill (`FillGyroid.cpp:288-324`).
use crate::project_slice::perimeters::classic::materialize::Polyline3;

use crate::{
    SliceError,
    fill::gyroid::{GyroidFillParams, fill_surface},
    geometry::CoordinateScale,
    project_slice::group_fills::SurfaceFill,
};

use super::{FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath, LayerFillEntities};

pub(super) fn polylines(
    fill: &SurfaceFill,
    print_z: f64,
    layer_height: f64,
    scale: CoordinateScale,
) -> Result<Vec<crate::geometry::Polyline>, SliceError> {
    let params = GyroidFillParams {
        z: print_z,
        spacing: fill.params.spacing,
        overlap: fill.params.overlap,
        angle: fill.params.angle,
        density: (0.01_f64 * f64::from(fill.params.density)) as f32,
        multiline: fill.params.multiline,
        // `params.layer_height` (Fill.cpp:1290) — the planned layer thickness,
        // NOT `flow.height` (which combined infill multiplies).
        layer_height: layer_height as f32,
        gyroid_optimized: fill.params.gyroid_optimized,
        anchor_length: fill.params.anchor_length,
        anchor_length_max: fill.params.anchor_length_max,
        dont_sort: false,
    };
    let mut polylines = Vec::new();
    for expolygon in &fill.expolygons {
        polylines.extend(fill_surface(expolygon, params, scale).map_err(gyroid_error)?);
    }
    Ok(polylines)
}

pub(super) fn append(
    output: &mut LayerFillEntities,
    mut fill: SurfaceFill,
    print_z: f64,
    layer_height: f64,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    let expolygons = std::mem::take(&mut fill.expolygons);
    for expolygon in expolygons {
        fill.expolygons = vec![expolygon];
        let polylines = polylines(&fill, print_z, layer_height, scale)?;
        if polylines.is_empty() {
            continue;
        }
        let polylines = super::join_close_ends::join(
            polylines,
            fill.params.extrusion_role,
            fill.params.spacing,
            (false, true),
            scale,
        );
        let flow = super::materialized_flow(fill.params, fill.params.spacing as f32)?;
        output.collections.push(FillExtrusionCollection {
            entities: polylines
                .into_iter()
                .map(|polyline| {
                    FillExtrusionEntity::Path(FillExtrusionPath {
                        polyline: Polyline3::from_xy(polyline, Vec::new()),
                        z_contoured: false,
                        role: fill.params.extrusion_role,
                        mm3_per_mm: flow.mm3_per_mm,
                        width: flow.width,
                        height: flow.height,
                    })
                })
                .collect(),
            no_sort: false,
        });
    }
    Ok(())
}

fn gyroid_error(error: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput(format!("gyroid infill generation failed: {error:?}"))
}
