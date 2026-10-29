//! Grid sparse infill — `FillGrid::fill_surface` (`FillRectilinear.cpp:3422-
//! 3446`): two scanline families at 0° and 90° with the density split
//! between them, one connector pass, and polylines reversed on odd layers.
//! `Fill.cpp:1325` disables later path reversal, not collection sorting.
use crate::project_slice::perimeters::classic::materialize::Polyline3;

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
    // Raw upstream sweep bases {0, π/2}; the +π/2 frame offset from
    // `Fill::_infill_direction` (FillBase.cpp:329) is folded into
    // `params.angle` below so each family total reproduces upstream's f32
    // accumulation order (frame rounding first, then the sweep-base add).
    Sweep {
        angle: 0.0,
        shift: 0.0,
    },
    Sweep {
        angle: std::f32::consts::FRAC_PI_2,
        shift: 0.0,
    },
];

pub(super) struct LayerGeometry {
    pub(super) layer_id: usize,
    pub(super) bounds: Option<crate::geometry::BoundingBox>,
}

pub(super) fn append(
    output: &mut LayerFillEntities,
    fill: SurfaceFill,
    scale: CoordinateScale,
    object_reference: crate::geometry::Point,
    geometry: LayerGeometry,
) -> Result<(), SliceError> {
    for expolygon in &fill.expolygons {
        let polylines = expolygon_polylines(&fill, expolygon, scale, object_reference, &geometry)?;
        if polylines.is_empty() {
            continue;
        }
        // Fill.cpp:1326-1357 / FillBase.cpp:161-182: one collection per surface call.
        let flow = super::materialized_flow(fill.params, fill.params.spacing as f32)?;
        output.collections.push(FillExtrusionCollection {
            entities: polylines
                .into_iter()
                .map(|polyline| {
                    FillExtrusionEntity::Oriented(FillExtrusionPath {
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

pub(super) fn polylines(
    fill: &SurfaceFill,
    scale: crate::geometry::CoordinateScale,
    object_reference: crate::geometry::Point,
    geometry: LayerGeometry,
) -> Result<Vec<Polyline>, SliceError> {
    let mut result = Vec::new();
    for expolygon in &fill.expolygons {
        result.extend(expolygon_polylines(
            fill,
            expolygon,
            scale,
            object_reference,
            &geometry,
        )?);
    }
    Ok(result)
}

fn expolygon_polylines(
    fill: &SurfaceFill,
    expolygon: &crate::geometry::ExPolygon,
    scale: CoordinateScale,
    object_reference: crate::geometry::Point,
    geometry: &LayerGeometry,
) -> Result<Vec<Polyline>, SliceError> {
    let params = MultilineFillParams {
        spacing: fill.params.spacing,
        overlap: fill.params.overlap,
        // `Fill::_infill_direction` (FillBase.cpp:329) adds π/2 to the frame
        // angle once per fill call; the sweep bases ride on that frame, so
        // fold the offset here to reproduce upstream's f32 accumulation.
        angle: fill.params.angle + std::f32::consts::FRAC_PI_2,
        reference: object_reference,
        density: (0.01_f64 * f64::from(fill.params.density)) as f32,
        multiline: fill.params.multiline,
        anchor_length: fill.params.anchor_length,
        anchor_length_max: fill.params.anchor_length_max,
        dont_sort: false,
    };
    // FillRectilinear.cpp:3426-3444: multiline mode0 has XY transpose,
    // while only the ordinary count1 branch reverses odd-layer paths.
    let mut polylines = if params.multiline > 1 {
        let Some(bounds) = geometry.bounds else {
            unreachable!("Grid multiline bounds are prepared once per object");
        };
        crate::fill::grid_trapezoidal::fill_surface(
            expolygon,
            params,
            crate::fill::grid_trapezoidal::SurfaceGeometry {
                layer_id: geometry.layer_id,
                thickness_layers: fill.representative.thickness_layers,
                bounds,
            },
            scale,
        )
    } else {
        fill_surface(expolygon, params, &SWEEPS, scale)
    }
    .map_err(grid_error)?;
    // Odd UPSTREAM layer ids (0-based) reverse the grid polylines
    // (FillRectilinear.cpp:3441); our layer ids are 1-based, so the flip is
    // on even ids.
    if params.multiline == 1 && geometry.layer_id % 2 == 1 {
        for polyline in &mut polylines {
            polyline.reverse();
        }
    }
    Ok(polylines)
}

fn grid_error(error: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput(format!("grid infill generation failed: {error:?}"))
}
