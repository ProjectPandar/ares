use crate::{FloatOrPercent, ObjectOptions, OrcaFloats, OrcaInt, RegionOptions, SliceError};

use super::super::layers::PlannedLayer;
use super::types::{Flow, PerimeterFlows};

mod fill;

pub(in crate::project_slice) use fill::{
    FillFlowContext, FillFlowRole, resolve_configured_fill_flow, resolve_fill_bridge_flow,
    resolve_fill_flow, resolve_nominal_sparse_infill_flow, resolve_thick_solid_infill_bridge_flow,
};

const ROUNDED_RECTANGLE_FACTOR: f64 = 1.0 - 0.25 * std::f64::consts::PI;
const FLOW_EPSILON: f64 = 1e-4;

#[derive(Clone, Copy)]
enum PerimeterFlowRole {
    Internal,
    External,
    SolidInfill,
}

#[derive(Clone, Copy)]
struct FlowContext<'a> {
    layer: &'a PlannedLayer,
    initial_layer_width: FloatOrPercent,
    region: &'a RegionOptions,
    object: &'a ObjectOptions,
    nozzle_diameters: &'a OrcaFloats,
}

#[allow(clippy::too_many_arguments)]
pub(in crate::project_slice) fn resolve_external_perimeter_flow(
    layer: &PlannedLayer,
    initial_layer_width: FloatOrPercent,
    outer_wall_width: FloatOrPercent,
    object_line_width: FloatOrPercent,
    outer_wall_filament_id: OrcaInt,
    nozzle_diameters: &OrcaFloats,
) -> Result<Flow, SliceError> {
    let nozzle_diameter = selected_nozzle(outer_wall_filament_id, nozzle_diameters)?;
    let height = selected_height(layer)?;
    let selected_width = select_width(
        layer,
        initial_layer_width,
        outer_wall_width,
        object_line_width,
    );
    build_nonbridging_flow(selected_width, height, nozzle_diameter)
}

pub(in crate::project_slice) fn resolve_perimeter_flows(
    layer: &PlannedLayer,
    initial_layer_width: FloatOrPercent,
    region: &RegionOptions,
    object: &ObjectOptions,
    nozzle_diameters: &OrcaFloats,
) -> Result<PerimeterFlows, SliceError> {
    let context = FlowContext {
        layer,
        initial_layer_width,
        region,
        object,
        nozzle_diameters,
    };
    let perimeter_flow = resolve_role_flow(context, PerimeterFlowRole::Internal)?;
    let ext_perimeter_flow = resolve_role_flow(context, PerimeterFlowRole::External)?;
    let overhang_flow = resolve_overhang_flow(context)?;
    let solid_infill_flow = resolve_role_flow(context, PerimeterFlowRole::SolidInfill)?;
    Ok(PerimeterFlows {
        perimeter_flow,
        ext_perimeter_flow,
        overhang_flow,
        solid_infill_flow,
    })
}

fn resolve_role_flow(
    context: FlowContext<'_>,
    role: PerimeterFlowRole,
) -> Result<Flow, SliceError> {
    let nozzle_diameter = selected_nozzle(
        role_selector(role, context.region),
        context.nozzle_diameters,
    )?;
    let selected_width = select_width(
        context.layer,
        context.initial_layer_width,
        role_width(role, context.region),
        context.object.line_width,
    );
    let height = selected_height(context.layer)?;
    let flow = build_nonbridging_flow(selected_width, height, nozzle_diameter)?;
    require_positive_volume(flow, "invalid external perimeter flow volume")
}

fn resolve_overhang_flow(context: FlowContext<'_>) -> Result<Flow, SliceError> {
    if context.object.thick_bridges.0 {
        return resolve_thick_bridge_flow(
            context.region.inner_wall_filament_id,
            context.region,
            context.nozzle_diameters,
        );
    }

    let nozzle_diameter = selected_nozzle(
        context.region.inner_wall_filament_id,
        context.nozzle_diameters,
    )?;
    let configured_width = absolute_f64(context.region.bridge_line_width, nozzle_diameter);
    let mut flow = resolve_role_flow(context, PerimeterFlowRole::Internal)?;
    if configured_width > 0.0 {
        flow = build_nonbridging_from_width(configured_width as f32, flow.height, nozzle_diameter)?;
    }
    require_positive_volume(
        with_flow_ratio(flow, context.region.bridge_flow.0),
        "invalid Orca option bridge_flow",
    )
}

