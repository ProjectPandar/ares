use crate::geometry::ExPolygon;

#[derive(Clone, Copy)]
pub(crate) enum OracleStage {
    PostNarrow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OracleTotals {
    pub(super) layers: usize,
    pub(super) groups: usize,
    pub(super) fill_expolygons: usize,
    pub(super) fill_holes: usize,
    pub(super) fill_paths: usize,
    pub(super) fill_points: usize,
    pub(super) no_overlap_expolygons: usize,
    pub(super) nonempty_layers: usize,
    pub(super) empty_layers: usize,
}

#[derive(Clone, Copy)]
pub(crate) struct OracleFlow {
    pub(super) width: f32,
    pub(super) height: f32,
    pub(super) spacing: f32,
    pub(super) nozzle_diameter: f32,
    pub(super) bridge: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct OracleParams {
    pub(super) idx: usize,
    pub(super) extruder: u32,
    pub(super) pattern: u8,
    pub(super) spacing: f64,
    pub(super) overlap: f64,
    pub(super) angle: f32,
    pub(super) fixed_angle: bool,
    pub(super) bridge: bool,
    pub(super) bridge_angle: f32,
    pub(super) density: f32,
    pub(super) multiline: i32,
    pub(super) anchor_length: f32,
    pub(super) anchor_length_max: f32,
    pub(super) flow: OracleFlow,
    pub(super) extrusion_role: u8,
    pub(super) role_speed: f32,
    pub(super) lateral_lattice_angle_1: f32,
    pub(super) lateral_lattice_angle_2: f32,
    pub(super) infill_lock_depth: f32,
    pub(super) skin_infill_depth: f32,
    pub(super) symmetric_infill_y_axis: bool,
    pub(super) infill_overhang_angle: f32,
    pub(super) gyroid_optimized: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct OracleRepresentative {
    pub(super) kind: u8,
    pub(super) thickness: f64,
    pub(super) thickness_layers: u16,
    pub(super) bridge_angle: f64,
    pub(super) extra_perimeters: u16,
}

pub(crate) struct OracleGroup<'a> {
    pub(super) region_id: usize,
    pub(super) representative: OracleRepresentative,
    pub(super) params: OracleParams,
    pub(super) region_id_group: &'a [usize],
    pub(super) fills: &'a [ExPolygon],
    pub(super) no_overlap: &'a [ExPolygon],
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct OracleLockCounts {
    pub(super) skin_density: usize,
    pub(super) skeleton_density: usize,
    pub(super) skin_flow: usize,
    pub(super) skeleton_flow: usize,
}

pub(crate) struct OracleLayer<'a> {
    pub(super) stage: OracleStage,
    pub(super) layer_id: usize,
    pub(super) layer_height: f64,
    pub(super) print_z: f64,
    pub(super) lock_counts: OracleLockCounts,
    pub(super) groups: Vec<OracleGroup<'a>>,
}

pub(crate) struct EncodedOracle {
    pub(super) metadata: Vec<u8>,
    pub(super) canonical_geometry: Vec<u8>,
    pub(super) layer_table: Vec<u8>,
}
