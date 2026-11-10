//! Triangles sparse infill — `FillTriangles::fill_surface`
//! (`FillRectilinear.cpp:3464-3482`): three unshifted scanline families at
//! 0°, 60°, and 120°.
use crate::project_slice::perimeters::classic::materialize::Polyline3;

use crate::{
    SliceError,
    fill::multiline::{MultilineFillParams, Sweep, fill_surface},
    geometry::CoordinateScale,
    project_slice::group_fills::SurfaceFill,
};

use super::{FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath, LayerFillEntities};

const SWEEPS: [Sweep; 3] = [
    // Raw upstream sweep bases {0, π/3, 2π/3}; the +π/2 frame offset from
    // `Fill::_infill_direction` (FillBase.cpp:329) is folded into
    // `params.angle` at the call site to reproduce upstream's f32
    // accumulation order.
    Sweep {
        angle: 0.0,
        shift: 0.0,
    },
    Sweep {
        angle: std::f32::consts::FRAC_PI_3,
        shift: 0.0,
    },
    Sweep {
        angle: 2.0 * std::f32::consts::FRAC_PI_3,
        shift: 0.0,
    },
];

#[derive(Clone, Copy)]
pub(super) struct LayerGeometry {
    pub(super) layer_id: usize,
}

pub(super) fn polylines(
    fill: &SurfaceFill,
    scale: CoordinateScale,
    object_reference: crate::geometry::Point,
    geometry: LayerGeometry,
) -> Result<Vec<crate::geometry::Polyline>, SliceError> {
    let params = MultilineFillParams {
        spacing: fill.params.spacing,
        overlap: fill.params.overlap,
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
            crate::fill::triangles_trapezoidal::fill_surface(
                expolygon,
                params,
                crate::fill::triangles_trapezoidal::SurfaceGeometry {
                    layer_id: geometry.layer_id,
                    thickness_layers: fill.representative.thickness_layers,
                },
                scale,
            )
        } else {
            fill_surface(expolygon, params, &SWEEPS, scale)
        };
        polylines.extend(lines.map_err(triangles_error)?);
    }
    Ok(polylines)
}

pub(super) fn append(
    output: &mut LayerFillEntities,
    mut fill: SurfaceFill,
    scale: CoordinateScale,
    object_reference: crate::geometry::Point,
    geometry: LayerGeometry,
) -> Result<(), SliceError> {
    let expolygons = std::mem::take(&mut fill.expolygons);
    for expolygon in expolygons {
        fill.expolygons = vec![expolygon];
        let polylines = polylines(&fill, scale, object_reference, geometry)?;
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

fn triangles_error(error: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput(format!("triangles infill generation failed: {error:?}"))
}
