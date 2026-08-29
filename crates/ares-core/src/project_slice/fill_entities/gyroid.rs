//! Standard parametric Gyroid sparse infill (`FillGyroid.cpp:288-324`).

use crate::{
    SliceError,
    fill::gyroid::{GyroidFillParams, fill_surface},
    geometry::CoordinateScale,
    project_slice::group_fills::SurfaceFill,
};

use super::{FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath, LayerFillEntities};

pub(super) fn append(
    output: &mut LayerFillEntities,
    fill: SurfaceFill,
    print_z: f64,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    if fill.params.gyroid_optimized {
        return Err(SliceError::UnsupportedProjectFeature(
            "gyroid_optimized".to_owned(),
        ));
    }
    let params = GyroidFillParams {
        z: print_z,
        spacing: fill.params.spacing,
        overlap: fill.params.overlap,
        angle: fill.params.angle,
        density: (0.01_f64 * f64::from(fill.params.density)) as f32,
        multiline: fill.params.multiline,
        anchor_length: fill.params.anchor_length,
        anchor_length_max: fill.params.anchor_length_max,
        dont_sort: false,
    };
    let mut polylines = Vec::new();
    for expolygon in &fill.expolygons {
        polylines.extend(fill_surface(expolygon, params, scale).map_err(gyroid_error)?);
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
    });
    Ok(())
}

fn gyroid_error(error: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput(format!("gyroid infill generation failed: {error:?}"))
}
