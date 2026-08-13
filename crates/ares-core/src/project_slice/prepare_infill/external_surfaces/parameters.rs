use crate::geometry::CoordinateScale;

#[derive(Clone, Copy)]
pub(in crate::project_slice) struct ProcessExternalSurfacesConfig {
    pub(in crate::project_slice) wall_loops: i32,
    pub(in crate::project_slice) perimeter_spacing: i64,
    pub(in crate::project_slice) external_width: i64,
    pub(in crate::project_slice) external_spacing: i64,
    pub(in crate::project_slice) solid_infill_spacing: i64,
    pub(in crate::project_slice) bridge_angle_degrees: f64,
    pub(in crate::project_slice) relative_bridge_angle: bool,
    pub(in crate::project_slice) model_rotation_radians: f64,
    pub(in crate::project_slice) sparse_infill_density_percent: f64,
    pub(in crate::project_slice) minimum_sparse_infill_area_mm2: f64,
    pub(in crate::project_slice) spiral_mode: bool,
    pub(in crate::project_slice) scale: CoordinateScale,
}

pub(super) struct ExternalSurfaceParameters {
    pub(super) expansion_min: f32,
    pub(super) expansion_top: f32,
    pub(super) expansion_bottom: f32,
    pub(super) expansion_bottom_bridge: f32,
    pub(super) expansion_step: f32,
    pub(super) closing_radius: f32,
    pub(super) minimum_sparse_area: f64,
}

pub(super) fn derive(config: ProcessExternalSurfacesConfig) -> ExternalSurfaceParameters {
    let (shell_width, expansion_min) = if config.wall_loops > 0 {
        let shell_width = 0.5_f32 * config.external_width as f32
            + config.external_spacing as f32
            + (config.perimeter_spacing * i64::from(config.wall_loops - 1)) as f32;
        (shell_width, config.perimeter_spacing as f32)
    } else {
        let epsilon = (1e-4_f64 / config.scale.factor()) as f32;
        (epsilon, epsilon)
    };
    let expansion = (f64::from(shell_width) * 2.0_f64.sqrt()) as f32;
    ExternalSurfaceParameters {
        expansion_min,
        expansion_top: expansion,
        expansion_bottom: expansion,
        expansion_bottom_bridge: expansion,
        expansion_step: (0.1_f64 / config.scale.factor()) as f32,
        closing_radius: 0.55_f32 * 0.65_f32 * 1.05_f32 * config.solid_infill_spacing as f32,
        minimum_sparse_area: config.minimum_sparse_infill_area_mm2
            / config.scale.factor()
            / config.scale.factor(),
    }
}
