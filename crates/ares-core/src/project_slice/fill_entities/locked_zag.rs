use crate::project_slice::perimeters::classic::materialize::Polyline3;
use crate::{
    SliceError,
    fill::locked_zag::{self, LockDensityParam, LockFlowParam, LockRegionParam},
    fill::rectilinear::{MonotonicFillParams, fill_rectilinear_surface},
    geometry::Point,
    project_slice::group_fills::{LockRegionParam as GroupLock, SurfaceFill},
};

use super::{
    FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath, LayerFillEntities,
    geometry_error,
};

/// `FillLockedZag::fill_surface_extrusion` → `fill_surface_locked_zag`
/// (`FillRectilinear.cpp:3866`): emits the two-density skin/skeleton locked-zag
/// fill, each polyline carrying the `Flow` of the region it lands in.
pub(super) fn append(
    output: &mut LayerFillEntities,
    fill: SurfaceFill,
    lock: &GroupLock,
    call: super::types::SurfaceCall,
    super::types::ObjectFillGeometry {
        scale,
        extended_object_min_x,
        extended_object_max_x,
    }: super::types::ObjectFillGeometry,
) -> Result<(), SliceError> {
    let params = fill.params;
    let kind = fill.representative.kind;
    let no_overlap_expolygons = fill.no_overlap_expolygons.clone();
    let base = MonotonicFillParams {
        dont_alternate_fill_direction: call.dont_alternate_fill_direction,
        layer_angle_fixed: false,
        spacing: params.spacing,
        overlap: params.overlap,
        density: 0.01 * params.density,
        angle: params.angle,
        layer_index: call.layer_id,
        thickness_layers: fill.representative.thickness_layers.max(1),
        fixed_angle: params.fixed_angle,
        bridge_angle: params.bridge.then_some(params.bridge_angle),
        reference_point: Point::new(0, 0),
        dont_adjust: params.bridge,
        anchor_length_max: params.anchor_length_max,
        link_max_length: 0.0,
        consistent_pattern: true,
        horiz_move: params.horiz_move,
        extended_object_min_x: Some(extended_object_min_x),
        extended_object_max_x: Some(extended_object_max_x),
        locked_zag: false,
    };
    let lock_param = LockRegionParam {
        skin_density_params: convert_density(&lock.skin_density_params),
        skeleton_density_params: convert_density(&lock.skeleton_density_params),
        skin_flow_params: convert_flow(&lock.skin_flow_params),
        skeleton_flow_params: convert_flow(&lock.skeleton_flow_params),
    };
    for expolygon in fill.expolygons {
        let pairs = locked_zag::fill_surface_locked_zag(
            &expolygon,
            &base,
            &lock_param,
            locked_zag::LockedZagDepths {
                skin_infill: params.skin_infill_depth as i64,
                infill_lock: params.infill_lock_depth as i64,
            },
            |exp, density, horiz_move, locked_zag| {
                let mut sub = base;
                // The skin pass clears `params.locked_zag`
                // (`FillRectilinear.cpp:3919`); the skeleton keeps it so the
                // line grid rotates +90° (`:2763`).
                sub.density = density;
                sub.horiz_move = horiz_move;
                sub.locked_zag = locked_zag;
                fill_rectilinear_surface(exp, sub, scale).map(|out| out.polylines)
            },
        )
        .map_err(geometry_error)?;
        if pairs.is_empty() {
            continue;
        }
        let mut entities: Vec<FillExtrusionEntity> = pairs
            .into_iter()
            .map(|(polyline, flow)| {
                FillExtrusionEntity::Path(FillExtrusionPath {
                    polyline: Polyline3::from_xy(polyline, Vec::new()),
                    z_contoured: false,
                    role: params.extrusion_role,
                    mm3_per_mm: flow.mm3_per_mm,
                    width: flow.width,
                    height: flow.height,
                })
            })
            .collect();
        super::gap_residual::append_residual(super::gap_residual::ResidualInput {
            output_entities: &mut entities,
            no_sort: false,
            no_overlap_expolygons: &no_overlap_expolygons,
            params,
            kind,
            expolygon: &expolygon,
            scale,
        })?;
        output.collections.push(FillExtrusionCollection {
            entities,
            no_sort: false,
        });
    }
    Ok(())
}

fn convert_density(
    params: &[crate::project_slice::group_fills::LockDensityParam],
) -> Vec<LockDensityParam> {
    params
        .iter()
        .map(|p| LockDensityParam {
            density: p.density,
            expolygons: p.expolygons.clone(),
        })
        .collect()
}

fn convert_flow(
    params: &[crate::project_slice::group_fills::LockFlowParam],
) -> Vec<LockFlowParam<crate::project_slice::perimeters::types::Flow>> {
    params
        .iter()
        .map(|p| LockFlowParam {
            flow: p.flow,
            expolygons: p.expolygons.clone(),
        })
        .collect()
}
