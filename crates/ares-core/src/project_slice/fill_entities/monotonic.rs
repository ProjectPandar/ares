use crate::{
    ProcessInfillPattern, SliceError,
    fill::rectilinear::{MonotonicFillParams, fill_monotonic_surface},
    geometry::CoordinateScale,
    project_slice::group_fills::SurfaceFill,
};

use super::{FillExtrusionCollection, FillExtrusionPath, LayerFillEntities, geometry_error};

pub(super) fn append(
    output: &mut LayerFillEntities,
    fill: SurfaceFill,
    pattern: ProcessInfillPattern,
    layer_id: usize,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    let density = (0.01_f64 * f64::from(fill.params.density)) as f32;
    let anchor_length_max = if pattern == ProcessInfillPattern::MonotonicLine {
        0.0
    } else {
        fill.params.anchor_length_max
    };
    let params = MonotonicFillParams {
        spacing: fill.params.spacing,
        overlap: fill.params.overlap,
        density,
        angle: fill.params.angle,
        layer_index: layer_id,
        thickness_layers: fill.representative.thickness_layers.max(1),
        fixed_angle: fill.params.fixed_angle,
        bridge_angle: fill.params.bridge.then_some(fill.params.bridge_angle),
        dont_adjust: false,
        anchor_length_max,
        link_max_length: if !fill.params.bridge && fill.params.density > 80.0 {
            3.0 * fill.params.spacing
        } else {
            0.0
        },
    };
    for expolygon in fill.expolygons {
        let polylines =
            fill_monotonic_surface(&expolygon, params, scale).map_err(geometry_error)?;
        if polylines.is_empty() {
            continue;
        }
        output.collections.push(FillExtrusionCollection {
            paths: polylines
                .into_iter()
                .map(|polyline| FillExtrusionPath {
                    polyline,
                    role: fill.params.extrusion_role,
                    mm3_per_mm: fill.params.flow.mm3_per_mm,
                    width: fill.params.flow.width,
                    height: fill.params.flow.height,
                })
                .collect(),
            no_sort: true,
        });
    }
    Ok(())
}
