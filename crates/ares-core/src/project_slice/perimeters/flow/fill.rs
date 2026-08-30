use crate::{
    FloatOrPercent, ObjectOptions, OrcaFloats, OrcaInt, RegionOptions, SliceError,
    project_slice::{layers::PlannedLayer, perimeters::types::Flow},
};

use super::{
    absolute_f64, build_nonbridging_flow, build_nonbridging_from_width, raw,
    require_positive_volume, resolve_thick_bridge_flow, select_width, selected_nozzle,
    with_flow_ratio,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) enum FillFlowRole {
    Infill,
    Solid,
    Top,
}

impl FillFlowRole {
    pub(in crate::project_slice) const fn selector(self, region: &RegionOptions) -> OrcaInt {
        match self {
            Self::Infill => region.sparse_infill_filament_id,
            Self::Solid => region.internal_solid_filament_id,
            Self::Top => region.top_surface_filament_id,
        }
    }
}

#[derive(Clone, Copy)]
pub(in crate::project_slice) struct FillFlowContext<'a> {
    layer: &'a PlannedLayer,
    actual_height: f64,
    initial_layer_width: FloatOrPercent,
    region: &'a RegionOptions,
    object: &'a ObjectOptions,
    nozzle_diameters: &'a OrcaFloats,
}

impl<'a> FillFlowContext<'a> {
    #[expect(
        clippy::too_many_arguments,
        reason = "the source fill Flow context has six independent inputs"
    )]
    pub(in crate::project_slice) const fn new(
        layer: &'a PlannedLayer,
        actual_height: f64,
        initial_layer_width: FloatOrPercent,
        region: &'a RegionOptions,
        object: &'a ObjectOptions,
        nozzle_diameters: &'a OrcaFloats,
    ) -> Self {
        Self {
            layer,
            actual_height,
            initial_layer_width,
            region,
            object,
            nozzle_diameters,
        }
    }
}

pub(in crate::project_slice) fn resolve_fill_flow(
    context: FillFlowContext<'_>,
    role: FillFlowRole,
) -> Result<Flow, SliceError> {
    let width = select_width(
        context.layer,
        context.initial_layer_width,
        role_width(role, context.region),
        context.object.line_width,
    );
    resolve_with_width(context, role, width, context.actual_height)
}

pub(in crate::project_slice) fn resolve_fill_bridge_flow(
    context: FillFlowContext<'_>,
    role: FillFlowRole,
    thick_bridge: bool,
) -> Result<Flow, SliceError> {
    if thick_bridge {
        return resolve_thick_bridge_flow(
            role.selector(context.region),
            context.region,
            context.nozzle_diameters,
        );
    }

    let nozzle_diameter = selected_nozzle(role.selector(context.region), context.nozzle_diameters)?;
    let width = select_width(
        context.layer,
        context.initial_layer_width,
        role_width(role, context.region),
        context.object.line_width,
    );
    let mut flow = build_role_flow(
        role,
        width,
        checked_height(context.layer.height)?,
        nozzle_diameter,
    )?;
    let bridge_width = absolute_f64(context.region.bridge_line_width, nozzle_diameter);
    if bridge_width > 0.0 {
        flow = build_nonbridging_from_width(bridge_width as f32, flow.height, nozzle_diameter)?;
    }
    require_positive_volume(
        with_flow_ratio(flow, context.region.bridge_flow.0),
        "invalid Orca option bridge_flow",
    )
}

pub(in crate::project_slice) fn resolve_configured_fill_flow(
    context: FillFlowContext<'_>,
    role: FillFlowRole,
    configured_width: FloatOrPercent,
) -> Result<Flow, SliceError> {
    resolve_with_width(context, role, configured_width, context.actual_height)
}

pub(in crate::project_slice) fn resolve_nominal_sparse_infill_flow(
    region: &RegionOptions,
    object: &ObjectOptions,
    nozzle_diameters: &OrcaFloats,
) -> Result<Flow, SliceError> {
    let nozzle_diameter = selected_nozzle(region.sparse_infill_filament_id, nozzle_diameters)?;
    let width = if raw(region.sparse_infill_line_width) == 0.0 {
        object.line_width
    } else {
        region.sparse_infill_line_width
    };
    build_role_flow(
        FillFlowRole::Infill,
        width,
        checked_height(object.layer_height.0)?,
        nozzle_diameter,
    )
}

pub(in crate::project_slice) fn resolve_thick_solid_infill_bridge_flow(
    region: &RegionOptions,
    nozzle_diameters: &OrcaFloats,
) -> Result<Flow, SliceError> {
    resolve_thick_bridge_flow(
        FillFlowRole::Solid.selector(region),
        region,
        nozzle_diameters,
    )
}

fn resolve_with_width(
    context: FillFlowContext<'_>,
    role: FillFlowRole,
    width: FloatOrPercent,
    height: f64,
) -> Result<Flow, SliceError> {
    let nozzle_diameter = selected_nozzle(role.selector(context.region), context.nozzle_diameters)?;
    build_role_flow(role, width, checked_height(height)?, nozzle_diameter)
}

fn build_role_flow(
    role: FillFlowRole,
    width: FloatOrPercent,
    height: f32,
    nozzle_diameter: f32,
) -> Result<Flow, SliceError> {
    if role == FillFlowRole::Top && width.is_non_positive() {
        build_nonbridging_from_width(nozzle_diameter, height, nozzle_diameter)
    } else {
        build_nonbridging_flow(width, height, nozzle_diameter)
    }
}

const fn role_width(role: FillFlowRole, region: &RegionOptions) -> FloatOrPercent {
    match role {
        FillFlowRole::Infill => region.sparse_infill_line_width,
        FillFlowRole::Solid => region.internal_solid_infill_line_width,
        FillFlowRole::Top => region.top_surface_line_width,
    }
}

fn checked_height(height: f64) -> Result<f32, SliceError> {
    let height = height as f32;
    if !height.is_finite() || height <= 0.0 {
        return Err(SliceError::InvalidInput(
            "invalid Orca option layer_height".to_owned(),
        ));
    }
    Ok(height)
}
