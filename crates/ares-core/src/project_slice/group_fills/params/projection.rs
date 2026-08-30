mod rotation;

use rotation::projected_angle;
pub(in crate::project_slice) use rotation::simple_rotation_angle;

use crate::{
    ExtrusionRole, FloatOrPercent, ProcessInfillPattern, RegionOptions, SliceError,
    geometry::CoordinateScale,
    project_slice::{
        perimeters::{
            flow::{
                FillFlowContext, FillFlowRole, resolve_fill_bridge_flow, resolve_fill_flow,
                resolve_nominal_sparse_infill_flow,
            },
            types::Flow,
        },
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

use super::super::{SurfaceFillParams, SurfaceFillPattern};
use super::LayerContext;

pub(super) fn project_surface(
    context: &LayerContext<'_>,
    surface: &RegionSurface,
    params: &mut SurfaceFillParams,
) -> Result<Option<SurfaceFillParams>, SliceError> {
    let (kind, _, thickness, _, bridge_angle, _) = surface.as_parts();
    let solid = is_solid(kind);
    let is_bridge = context.planned.id > 0 && kind.is_bridge();
    let flow_role = flow_role(kind, solid);

    params.extruder = flow_role.selector(context.region).0 as u32;
    params.pattern = configured(context.region.sparse_infill_pattern);
    params.density = context.region.sparse_infill_density.0 as f32;
    params.lateral_lattice_angle_1 = context.region.lateral_lattice_angle_1.0 as f32;
    params.lateral_lattice_angle_2 = context.region.lateral_lattice_angle_2.0 as f32;
    params.infill_overhang_angle = context.region.infill_overhang_angle.0 as f32;
    apply_sticky_pattern_fields(params, context.region, context.scale);

    match kind {
        RegionSurfaceKind::Top => {
            params.pattern = configured(context.region.top_surface_pattern);
            params.density = context.region.top_surface_density.0 as f32;
            if params.density <= 0.0 {
                return Ok(None);
            }
        }
        RegionSurfaceKind::Bottom => {
            params.pattern = configured(context.region.bottom_surface_pattern);
            params.density = context.region.bottom_surface_density.0 as f32;
        }
        RegionSurfaceKind::BottomBridge if !is_bridge => {
            params.pattern = configured(context.region.bottom_surface_pattern);
            params.density = context.region.bottom_surface_density.0 as f32;
        }
        RegionSurfaceKind::InternalSolid => {
            params.pattern = configured(context.region.internal_solid_infill_pattern);
            params.density = 100.0;
        }
        RegionSurfaceKind::BottomBridge | RegionSurfaceKind::InternalBridge => {
            params.pattern = configured(bridge_pattern(context.region.top_surface_pattern));
            params.density = 100.0;
        }
        RegionSurfaceKind::Internal => {
            if params.density <= 0.0 {
                return Ok(None);
            }
        }
        RegionSurfaceKind::InternalVoid => {
            unreachable!("InternalVoid is observed before surface parameter projection")
        }
    }

    if kind == RegionSurfaceKind::InternalSolid {
        let layers = context.object.elefant_foot_compensation_layers.0.max(0) as usize;
        let density = context.object.elefant_foot_layers_density.0 / 100.0;
        if context.planned.id > 0 && context.planned.id <= layers && density != 1.0 {
            let remaining = layers - (context.planned.id - 1);
            params.density =
                (100.0 * (1.0 - (1.0 - density) * remaining as f64 / layers as f64)) as f32;
        }
    }
    params.extrusion_role = extrusion_role(kind, solid, is_bridge);
    params.extruder = output_selector(context.region, params.extrusion_role, params.extruder);
    params.multiline = if params.extrusion_role == ExtrusionRole::InternalInfill {
        context.region.fill_multiline.0
    } else {
        1
    };
    params.gyroid_optimized = matches!(
        params.pattern,
        SurfaceFillPattern::Configured(ProcessInfillPattern::Gyroid)
    ) && context.region.gyroid_optimized.0;
    let (angle, fixed_angle) = projected_angle(context, params.extrusion_role)?;
    params.angle = angle;
    params.fixed_angle = fixed_angle
        || matches!(
            params.pattern,
            SurfaceFillPattern::Configured(ProcessInfillPattern::AlignedRectilinear)
        );
    params.bridge_angle = bridge_angle as f32;
    params.bridge = is_bridge;

    let actual_height = if thickness == -1.0 {
        context.planned.height
    } else {
        thickness
    };
    let flow_context = FillFlowContext::new(
        context.planned,
        actual_height,
        context.initial_layer_width,
        context.region,
        context.object,
        context.nozzles,
    );
    let thick_bridge = kind.is_bridge()
        && if kind == RegionSurfaceKind::InternalBridge {
            context.object.thick_internal_bridges.0
        } else {
            context.object.thick_bridges.0
        };
    params.flow = if params.bridge {
        resolve_fill_bridge_flow(flow_context, flow_role, thick_bridge)?
    } else {
        resolve_fill_flow(flow_context, flow_role)?
    };
    params.flow_ratio = role_flow_ratio(context, params.extrusion_role);
    let seam_gap = match context.region.seam_gap {
        FloatOrPercent::Float(value) => value,
        FloatOrPercent::Percent(value) => f64::from(params.flow.nozzle_diameter) * value.0 / 100.0,
    };
    params.loop_clipping = context
        .scale
        .checked_scale(seam_gap)
        .ok_or_else(|| SliceError::InvalidInput("seam_gap is out of coordinate range".into()))?;
    params.role_speed = role_speed(context.region, params.extrusion_role);
    params.filter_out_gap_fill = context.region.filter_out_gap_fill.0;
    params.gap_fill_target = context.object.gap_fill_target;

    if solid || is_bridge {
        params.spacing = f64::from(params.flow.spacing);
        params.anchor_length = 1000.0;
        params.anchor_length_max = 1000.0;
    } else {
        params.spacing = f64::from(
            resolve_nominal_sparse_infill_flow(context.region, context.object, context.nozzles)?
                .spacing,
        );
        (params.anchor_length, params.anchor_length_max) =
            projected_anchor_lengths(context.region, params.spacing);
    }
    Ok(Some(*params))
}

pub(super) fn source_defaults() -> SurfaceFillParams {
    SurfaceFillParams {
        idx: 0,
        extruder: 0,
        pattern: configured(ProcessInfillPattern::Monotonic),
        spacing: 0.0,
        overlap: 0.0,
        angle: 0.0,
        fixed_angle: false,
        bridge: false,
        bridge_angle: 0.0,
        density: 0.0,
        multiline: 1,
        anchor_length: 1000.0,
        anchor_length_max: 1000.0,
        flow: Flow {
            width: 0.0,
            height: 0.0,
            spacing: 0.0,
            nozzle_diameter: 0.0,
            bridge: false,
            mm3_per_mm: 0.0,
        },
        flow_ratio: 1.0,
        extrusion_role: ExtrusionRole::None,
        loop_clipping: 0,
        role_speed: 0.0,
        lateral_lattice_angle_1: 0.0,
        lateral_lattice_angle_2: 0.0,
        infill_lock_depth: 0.0,
        skin_infill_depth: 0.0,
        symmetric_infill_y_axis: false,
        infill_overhang_angle: 60.0,
        gyroid_optimized: false,
        filter_out_gap_fill: 0.0,
        gap_fill_target: crate::ProcessGapFillTarget::Nowhere,
    }
}

pub(super) const fn configured(pattern: ProcessInfillPattern) -> SurfaceFillPattern {
    SurfaceFillPattern::Configured(pattern)
}

pub(super) fn flow_role(kind: RegionSurfaceKind, solid: bool) -> FillFlowRole {
    if kind == RegionSurfaceKind::Top {
        FillFlowRole::Top
    } else if solid {
        FillFlowRole::Solid
    } else {
        FillFlowRole::Infill
    }
}

fn role_flow_ratio(context: &LayerContext<'_>, role: ExtrusionRole) -> f64 {
    let region = context.region;
    let mut ratio = match role {
        ExtrusionRole::TopSolidInfill => region.top_solid_infill_flow_ratio.0,
        ExtrusionRole::BottomSurface => region.bottom_solid_infill_flow_ratio.0,
        ExtrusionRole::InternalBridgeInfill => region.internal_bridge_flow.0,
        _ => 1.0,
    };
    if context.object.set_other_flow_ratios.0 {
        ratio *= match role {
            ExtrusionRole::InternalInfill => region.sparse_infill_flow_ratio.0,
            ExtrusionRole::SolidInfill => region.internal_solid_infill_flow_ratio.0,
            _ => 1.0,
        };
        if context.planned.id == 0 {
            ratio *= region.first_layer_flow_ratio.0;
        }
    }
    ratio
}

fn output_selector(region: &RegionOptions, role: ExtrusionRole, current: u32) -> u32 {
    match role {
        ExtrusionRole::TopSolidInfill => region.top_surface_filament_id.0 as u32,
        ExtrusionRole::BottomSurface => region.bottom_surface_filament_id.0 as u32,
        ExtrusionRole::SolidInfill => region.internal_solid_filament_id.0 as u32,
        ExtrusionRole::None
        | ExtrusionRole::Perimeter
        | ExtrusionRole::ExternalPerimeter
        | ExtrusionRole::OverhangPerimeter
        | ExtrusionRole::InternalInfill
        | ExtrusionRole::Ironing
        | ExtrusionRole::BridgeInfill
        | ExtrusionRole::InternalBridgeInfill
        | ExtrusionRole::GapFill
        | ExtrusionRole::Skirt
        | ExtrusionRole::Brim
        | ExtrusionRole::SupportMaterial
        | ExtrusionRole::SupportMaterialInterface
        | ExtrusionRole::SupportTransition
        | ExtrusionRole::WipeTower
        | ExtrusionRole::Custom
        | ExtrusionRole::Mixed => current,
    }
}

fn extrusion_role(kind: RegionSurfaceKind, solid: bool, bridge: bool) -> ExtrusionRole {
    if bridge {
        if kind == RegionSurfaceKind::InternalBridge {
            ExtrusionRole::InternalBridgeInfill
        } else {
            ExtrusionRole::BridgeInfill
        }
    } else if solid {
        match kind {
            RegionSurfaceKind::Top => ExtrusionRole::TopSolidInfill,
            RegionSurfaceKind::Bottom | RegionSurfaceKind::BottomBridge => {
                ExtrusionRole::BottomSurface
            }
            RegionSurfaceKind::InternalSolid | RegionSurfaceKind::InternalBridge => {
                ExtrusionRole::SolidInfill
            }
            RegionSurfaceKind::Internal | RegionSurfaceKind::InternalVoid => {
                unreachable!("only solid surface kinds reach solid fill-role selection")
            }
        }
    } else {
        ExtrusionRole::InternalInfill
    }
}

pub(super) const fn is_solid(kind: RegionSurfaceKind) -> bool {
    match kind {
        RegionSurfaceKind::Top
        | RegionSurfaceKind::Bottom
        | RegionSurfaceKind::BottomBridge
        | RegionSurfaceKind::InternalSolid
        | RegionSurfaceKind::InternalBridge => true,
        RegionSurfaceKind::Internal | RegionSurfaceKind::InternalVoid => false,
    }
}

fn bridge_pattern(top: ProcessInfillPattern) -> ProcessInfillPattern {
    match top {
        ProcessInfillPattern::Monotonic | ProcessInfillPattern::MonotonicLine => {
            ProcessInfillPattern::Monotonic
        }
        ProcessInfillPattern::Rectilinear
        | ProcessInfillPattern::AlignedRectilinear
        | ProcessInfillPattern::ZigZag
        | ProcessInfillPattern::CrossZag
        | ProcessInfillPattern::LockedZag
        | ProcessInfillPattern::Line
        | ProcessInfillPattern::Grid
        | ProcessInfillPattern::Triangles
        | ProcessInfillPattern::TriHexagon
        | ProcessInfillPattern::Cubic
        | ProcessInfillPattern::AdaptiveCubic
        | ProcessInfillPattern::QuarterCubic
        | ProcessInfillPattern::SupportCubic
        | ProcessInfillPattern::Lightning
        | ProcessInfillPattern::Honeycomb
        | ProcessInfillPattern::ThreeDHoneycomb
        | ProcessInfillPattern::LateralHoneycomb
        | ProcessInfillPattern::LateralLattice
        | ProcessInfillPattern::CrossHatch
        | ProcessInfillPattern::TpmsD
        | ProcessInfillPattern::TpmsFk
        | ProcessInfillPattern::Gyroid
        | ProcessInfillPattern::Concentric
        | ProcessInfillPattern::HilbertCurve
        | ProcessInfillPattern::ArchimedeanChords
        | ProcessInfillPattern::OctagramSpiral => ProcessInfillPattern::Rectilinear,
    }
}

fn role_speed(region: &RegionOptions, role: ExtrusionRole) -> f32 {
    match role {
        ExtrusionRole::BridgeInfill => region.bridge_speed.0 as f32,
        ExtrusionRole::InternalBridgeInfill => {
            absolute_speed(region.internal_bridge_speed, region.bridge_speed.0) as f32
        }
        ExtrusionRole::InternalInfill => region.sparse_infill_speed.0 as f32,
        ExtrusionRole::TopSolidInfill => region.top_surface_speed.0 as f32,
        ExtrusionRole::SolidInfill => region.internal_solid_infill_speed.0 as f32,
        ExtrusionRole::None
        | ExtrusionRole::Perimeter
        | ExtrusionRole::ExternalPerimeter
        | ExtrusionRole::OverhangPerimeter
        | ExtrusionRole::BottomSurface
        | ExtrusionRole::Ironing
        | ExtrusionRole::GapFill
        | ExtrusionRole::Skirt
        | ExtrusionRole::Brim
        | ExtrusionRole::SupportMaterial
        | ExtrusionRole::SupportMaterialInterface
        | ExtrusionRole::SupportTransition
        | ExtrusionRole::WipeTower
        | ExtrusionRole::Custom
        | ExtrusionRole::Mixed => 0.0,
    }
}

fn absolute_speed(value: FloatOrPercent, base: f64) -> f64 {
    match value {
        FloatOrPercent::Float(value) => value,
        FloatOrPercent::Percent(value) => base * value.0 / 100.0,
    }
}

fn apply_sticky_pattern_fields(
    params: &mut SurfaceFillParams,
    region: &RegionOptions,
    scale: CoordinateScale,
) {
    if region.sparse_infill_pattern == ProcessInfillPattern::LockedZag {
        params.infill_lock_depth = (region.infill_lock_depth.0 / scale.factor()) as f32;
        params.skin_infill_depth = (region.skin_infill_depth.0 / scale.factor()) as f32;
    }
    if matches!(
        region.sparse_infill_pattern,
        ProcessInfillPattern::CrossZag
            | ProcessInfillPattern::LockedZag
            | ProcessInfillPattern::ZigZag
    ) {
        params.symmetric_infill_y_axis = region.symmetric_infill_y_axis.0;
    }
}

fn projected_anchor_lengths(options: &RegionOptions, spacing: f64) -> (f32, f32) {
    let anchor_length = projected_length(options.infill_anchor, spacing);
    let anchor_length_max = projected_length(options.infill_anchor_max, spacing);
    let anchor_length = if anchor_length_max < anchor_length {
        anchor_length_max
    } else {
        anchor_length
    };
    (anchor_length, anchor_length_max)
}

fn projected_length(value: FloatOrPercent, spacing: f64) -> f32 {
    match value {
        FloatOrPercent::Float(value) => value as f32,
        FloatOrPercent::Percent(value) => (f64::from(value.0 as f32) * 0.01 * spacing) as f32,
    }
}
