use crate::project_slice::perimeters::classic::materialize::Polyline3;
use crate::{
    ProcessInfillPattern, SliceError,
    fill::honeycomb::{HoneycombFillParams, fill_honeycomb},
    geometry::{CoordinateScale, JoinType, offset_expolygon},
    project_slice::group_fills::SurfaceFill,
};

use super::{
    FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath, LayerFillEntities,
    geometry_error,
};

/// Emitted sparse fill for `ProcessInfillPattern::Honeycomb` — wraps
/// `fill::honeycomb::fill_honeycomb` (port of `FillHoneycomb::_fill_surface_single`,
/// which already runs `chain_or_connect_infill`).
pub(super) fn expolygon_polylines(
    expolygon: &crate::geometry::ExPolygon,
    delta: f32,
    params: HoneycombFillParams,
    scale: CoordinateScale,
) -> Result<Vec<Vec<crate::geometry::Polyline>>, SliceError> {
    // `Fill::fill_surface` offsets the surface by `overlap - spacing/2`
    // (`FillBase.cpp:108`) before `_fill_surface_single`; each resulting
    // component is filled independently.
    let components =
        offset_expolygon(expolygon, delta, JoinType::Miter, 3.0).map_err(geometry_error)?;
    let mut per_component = Vec::with_capacity(components.len());
    for component in &components {
        per_component.push(fill_honeycomb(component, params, scale).map_err(geometry_error)?);
    }
    Ok(per_component)
}

pub(super) fn polylines_params(
    fill: &SurfaceFill,
    call: &super::types::SurfaceCall,
) -> HoneycombFillParams {
    HoneycombFillParams {
        dont_alternate_fill_direction: call.dont_alternate_fill_direction,
        spacing: fill.params.spacing,
        density: (0.01_f64 * f64::from(fill.params.density)) as f32,
        multiline: fill.params.multiline,
        angle: fill.params.angle,
        layer_index: call.layer_id,
        thickness_layers: fill.representative.thickness_layers.max(1),
        fixed_angle: fill.params.fixed_angle,
        bridge_angle: fill.params.bridge.then_some(fill.params.bridge_angle),
        anchor_length: fill.params.anchor_length,
        anchor_length_max: fill.params.anchor_length_max,
    }
}

pub(super) fn append(
    output: &mut LayerFillEntities,
    fill: SurfaceFill,
    call: super::types::SurfaceCall,
) -> Result<(), SliceError> {
    let params = polylines_params(&fill, &call);
    let scale = call.scale;
    let no_overlap_expolygons = fill.no_overlap_expolygons.clone();
    let fill_params = fill.params;
    let fill_kind = fill.representative.kind;
    // `Fill::fill_surface` offsets the surface by `overlap - spacing/2`
    // (`FillBase.cpp:108`) before `_fill_surface_single`.
    let delta = ((fill_params.overlap - 0.5 * fill_params.spacing) / scale.factor()) as f32;
    for expolygon in fill.expolygons {
        for polylines in expolygon_polylines(&expolygon, delta, params, scale)? {
            if polylines.is_empty() {
                continue;
            }
            let polylines = super::join_close_ends::join(
                polylines,
                fill_params.extrusion_role,
                fill_params.spacing,
                (false, true),
                scale,
            );
            let flow = super::materialized_flow(fill_params, fill_params.spacing as f32)?;
            let mut entities: Vec<FillExtrusionEntity> = polylines
                .iter()
                .cloned()
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
            // `Fill.cpp:1328` intersects `no_overlap_expolygons` with the
            // ORIGINAL surface expolygon (before the `fill_surface` offset),
            // so the residual domain uses the un-offset surface, not `component`.
            super::gap_residual::append_residual(super::gap_residual::ResidualInput {
                output_entities: &mut entities,
                no_sort: false,
                no_overlap_expolygons: &no_overlap_expolygons,
                params: fill_params,
                kind: fill_kind,
                expolygon: &expolygon,
                scale,
            })?;
            output.collections.push(FillExtrusionCollection {
                entities,
                no_sort: false,
            });
        }
    }
    let _ = ProcessInfillPattern::Honeycomb;
    Ok(())
}
