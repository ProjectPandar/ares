use crate::{
    ProcessInfillPattern, SliceError,
    fill::line::{LineFillParams, fill_line},
    geometry::{CoordinateScale, Point},
    project_slice::group_fills::SurfaceFill,
};

use super::{
    FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath, LayerFillEntities,
    geometry_error,
};

/// Emitted sparse fill for `ProcessInfillPattern::Line` — wraps
/// `fill::line::fill_line` (port of `FillLine::_fill_surface_single`).
pub(super) fn expolygon_polylines(
    expolygon: &crate::geometry::ExPolygon,
    fill: &SurfaceFill,
    layer_id: usize,
    reference: Point,
    scale: CoordinateScale,
) -> Result<Vec<crate::geometry::Polyline>, SliceError> {
    let params = LineFillParams {
        spacing: fill.params.spacing,
        overlap: fill.params.overlap,
        density: (0.01_f64 * f64::from(fill.params.density)) as f32,
        angle: fill.params.angle,
        layer_index: layer_id,
        thickness_layers: fill.representative.thickness_layers.max(1),
        fixed_angle: fill.params.fixed_angle,
        bridge_angle: fill.params.bridge.then_some(fill.params.bridge_angle),
        reference,
        dont_adjust: fill.params.bridge,
    };
    fill_line(expolygon, params, scale).map_err(geometry_error)
}

pub(super) fn append(
    output: &mut LayerFillEntities,
    fill: SurfaceFill,
    layer_id: usize,
    reference: Point,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    let no_overlap_expolygons = fill.no_overlap_expolygons.clone();
    let fill_params = fill.params;
    let fill_kind = fill.representative.kind;
    for expolygon in &fill.expolygons {
        let polylines = expolygon_polylines(expolygon, &fill, layer_id, reference, scale)?;
        if polylines.is_empty() {
            continue;
        }
        let flow = super::materialized_flow(fill_params, fill_params.spacing as f32)?;
        let mut entities: Vec<FillExtrusionEntity> = polylines
            .into_iter()
            .map(|polyline| {
                FillExtrusionEntity::Path(FillExtrusionPath {
                    polyline,
                    fitting: Vec::new(),
                    role: fill_params.extrusion_role,
                    mm3_per_mm: flow.mm3_per_mm,
                    width: flow.width,
                    height: flow.height,
                })
            })
            .collect();
        super::gap_residual::append_residual(super::gap_residual::ResidualInput {
            output_entities: &mut entities,
            no_overlap_expolygons: &no_overlap_expolygons,
            params: fill_params,
            kind: fill_kind,
            expolygon,
            scale,
        })?;
        output.collections.push(FillExtrusionCollection {
            entities,
            no_sort: false,
        });
    }
    let _ = ProcessInfillPattern::Line;
    Ok(())
}
