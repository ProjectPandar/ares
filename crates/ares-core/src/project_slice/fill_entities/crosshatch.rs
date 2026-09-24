use crate::{
    SliceError,
    fill::cross_hatch::{CrossHatchFillParams, fill_surface},
    geometry::CoordinateScale,
    project_slice::group_fills::SurfaceFill,
};

use super::{
    FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath, LayerFillEntities,
    geometry_error,
};

pub(super) fn append(
    output: &mut LayerFillEntities,
    fill: SurfaceFill,
    z: f64,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
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
    for expolygon in fill.expolygons {
        let polylines = fill_surface(&expolygon, params, scale).map_err(geometry_error)?;
        if polylines.is_empty() {
            continue;
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
    }
    Ok(())
}
