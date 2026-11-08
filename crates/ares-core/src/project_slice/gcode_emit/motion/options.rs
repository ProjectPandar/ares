pub(super) mod helpers;

use crate::{
    FloatOrPercent, RawOverhangFanThreshold, RetractLiftEnforce, options::InternalBridgeFanSpeed,
};

pub(in crate::project_slice::gcode_emit) use helpers::{first_nullable_float, nullable_float_at};

mod build;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(in crate::project_slice::gcode_emit) struct ScarfOptions {
    pub(in crate::project_slice::gcode_emit) seam_slope_type: crate::ProcessSeamScarfType,
    pub(in crate::project_slice::gcode_emit) conditional: bool,
    pub(in crate::project_slice::gcode_emit) angle_threshold: f64,
    pub(in crate::project_slice::gcode_emit) overhang_threshold: f64,
    pub(in crate::project_slice::gcode_emit) outer_line_width: f64,
    pub(in crate::project_slice::gcode_emit) inner_line_width: f64,
    pub(in crate::project_slice::gcode_emit) start_height: Option<FloatOrPercent>,
    pub(in crate::project_slice::gcode_emit) entire_loop: bool,
    pub(in crate::project_slice::gcode_emit) min_length: f64,
    pub(in crate::project_slice::gcode_emit) steps: usize,
    pub(in crate::project_slice::gcode_emit) inner_walls: bool,
    pub(in crate::project_slice::gcode_emit) speed: Option<FloatOrPercent>,
    pub(in crate::project_slice::gcode_emit) flow_ratio: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(in crate::project_slice::gcode_emit) struct MotionOptions {
    pub(in crate::project_slice::gcode_emit) adaptive_pa: super::super::adaptive_pa::Options,
    pub(in crate::project_slice::gcode_emit) filament_area: f64,
    pub(in crate::project_slice::gcode_emit) nozzle_config_index: usize,
    /// `filament_density` g/cc for the `extruded_weight`
    /// placeholders (`Extruder.cpp:133-135`).
    pub(in crate::project_slice::gcode_emit) filament_density: f64,
    pub(in crate::project_slice::gcode_emit) filament_flow_ratio: f64,
    pub(in crate::project_slice::gcode_emit) print_flow_ratio: f64,
    pub(in crate::project_slice::gcode_emit) set_other_flow_ratios: bool,
    pub(in crate::project_slice::gcode_emit) outer_wall_flow_ratio: f64,
    pub(in crate::project_slice::gcode_emit) inner_wall_flow_ratio: f64,
    pub(in crate::project_slice::gcode_emit) overhang_flow_ratio: f64,
    pub(in crate::project_slice::gcode_emit) sparse_infill_flow_ratio: f64,
    pub(in crate::project_slice::gcode_emit) internal_solid_infill_flow_ratio: f64,
    pub(in crate::project_slice::gcode_emit) gap_fill_flow_ratio: f64,
    pub(in crate::project_slice::gcode_emit) support_flow_ratio: f64,
    pub(in crate::project_slice::gcode_emit) support_interface_flow_ratio: f64,
    pub(in crate::project_slice::gcode_emit) first_layer_flow_ratio: f64,
    pub(in crate::project_slice::gcode_emit) max_volumetric_speed: f64,
    pub(in crate::project_slice::gcode_emit) travel_feedrate: f64,
    pub(in crate::project_slice::gcode_emit) first_layer_travel_feedrate: f64,
    /// `_travel_to_z` speed — `travel_speed_z` when non-zero, else the
    /// first-layer or normal travel speed (`GCodeWriter.cpp:832-842`).
    pub(in crate::project_slice::gcode_emit) z_travel_feedrate: f64,
    pub(in crate::project_slice::gcode_emit) initial_layer_speed: f64,
    pub(in crate::project_slice::gcode_emit) initial_layer_infill_speed: f64,
    pub(in crate::project_slice::gcode_emit) inner_wall_speed: f64,
    pub(in crate::project_slice::gcode_emit) outer_wall_speed: f64,
    pub(in crate::project_slice::gcode_emit) small_perimeter_speed: f64,
    pub(in crate::project_slice::gcode_emit) small_perimeter_threshold: f64,
    pub(in crate::project_slice::gcode_emit) bridge_speed: f64,
    pub(in crate::project_slice::gcode_emit) internal_bridge_speed: f64,
    pub(in crate::project_slice::gcode_emit) sparse_infill_speed: f64,
    pub(in crate::project_slice::gcode_emit) skirt_speed: f64,
    /// `(min, max)` resonance-avoidance band when `resonance_avoidance`
    /// is enabled (`GCode.cpp:6618-6651`).
    pub(in crate::project_slice::gcode_emit) resonance_avoidance: Option<(f64, f64)>,
    pub(in crate::project_slice::gcode_emit) slow_down_layers: u32,
    pub(in crate::project_slice::gcode_emit) raft_layers: u32,
    pub(in crate::project_slice::gcode_emit) internal_solid_infill_speed: f64,
    pub(in crate::project_slice::gcode_emit) top_surface_speed: f64,
    pub(in crate::project_slice::gcode_emit) ironing_speed: f64,
    pub(in crate::project_slice::gcode_emit) gap_infill_speed: f64,
    pub(in crate::project_slice::gcode_emit) support_speed: f64,
    pub(in crate::project_slice::gcode_emit) support_type: crate::ProcessSupportType,
    pub(in crate::project_slice::gcode_emit) wall_direction: crate::ProcessWallDirection,
    /// `support_interface_speed` (`GCode.cpp:6548-6550`) —
    /// `erSupportMaterialInterface` paths use it, not `support_speed`.
    pub(in crate::project_slice::gcode_emit) support_interface_speed: f64,
    pub(in crate::project_slice::gcode_emit) enable_overhang_speed: bool,
    pub(in crate::project_slice::gcode_emit) slowdown_for_curled_perimeters: bool,
    pub(in crate::project_slice::gcode_emit) overhang_speed_bands: [Option<FloatOrPercent>; 4],
    pub(in crate::project_slice::gcode_emit) enable_overhang_bridge_fan: bool,
    pub(in crate::project_slice::gcode_emit) overhang_fan_speed: u8,
    pub(in crate::project_slice::gcode_emit) overhang_fan_threshold: RawOverhangFanThreshold,
    /// `change_extrusion_role_gcode` (machine), `process_change_extrusion_
    /// role_gcode` (process), `filament_change_extrusion_role_gcode`
    /// (filament[0]) — emitted at every extrusion-role transition
    /// (`GCode.cpp:6745-6770`).
    pub(in crate::project_slice::gcode_emit) role_change_machine: String,
    pub(in crate::project_slice::gcode_emit) role_change_process: String,
    pub(in crate::project_slice::gcode_emit) role_change_filament: String,
    pub(in crate::project_slice::gcode_emit) internal_bridge_fan_speed: InternalBridgeFanSpeed,
    pub(in crate::project_slice::gcode_emit) support_interface_fan_speed: Option<u8>,
    pub(in crate::project_slice::gcode_emit) ironing_fan_speed: Option<u8>,
    pub(in crate::project_slice::gcode_emit) initial_layer_acceleration: f64,
    pub(in crate::project_slice::gcode_emit) default_acceleration: f64,
    pub(in crate::project_slice::gcode_emit) outer_wall_acceleration: f64,
    pub(in crate::project_slice::gcode_emit) inner_wall_acceleration: f64,
    pub(in crate::project_slice::gcode_emit) bridge_acceleration: f64,
    pub(in crate::project_slice::gcode_emit) sparse_infill_acceleration: f64,
    pub(in crate::project_slice::gcode_emit) internal_solid_infill_acceleration: f64,
    pub(in crate::project_slice::gcode_emit) top_surface_acceleration: f64,
    pub(in crate::project_slice::gcode_emit) initial_layer_travel_acceleration: f64,
    pub(in crate::project_slice::gcode_emit) travel_acceleration: f64,
    pub(in crate::project_slice::gcode_emit) default_jerk: f64,
    pub(in crate::project_slice::gcode_emit) outer_wall_jerk: f64,
    pub(in crate::project_slice::gcode_emit) inner_wall_jerk: f64,
    pub(in crate::project_slice::gcode_emit) top_surface_jerk: f64,
    pub(in crate::project_slice::gcode_emit) infill_jerk: f64,
    pub(in crate::project_slice::gcode_emit) initial_layer_jerk: f64,
    pub(in crate::project_slice::gcode_emit) travel_jerk: f64,
    pub(in crate::project_slice::gcode_emit) max_jerk_x: f64,
    pub(in crate::project_slice::gcode_emit) max_jerk_y: f64,
    pub(in crate::project_slice::gcode_emit) max_jerk_z: f64,
    pub(in crate::project_slice::gcode_emit) max_jerk_e: f64,
    pub(in crate::project_slice::gcode_emit) gcode_flavor: crate::GCodeFlavor,
    pub(in crate::project_slice::gcode_emit) gcode_comments: bool,
    pub(in crate::project_slice::gcode_emit) extrusion_role_markers: bool,
    pub(in crate::project_slice::gcode_emit) use_relative_e_distances: bool,
    pub(in crate::project_slice::gcode_emit) single_extruder_multi_material: bool,
    pub(in crate::project_slice::gcode_emit) max_acceleration: u32,
    pub(in crate::project_slice::gcode_emit) max_travel_acceleration: u32,
    pub(in crate::project_slice::gcode_emit) accel_to_decel_enable: bool,
    pub(in crate::project_slice::gcode_emit) accel_to_decel_factor: f64,
    pub(in crate::project_slice::gcode_emit) retraction_length: f64,
    pub(in crate::project_slice::gcode_emit) retract_length_toolchange: f64,
    pub(in crate::project_slice::gcode_emit) retract_restart_extra_toolchange: f64,
    /// `GCodeWriter::_retract` swaps the E move for `G10`/`G22` when
    /// `use_firmware_retraction` is set (and `unretract` for `G11`/`G23`).
    pub(in crate::project_slice::gcode_emit) use_firmware_retraction: bool,
    /// `filament_retract_restart_extra` — added to the retraction length
    /// on de-retract (`Extruder::unretract`: `dE = m_retracted +
    /// m_restart_extra`).
    pub(in crate::project_slice::gcode_emit) retract_restart_extra: f64,
    pub(in crate::project_slice::gcode_emit) deretraction_feedrate: f64,
    /// Printer-scope `z_offset` (`GCode.cpp:5692`: every Z-target
    /// computation adds it to the model print Z; `;Z:` headers stay
    /// model-space). Z-hop/lift targets are `layer_z + z_hop` in the
    /// offset domain, matching upstream's already-offset `m_pos.z`.
    pub(in crate::project_slice::gcode_emit) z_offset: f64,
    pub(in crate::project_slice::gcode_emit) z_hop: f64,
    pub(in crate::project_slice::gcode_emit) retract_lift_above: f64,
    pub(in crate::project_slice::gcode_emit) retract_lift_below: f64,
    pub(in crate::project_slice::gcode_emit) retract_lift_enforce: RetractLiftEnforce,
    pub(in crate::project_slice::gcode_emit) retraction_feedrate: f64,
    /// Effective `retraction_speed` (filament `filament_retraction_speed`
    /// override applied, UNROUNDED, mm/s) for the during-wipe cap
    /// `retractionDuringWipe = retraction_speed * wipe_path_length /
    /// wipe_speed` (`GCode.cpp:350`). Distinct from `retraction_feedrate`,
    /// which is `floor(retraction_speed + 0.5) * 60` for the F word
    /// (`Extruder.cpp:184-186`).
    pub(in crate::project_slice::gcode_emit) retraction_during_wipe_speed: f64,
    /// Physical-extruder wipe budget (`GCode.cpp:342-350`), distinct from the logical path.
    pub(in crate::project_slice::gcode_emit) retraction_during_wipe_distance: f64,
    pub(in crate::project_slice::gcode_emit) wipe: bool,
    pub(in crate::project_slice::gcode_emit) wipe_distance: f64,
    pub(in crate::project_slice::gcode_emit) retraction_minimum_travel: f64,
    pub(in crate::project_slice::gcode_emit) reduce_infill_retraction: bool,
    pub(in crate::project_slice::gcode_emit) has_sparse_infill: bool,
    /// `GCode::_retract` suppresses the writer retract when the bottom
    /// (first layer) or top (TopSolidInfill destination) pattern is
    /// HilbertCurve (`GCode.cpp:7667-7672`).
    pub(in crate::project_slice::gcode_emit) bottom_surface_pattern_hilbert: bool,
    pub(in crate::project_slice::gcode_emit) top_surface_pattern_hilbert: bool,
    pub(in crate::project_slice::gcode_emit) retract_before_wipe: f64,
    pub(in crate::project_slice::gcode_emit) retract_after_wipe: f64,
    pub(in crate::project_slice::gcode_emit) role_based_wipe_speed: bool,
    pub(in crate::project_slice::gcode_emit) wipe_speed: f64,
    pub(in crate::project_slice::gcode_emit) wipe_on_loops: bool,
    pub(in crate::project_slice::gcode_emit) wipe_inward: bool,
    pub(in crate::project_slice::gcode_emit) wipe_inward_distance: FloatOrPercent,
    pub(in crate::project_slice::gcode_emit) wipe_before_external_loop: bool,
    pub(in crate::project_slice::gcode_emit) wall_loops: i32,
    pub(in crate::project_slice::gcode_emit) nozzle_diameter: f64,
    pub(in crate::project_slice::gcode_emit) retract_when_changing_layer: bool,
    pub(in crate::project_slice::gcode_emit) z_hop_type: crate::ZHopType,
    pub(in crate::project_slice::gcode_emit) travel_slope_radians: f64,
    pub(in crate::project_slice::gcode_emit) enable_arc_fitting: bool,
    pub(in crate::project_slice::gcode_emit) arc_fitting_tolerance: f64,
    pub(in crate::project_slice::gcode_emit) seam_gap: f64,
    pub(in crate::project_slice::gcode_emit) seam_position: crate::ProcessSeamPosition,
    pub(in crate::project_slice::gcode_emit) reduce_crossing_wall: bool,
    pub(in crate::project_slice::gcode_emit) crossing_boundary_inset: f64,
    /// `max_travel_detour_distance` (`PrintConfig.hpp:1489`), consumed by
    /// `AvoidCrossingPerimeters::travel_to`
    /// (`AvoidCrossingPerimeters.cpp:1291-1302`).
    pub(in crate::project_slice::gcode_emit) max_travel_detour_distance: FloatOrPercent,
    pub(in crate::project_slice::gcode_emit) scarf: ScarfOptions,
}
