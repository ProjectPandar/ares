use crate::{
    SliceError,
    fill::cross_hatch::{CrossHatchFillParams, fill_surface},
    geometry::CoordinateScale,
    project_slice::group_fills::SurfaceFill,
};

use super::{FillExtrusionCollection, FillExtrusionPath, LayerFillEntities, geometry_error};

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
        output.collections.push(FillExtrusionCollection {
            paths: polylines
                .into_iter()
                .map(|polyline| FillExtrusionPath {
                    polyline,
                    fitting: Vec::new(),
                    role: fill.params.extrusion_role,
                    mm3_per_mm: fill.params.flow.mm3_per_mm,
                    width: fill.params.flow.width,
                    height: fill.params.flow.height,
                })
                .collect(),
            no_sort: false,
        });
    }
    Ok(())
}