fn resolve_thick_bridge_flow(
    selector: OrcaInt,
    region: &RegionOptions,
    nozzle_diameters: &OrcaFloats,
) -> Result<Flow, SliceError> {
    let nozzle_diameter = selected_nozzle(selector, nozzle_diameters)?;
    let configured_width = absolute_f64(region.bridge_line_width, nozzle_diameter);
    let mut diameter = if configured_width > 0.0 {
        configured_width as f32
    } else {
        nozzle_diameter
    };
    if region.bridge_flow.0 > 0.0 {
        diameter *= region.bridge_flow.0.sqrt() as f32;
    }
    require_positive_volume(
        Flow {
            width: diameter,
            height: diameter,
            spacing: (f64::from(diameter) + 0.05) as f32,
            nozzle_diameter,
            bridge: true,
            mm3_per_mm: bridge_volume(diameter),
        },
        "invalid Orca option bridge_flow",
    )
}

fn role_selector(role: PerimeterFlowRole, region: &RegionOptions) -> OrcaInt {
    match role {
        PerimeterFlowRole::Internal => region.inner_wall_filament_id,
        PerimeterFlowRole::External => region.outer_wall_filament_id,
        PerimeterFlowRole::SolidInfill => region.internal_solid_filament_id,
    }
}

fn role_width(role: PerimeterFlowRole, region: &RegionOptions) -> FloatOrPercent {
    match role {
        PerimeterFlowRole::Internal => region.inner_wall_line_width,
        PerimeterFlowRole::External => region.outer_wall_line_width,
        PerimeterFlowRole::SolidInfill => region.internal_solid_infill_line_width,
    }
}

fn selected_nozzle(selector: OrcaInt, nozzle_diameters: &OrcaFloats) -> Result<f32, SliceError> {
    let nozzle_index = selector
        .0
        .checked_sub(1)
        .and_then(|index| usize::try_from(index).ok())
        .filter(|index| *index < nozzle_diameters.0.len())
        .unwrap_or(0);
    nozzle_diameters
        .0
        .get(nozzle_index)
        .map(|diameter| diameter.0 as f32)
        .filter(|diameter| diameter.is_finite() && *diameter > 0.0)
        .ok_or_else(|| invalid("invalid Orca option nozzle_diameter"))
}

fn selected_height(layer: &PlannedLayer) -> Result<f32, SliceError> {
    let height = layer.height as f32;
    if !height.is_finite() || height <= 0.0 {
        return Err(invalid("invalid Orca option layer_height"));
    }
    Ok(height)
}

fn select_width(
    layer: &PlannedLayer,
    initial_layer_width: FloatOrPercent,
    role_width: FloatOrPercent,
    object_line_width: FloatOrPercent,
) -> FloatOrPercent {
    let mut selected = if layer.id == 0 && raw(initial_layer_width) > 0.0 {
        initial_layer_width
    } else {
        role_width
    };
    if raw(selected) == 0.0 {
        selected = object_line_width;
    }
    selected
}

pub(super) fn build_nonbridging_flow(
    selected_width: FloatOrPercent,
    height: f32,
    nozzle_diameter: f32,
) -> Result<Flow, SliceError> {
    let width = match selected_width {
        FloatOrPercent::Float(value) if value <= 0.0 => 1.125_f32 * nozzle_diameter,
        value => absolute(value, nozzle_diameter),
    };
    build_nonbridging_from_width(width, height, nozzle_diameter)
}

fn build_nonbridging_from_width(
    width: f32,
    height: f32,
    nozzle_diameter: f32,
) -> Result<Flow, SliceError> {
    let spacing = width - height * (ROUNDED_RECTANGLE_FACTOR as f32);
    if !spacing.is_finite() || spacing <= 0.0 {
        return Err(invalid("invalid external perimeter flow spacing"));
    }
    Ok(Flow {
        width,
        height,
        spacing,
        nozzle_diameter,
        bridge: false,
        mm3_per_mm: ordinary_volume(width, height),
    })
}

