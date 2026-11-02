//! Cubic sparse infill — `FillCubic::fill_surface`
//! (`FillRectilinear.cpp:3603-3618`): count one retains three z-shifted
//! scanline families; count greater than one dispatches trapezoidal mode 3.
use crate::project_slice::perimeters::classic::materialize::Polyline3;

use crate::{
    SliceError,
    fill::multiline::{MultilineFillParams, Sweep, fill_surface},
    geometry::CoordinateScale,
    project_slice::group_fills::SurfaceFill,
};

use super::{FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath, LayerFillEntities};

#[derive(Clone, Copy)]
pub(super) struct LayerGeometry {
    pub(super) print_z: f64,
    pub(super) layer_id: usize,
}

pub(super) fn polylines(
    fill: &SurfaceFill,
    geometry: LayerGeometry,
    scale: CoordinateScale,
    object_reference: crate::geometry::Point,
) -> Result<Vec<crate::geometry::Polyline>, SliceError> {
    let shift = (std::f64::consts::FRAC_1_SQRT_2 * geometry.print_z) as f32;
    // Raw upstream sweep bases {0, π/3, 2π/3}; the +π/2 frame offset from
    // `Fill::_infill_direction` (FillBase.cpp:329) is folded into
    // `params.angle` below to reproduce upstream's f32 accumulation order.
    let sweeps = [
        Sweep { angle: 0.0, shift },
        Sweep {
            angle: std::f32::consts::FRAC_PI_3,
            shift: -shift,
        },
        Sweep {
            angle: 2.0 * std::f32::consts::FRAC_PI_3,
            shift,
        },
    ];
    let params = MultilineFillParams {
        spacing: fill.params.spacing,
        overlap: fill.params.overlap,
        // `Fill::_infill_direction` (FillBase.cpp:329) adds π/2 to the frame
        // angle once per fill call; the sweep bases ride on that frame.
        angle: fill.params.angle + std::f32::consts::FRAC_PI_2,
        reference: object_reference,
        density: (0.01_f64 * f64::from(fill.params.density)) as f32,
        multiline: fill.params.multiline,
        anchor_length: fill.params.anchor_length,
        anchor_length_max: fill.params.anchor_length_max,
        dont_sort: false,
    };
    let mut polylines = Vec::new();
    for expolygon in &fill.expolygons {
        let lines = if params.multiline > 1 {
            crate::fill::cubic_trapezoidal::fill_surface(
                expolygon,
                params,
                crate::fill::cubic_trapezoidal::SurfaceGeometry {
                    layer_id: geometry.layer_id,
                    thickness_layers: fill.representative.thickness_layers,
                    print_z: geometry.print_z,
                },
                scale,
            )
        } else {
            fill_surface(expolygon, params, &sweeps, scale)
        };
        polylines.extend(lines.map_err(cubic_error)?);
    }
    Ok(polylines)
}

pub(super) fn append(
    output: &mut LayerFillEntities,
    mut fill: SurfaceFill,
    geometry: LayerGeometry,
    scale: CoordinateScale,
    object_reference: crate::geometry::Point,
) -> Result<(), SliceError> {
    let expolygons = std::mem::take(&mut fill.expolygons);
    for expolygon in expolygons {
        fill.expolygons = vec![expolygon];
        let polylines = polylines(&fill, geometry, scale, object_reference)?;
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

fn cubic_error(error: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput(format!("cubic infill generation failed: {error:?}"))
}
