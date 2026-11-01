use crate::project_slice::perimeters::classic::materialize::Polyline3;
use crate::{
    ProcessInfillPattern, SliceError,
    fill::line::{LineFillParams, fill_line},
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
    call: &super::types::SurfaceCall,
) -> Result<Vec<crate::geometry::Polyline>, SliceError> {
    let params = LineFillParams {
        dont_alternate_fill_direction: call.dont_alternate_fill_direction,
        spacing: fill.params.spacing,
        overlap: fill.params.overlap,
        density: (0.01_f64 * f64::from(fill.params.density)) as f32,
        angle: fill.params.angle,
        layer_index: call.layer_id,
        thickness_layers: fill.representative.thickness_layers.max(1),
        fixed_angle: fill.params.fixed_angle,
        bridge_angle: fill.params.bridge.then_some(fill.params.bridge_angle),
        reference: call.reference,
        dont_adjust: call.dont_adjust,
    };
    fill_line(expolygon, params, call.scale).map_err(geometry_error)
}

pub(super) fn append(
    output: &mut LayerFillEntities,
    fill: SurfaceFill,
    call: super::types::SurfaceCall,
) -> Result<(), SliceError> {
    let no_overlap_expolygons = fill.no_overlap_expolygons.clone();
    let fill_params = fill.params;
    let fill_kind = fill.representative.kind;
    for expolygon in &fill.expolygons {
        let polylines = expolygon_polylines(expolygon, &fill, &call)?;
        if polylines.is_empty() {
            continue;
        }
        let polylines = super::join_close_ends::join(
            polylines,
            fill_params.extrusion_role,
            fill_params.spacing,
            (false, true),
            call.scale,
        );
        let flow = super::materialized_flow(fill_params, fill_params.spacing as f32)?;
        let mut entities: Vec<FillExtrusionEntity> = polylines
            .into_iter()
            .map(|polyline| {
                FillExtrusionEntity::Path(FillExtrusionPath {
                    polyline: Polyline3::from_xy(polyline, Vec::new()),
                    z_contoured: false,
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
            scale: call.scale,
        })?;
        output.collections.push(FillExtrusionCollection {
            entities,
            no_sort: false,
        });
    }
    let _ = ProcessInfillPattern::Line;
    Ok(())
}
