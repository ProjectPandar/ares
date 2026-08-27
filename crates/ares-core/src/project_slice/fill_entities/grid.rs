//! Grid sparse infill — `FillGrid::fill_surface` (`FillRectilinear.cpp:3422-
//! 3446`): two scanline families at 0° and 90° with the density split
//! between them, one connector pass, and polylines reversed on odd layers.

use crate::{
    SliceError,
    fill::cross_hatch::{CrossHatchFillParams, fill_surface_multilines},
    geometry::{CoordinateScale, Polyline},
    project_slice::{
        fill_entities::{
            FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath, LayerFillEntities,
        },
        group_fills::SurfaceFill,
        perimeters::flow::with_spacing,
    },
};

const SWEEPS: [f32; 2] = [0.0, std::f32::consts::FRAC_PI_2];

pub(super) fn append(
    output: &mut LayerFillEntities,
    fill: SurfaceFill,
    z: f64,
    layer_id: usize,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    let polylines = grid_polylines_inner(&fill, z, scale, layer_id)?;
    if polylines.is_empty() {
        return Ok(());
    }
    let flow = with_spacing(fill.params.flow, fill.params.spacing as f32);
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
    });
    Ok(())
}

fn grid_polylines_inner(
    fill: &SurfaceFill,
    z: f64,
    scale: crate::geometry::CoordinateScale,
    layer_id: usize,
) -> Result<Vec<Polyline>, SliceError> {
    let params = CrossHatchFillParams {
        z,
        spacing: fill.params.spacing,
        overlap: fill.params.overlap,
        angle: fill.params.angle,
        density: (0.01_f64 * f64::from(fill.params.density)) as f32,
        multiline: fill.params.multiline,
        anchor_length: fill.params.anchor_length,
        anchor_length_max: fill.params.anchor_length_max,
        dont_sort: false,
    };
    let mut result = Vec::new();
    for expolygon in &fill.expolygons {
        let mut polylines =
            fill_surface_multilines(expolygon, params, &SWEEPS, scale).map_err(grid_error)?;
        if layer_id % 2 == 1 {
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