fn require_positive_volume(flow: Flow, message: &str) -> Result<Flow, SliceError> {
    if !flow.mm3_per_mm.is_finite() || flow.mm3_per_mm <= 0.0 {
        return Err(invalid(message));
    }
    Ok(flow)
}

pub(in crate::project_slice) fn with_spacing(flow: Flow, spacing: f32) -> Flow {
    let (width, height, mm3_per_mm) = if flow.bridge {
        let diameter = spacing - (flow.spacing - flow.width);
        (diameter, diameter, bridge_volume(diameter))
    } else {
        let width = flow.width + (spacing - flow.spacing);
        debug_assert!(width >= flow.height);
        (width, flow.height, ordinary_volume(width, flow.height))
    };
    Flow {
        width,
        height,
        spacing,
        nozzle_diameter: flow.nozzle_diameter,
        bridge: flow.bridge,
        mm3_per_mm,
    }
}

fn with_flow_ratio(flow: Flow, ratio: f64) -> Flow {
    with_cross_section(flow, (flow.mm3_per_mm * ratio) as f32)
}

fn with_cross_section(flow: Flow, area_new: f32) -> Flow {
    let area = flow.mm3_per_mm as f32;
    if f64::from(area_new) > f64::from(area) + FLOW_EPSILON {
        let new_full_spacing = area_new / flow.height;
        if new_full_spacing > flow.spacing {
            let height = area_new / flow.spacing;
            let width = rounded_width(flow.spacing, height);
            return Flow {
                width,
                height,
                spacing: flow.spacing,
                nozzle_diameter: flow.nozzle_diameter,
                bridge: false,
                mm3_per_mm: ordinary_volume(width, height),
            };
        }
        return build_nonbridging_from_width(
            rounded_width(area / flow.height, flow.height),
            flow.height,
            flow.nozzle_diameter,
        )
        .expect("trusted canonical Flow must preserve positive spacing");
    }
    if f64::from(area_new) < f64::from(area) - FLOW_EPSILON {
        let width = flow.width - (area - area_new) / flow.height;
        if width > flow.height {
            return build_nonbridging_from_width(width, flow.height, flow.nozzle_diameter)
                .expect("trusted cross-section width must preserve canonical spacing");
        }
        let diameter = (f64::from(area_new) / std::f64::consts::PI).sqrt() as f32;
        return Flow {
            width: diameter,
            height: diameter,
            spacing: flow.spacing,
            nozzle_diameter: flow.nozzle_diameter,
            bridge: false,
            mm3_per_mm: ordinary_volume(diameter, diameter),
        };
    }
    flow
}

fn rounded_width(spacing: f32, height: f32) -> f32 {
    (f64::from(spacing) + f64::from(height) * ROUNDED_RECTANGLE_FACTOR) as f32
}

fn ordinary_volume(width: f32, height: f32) -> f64 {
    f64::from(
        (f64::from(height) * (f64::from(width) - f64::from(height) * ROUNDED_RECTANGLE_FACTOR))
            as f32,
    )
}

fn bridge_volume(width: f32) -> f64 {
    f64::from((f64::from(width * width) * 0.25 * std::f64::consts::PI) as f32)
}

fn absolute(value: FloatOrPercent, nozzle_diameter: f32) -> f32 {
    match value {
        FloatOrPercent::Float(value) => value as f32,
        FloatOrPercent::Percent(percent) => (f64::from(nozzle_diameter) * percent.0 / 100.0) as f32,
    }
}

fn absolute_f64(value: FloatOrPercent, nozzle_diameter: f32) -> f64 {
    match value {
        FloatOrPercent::Float(value) => value,
        FloatOrPercent::Percent(percent) => f64::from(nozzle_diameter) * percent.0 / 100.0,
    }
}

fn raw(value: FloatOrPercent) -> f64 {
    match value {
        FloatOrPercent::Float(value) => value,
        FloatOrPercent::Percent(percent) => percent.0,
    }
}

fn invalid(message: &str) -> SliceError {
    SliceError::InvalidInput(message.to_owned())
}

type ResolveExternalPerimeterFlow = fn(
    &PlannedLayer,
    FloatOrPercent,
    FloatOrPercent,
    FloatOrPercent,
    OrcaInt,
    &OrcaFloats,
) -> Result<Flow, SliceError>;
const _: ResolveExternalPerimeterFlow = resolve_external_perimeter_flow;
