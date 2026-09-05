//! Grid sparse infill — `FillGrid::fill_surface` (`FillRectilinear.cpp:3422-
//! 3446`): two scanline families at 0° and 90° with the density split
//! between them, one connector pass, and polylines reversed on odd layers.

use crate::{
    SliceError,
    fill::multiline::{MultilineFillParams, Sweep, fill_surface},
    geometry::{CoordinateScale, Polyline},
    project_slice::{
        fill_entities::{
            FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath, LayerFillEntities,
        },
        group_fills::SurfaceFill,
    },
};

const SWEEPS: [Sweep; 2] = [
    Sweep {
        angle: std::f32::consts::FRAC_PI_2,
        shift: 0.0,
    },
    Sweep {
        angle: 0.0,
        shift: 0.0,
    },
];

pub(super) fn append(
    output: &mut LayerFillEntities,
    fill: SurfaceFill,
    layer_id: usize,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    let polylines = grid_polylines_inner(&fill, scale, layer_id)?;
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

fn grid_polylines_inner(
    fill: &SurfaceFill,
    scale: crate::geometry::CoordinateScale,
    layer_id: usize,
) -> Result<Vec<Polyline>, SliceError> {
    if let Ok(path) = std::env::var("ARES_DUMP_IORDER") {
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(
                file,
                "GRID layer={} angle={:.3} expolygons={}",
                layer_id,
                fill.params.angle,
                fill.expolygons.len()
            );
        }
    }
    let params = MultilineFillParams {
        spacing: fill.params.spacing,
        overlap: fill.params.overlap,
        // `Fill::_infill_direction` (FillBase.cpp:318) adds pi/2; FillGrid keeps a
        // constant angle (`_layer_angle` = 0, FillRectilinear.hpp:77), so the grid
        // frame angle is the configured direction plus pi/2.
        angle: fill.params.angle + std::f32::consts::FRAC_PI_2,
        density: (0.01_f64 * f64::from(fill.params.density)) as f32,
        multiline: fill.params.multiline,
        anchor_length: fill.params.anchor_length,
        anchor_length_max: fill.params.anchor_length_max,
        dont_sort: false,
    };
    let mut result = Vec::new();
    for expolygon in &fill.expolygons {
        let mut polylines = fill_surface(expolygon, params, &SWEEPS, scale).map_err(grid_error)?;
        // Odd UPSTREAM layer ids (0-based) reverse the grid polylines
        // (FillRectilinear.cpp:3441); our layer ids are 1-based, so the flip is
        // on even ids.
        if layer_id % 2 == 0 {
            for polyline in &mut polylines {
                polyline.reverse();
            }
        }
        result.extend(polylines);
    }
    Ok(result)
}

fn grid_error(error: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput(format!("grid infill generation failed: {error:?}"))
}
