use crate::project_slice::perimeters::classic::materialize::Polyline3;
use crate::{
    SliceError,
    fill::cross_hatch::{CrossHatchFillParams, fill_surface},
    geometry::CoordinateScale,
    project_slice::group_fills::{SurfaceFill, SurfaceFillParams},
};

use super::{
    FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath, LayerFillEntities,
    geometry_error,
};

pub(super) fn expolygon_polylines(
    expolygon: &crate::geometry::ExPolygon,
    params: SurfaceFillParams,
    z: f64,
    scale: CoordinateScale,
) -> Result<Vec<crate::geometry::Polyline>, SliceError> {
    let params = CrossHatchFillParams {
        z,
        spacing: params.spacing,
        overlap: params.overlap,
        angle: params.angle,
        density: (0.01_f64 * f64::from(params.density)) as f32,
        multiline: params.multiline,
        anchor_length: params.anchor_length,
        anchor_length_max: params.anchor_length_max,
        dont_sort: false,
    };
    fill_surface(expolygon, params, scale).map_err(geometry_error)
}

pub(super) fn append(
    output: &mut LayerFillEntities,
    fill: SurfaceFill,
    z: f64,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    for expolygon in fill.expolygons {
        let polylines = expolygon_polylines(&expolygon, fill.params, z, scale)?;
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
