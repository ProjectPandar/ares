use crate::{
    ExtrusionRole, ProcessInfillPattern,
    geometry::ExPolygon,
    project_slice::{perimeters::types::Flow, region_slices::RegionSurfaceKind},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::project_slice) enum SurfaceFillPattern {
    Configured(ProcessInfillPattern),
    ConcentricInternal,
}

#[derive(Clone, Copy, Debug)]
pub(in crate::project_slice) struct SurfaceFillParams {
    pub(in crate::project_slice) extruder: u32,
    pub(in crate::project_slice) pattern: SurfaceFillPattern,
    pub(in crate::project_slice) spacing: f64,
    pub(in crate::project_slice) overlap: f64,
    pub(in crate::project_slice) angle: f32,
    pub(in crate::project_slice) fixed_angle: bool,
    pub(in crate::project_slice) bridge: bool,
    pub(in crate::project_slice) bridge_angle: f32,
    pub(in crate::project_slice) density: f32,
    pub(in crate::project_slice) multiline: i32,
    pub(in crate::project_slice) anchor_length: f32,
    pub(in crate::project_slice) anchor_length_max: f32,
    pub(in crate::project_slice) flow: Flow,
    pub(in crate::project_slice) extrusion_role: ExtrusionRole,
    pub(in crate::project_slice) idx: usize,
    pub(in crate::project_slice) loop_clipping: i64,
    pub(in crate::project_slice) role_speed: f32,
    pub(in crate::project_slice) lateral_lattice_angle_1: f32,
    pub(in crate::project_slice) lateral_lattice_angle_2: f32,
    pub(in crate::project_slice) infill_lock_depth: f32,
    pub(in crate::project_slice) skin_infill_depth: f32,
    pub(in crate::project_slice) symmetric_infill_y_axis: bool,
    pub(in crate::project_slice) infill_overhang_angle: f32,
    pub(in crate::project_slice) gyroid_optimized: bool,
}

#[derive(Clone, Copy, Debug)]
pub(in crate::project_slice) struct RepresentativeSurface {
    pub(in crate::project_slice) kind: RegionSurfaceKind,
    pub(in crate::project_slice) thickness: f64,
    pub(in crate::project_slice) thickness_layers: u16,
    pub(in crate::project_slice) bridge_angle: f64,
    pub(in crate::project_slice) extra_perimeters: u16,
}

pub(in crate::project_slice) struct SurfaceFill {
    pub(in crate::project_slice) region_id: usize,
    pub(in crate::project_slice) representative: RepresentativeSurface,
    pub(in crate::project_slice) expolygons: Vec<ExPolygon>,
    pub(in crate::project_slice) params: SurfaceFillParams,
    pub(in crate::project_slice) region_id_group: Vec<usize>,
    pub(in crate::project_slice) no_overlap_expolygons: Vec<ExPolygon>,
}

pub(in crate::project_slice) struct LockDensityParam {
    pub(in crate::project_slice) density: f32,
    pub(in crate::project_slice) expolygons: Vec<ExPolygon>,
}

pub(in crate::project_slice) struct LockFlowParam {
    pub(in crate::project_slice) flow: Flow,
    pub(in crate::project_slice) expolygons: Vec<ExPolygon>,
}

#[derive(Default)]
pub(in crate::project_slice) struct LockRegionParam {
    pub(in crate::project_slice) skin_density_params: Vec<LockDensityParam>,
    pub(in crate::project_slice) skeleton_density_params: Vec<LockDensityParam>,
    pub(in crate::project_slice) skin_flow_params: Vec<LockFlowParam>,
    pub(in crate::project_slice) skeleton_flow_params: Vec<LockFlowParam>,
}

pub(in crate::project_slice) struct GroupedFills {
    pub(in crate::project_slice) surface_fills: Vec<SurfaceFill>,
    pub(in crate::project_slice) lock_region_param: LockRegionParam,
}

impl GroupedFills {
    pub(super) fn empty() -> Self {
        Self {
            surface_fills: Vec::new(),
            lock_region_param: LockRegionParam::default(),
        }
    }
}

impl RepresentativeSurface {
    pub(super) fn from_parts(
        kind: RegionSurfaceKind,
        thickness: f64,
        thickness_layers: u16,
        bridge_angle: f32,
        extra_perimeters: u16,
    ) -> Self {
        Self {
            kind,
            thickness,
            thickness_layers,
            bridge_angle: f64::from(bridge_angle),
            extra_perimeters,
        }
    }
}
