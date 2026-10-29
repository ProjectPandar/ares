//! TPMS-FK sparse infill — `FillTpmsFK::fill_surface` → `_fill_surface_single`
//! (`FillTpmsFK.cpp:117`): Fischer–Koch S TPMS via marching-squares. Like
//! `FillTpmsD`/`FillGyroid`, it reads the raw `this->angle` (no
//! `_infill_direction`/`_layer_angle`).
use crate::project_slice::perimeters::classic::materialize::Polyline3;

use crate::{
    SliceError,
    fill::tpms_fk::{TpmsFKFillParams, fill_tpmsfk_surface},
    geometry::{CoordinateScale, JoinType, offset_expolygon},
    project_slice::group_fills::SurfaceFill,
};

use super::{FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath, LayerFillEntities};

pub(super) fn polylines(
    fill: &SurfaceFill,
    layer_z: f64,
    scale: CoordinateScale,
) -> Result<Vec<crate::geometry::Polyline>, SliceError> {
    let params = TpmsFKFillParams {
        spacing: fill.params.spacing,
        density: (0.01_f64 * f64::from(fill.params.density)) as f32,
        angle: fill.params.angle,
        layer_z,
        multiline: fill.params.multiline,
        anchor_length: fill.params.anchor_length,
        anchor_length_max: fill.params.anchor_length_max,
        dont_sort: false,
    };
    // `Fill::fill_surface` offsets the surface by `overlap - spacing/2`
    // (`FillBase.cpp:108`) before `_fill_surface_single`.
    let delta = ((fill.params.overlap - 0.5 * fill.params.spacing) / scale.factor()) as f32;
    let mut polylines = Vec::new();
    for expolygon in &fill.expolygons {
        for component in
            &offset_expolygon(expolygon, delta, JoinType::Miter, 3.0).map_err(tpmsfk_error)?
        {
            polylines.extend(fill_tpmsfk_surface(component, &params, scale).map_err(tpmsfk_error)?);
        }
    }
    Ok(polylines)
}

pub(super) fn append(
    output: &mut LayerFillEntities,
    fill: SurfaceFill,
    layer_z: f64,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    let polylines = polylines(&fill, layer_z, scale)?;
    if polylines.is_empty() {
        return Ok(());
    }
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
    Ok(())
}

fn tpmsfk_error(error: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput(format!("tpms-fk infill generation failed: {error:?}"))
}
