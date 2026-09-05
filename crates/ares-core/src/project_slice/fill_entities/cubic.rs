//! Cubic sparse infill — `FillCubic::fill_surface`
//! (`FillRectilinear.cpp:3506-3516`): three 60° scanline families shifted by
//! `sqrt(0.5) * print_z` and connected as one pool.

use crate::{
    SliceError,
    fill::multiline::{MultilineFillParams, Sweep, fill_surface},
    geometry::CoordinateScale,
    project_slice::group_fills::SurfaceFill,
};

use super::{FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath, LayerFillEntities};

pub(super) fn append(
    output: &mut LayerFillEntities,
    fill: SurfaceFill,
    print_z: f64,
    scale: CoordinateScale,
    object_reference: crate::geometry::Point,
) -> Result<(), SliceError> {
    let shift = (std::f64::consts::FRAC_1_SQRT_2 * print_z) as f32;
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
        polylines.extend(fill_surface(expolygon, params, &sweeps, scale).map_err(cubic_error)?);
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

fn cubic_error(error: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput(format!("cubic infill generation failed: {error:?}"))
}
