//! `MotionOptions` field assembly (split from `fields.rs`).

use super::super::super::MotionOptions;
use super::super::super::helpers::{
    absolute, acceleration, first_float, machine_acceleration_limit, rounded_acceleration,
    use_mach_limits,
};
use super::super::super::nullable_float_at;
use super::super::context::Prelude;
use super::super::crossing;
use crate::options::InternalBridgeFanSpeed;

pub(in crate::project_slice::gcode_emit::motion::options::build) fn assemble(
    traversal: &crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
    prelude: Prelude<'_>,
) -> MotionOptions {
    let wipe_tail = super::wipe::tail(&prelude);
    let Prelude {
        full,
        gcode,
        object,
        filament,
        filament_diameter,
        travel_speed,
        travel_acceleration,
        outer_wall_acceleration,
        default_acceleration,
        ..
    } = prelude;
    let use_mach_limits = use_mach_limits(full.printer.gcode.gcode_flavor);
    let mut options = MotionOptions {
        adaptive_pa: crate::project_slice::gcode_emit::adaptive_pa::Options::new(full, filament),
        // `filament_crossection` (Extruder.hpp:61): `diameter*diameter*0.25*PI`
        // — the multiplication order matters for f64 parity with upstream.
        filament_area: filament_diameter * filament_diameter * 0.25 * std::f64::consts::PI,
        filament_density: gcode
            .filament_density
            .0
            .get(filament)
            .or_else(|| gcode.filament_density.0.first())
            .map_or(0.0, |value| value.0),
        filament_flow_ratio: nullable_float_at(&gcode.filament_flow_ratio, filament, 1.0),
        set_other_flow_ratios: object
            .map_or(full.process.object.set_other_flow_ratios.0, |value| {
                value.object.set_other_flow_ratios.0
            }),
        max_volumetric_speed: gcode
            .filament_max_volumetric_speed
            .0
            .get(filament)
            .or_else(|| gcode.filament_max_volumetric_speed.0.first())
            .map_or(0.0, |value| value.0),
        travel_feedrate: travel_speed * 60.0,
        z_travel_feedrate: gcode.travel_speed_z.0 * 60.0,
        first_layer_travel_feedrate: absolute(gcode.initial_layer_travel_speed, travel_speed)
            * 60.0,
        initial_layer_speed: full.process.print.initial_layer_speed.0,
        initial_layer_infill_speed: full.process.print.initial_layer_infill_speed.0,
        skirt_speed: full.process.print.skirt_speed.0,
        resonance_avoidance: full.printer.machine.resonance_avoidance.0.then_some((
            full.printer.machine.min_resonance_avoidance_speed.0,
            full.printer.machine.max_resonance_avoidance_speed.0,
        )),
        slow_down_layers: full.process.print.slow_down_layers.0.max(0) as u32,
        raft_layers: full.process.object.raft_layers.0.max(0) as u32,
        support_speed: object.map_or(full.process.object.support_speed.0, |value| {
            value.object.support_speed.0
        }),
        support_flow_ratio: object.map_or(full.process.object.support_flow_ratio.0, |value| {
            value.object.support_flow_ratio.0
        }),
        support_interface_flow_ratio: object.map_or(
            full.process.object.support_interface_flow_ratio.0,
            |value| value.object.support_interface_flow_ratio.0,
        ),
        support_type: object.map_or(full.process.object.support_type, |value| {
            value.object.support_type
        }),
        support_interface_speed: object
            .map_or(full.process.object.support_interface_speed.0, |value| {
                value.object.support_interface_speed.0
            }),
        enable_overhang_bridge_fan: full
            .filament
            .print
            .enable_overhang_bridge_fan
            .0
            .get(filament)
            .or_else(|| full.filament.print.enable_overhang_bridge_fan.0.first())
            .is_some_and(|value| value.0),
        overhang_fan_speed: full
            .filament
            .print
            .overhang_fan_speed
            .0
            .get(filament)
            .or_else(|| full.filament.print.overhang_fan_speed.0.first())
            .map_or(100, |value| value.0.clamp(0, 100) as u8),
        overhang_fan_threshold: full
            .filament
            .print
            .overhang_fan_threshold
            .get(filament)
            .or_else(|| full.filament.print.overhang_fan_threshold.first())
            .copied()
            .unwrap_or_default(),
        role_change_machine: gcode.change_extrusion_role_gcode.0.clone(),
        role_change_process: gcode.process_change_extrusion_role_gcode.0.clone(),
        role_change_filament: gcode
            .filament_change_extrusion_role_gcode
            .0
            .get(filament)
            .or_else(|| gcode.filament_change_extrusion_role_gcode.0.first())
            .cloned()
            .unwrap_or_default(),
        internal_bridge_fan_speed: full
            .filament
            .print
            .internal_bridge_fan_speed
            .0
            .get(filament)
            .or_else(|| full.filament.print.internal_bridge_fan_speed.0.first())
            .map_or_else(InternalBridgeFanSpeed::fallback, |value| {
                if value.0 < 0 {
                    InternalBridgeFanSpeed::fallback()
                } else {
                    InternalBridgeFanSpeed::new(value.0.clamp(0, 100) as u8)
                }
            }),
        support_interface_fan_speed: full
            .filament
            .print
            .support_material_interface_fan_speed
            .0
            .get(filament)
            .or_else(|| {
                full.filament
                    .print
                    .support_material_interface_fan_speed
                    .0
                    .first()
            })
            .and_then(|value| (value.0 >= 0).then_some(value.0 as u8)),
        ironing_fan_speed: full
            .filament
            .print
            .ironing_fan_speed
            .0
            .get(filament)
            .or_else(|| full.filament.print.ironing_fan_speed.0.first())
            .and_then(|value| (value.0 >= 0).then_some(value.0 as u8)),
        initial_layer_acceleration: acceleration(
            object.map(|value| &value.object),
            full.process.object.initial_layer_acceleration.0,
            |value| value.initial_layer_acceleration.0,
        ),
        default_acceleration,
        outer_wall_acceleration,
        inner_wall_acceleration: acceleration(
            object.map(|value| &value.object),
            full.process.object.inner_wall_acceleration.0,
            |value| value.inner_wall_acceleration.0,
        ),
        bridge_acceleration: absolute(
            object.map_or(full.process.object.bridge_acceleration, |value| {
                value.object.bridge_acceleration
            }),
            outer_wall_acceleration,
        ),
        sparse_infill_acceleration: absolute(
            object.map_or(full.process.object.sparse_infill_acceleration, |value| {
                value.object.sparse_infill_acceleration
            }),
            default_acceleration,
        ),
        internal_solid_infill_acceleration: absolute(
            object.map_or(
                full.process.object.internal_solid_infill_acceleration,
                |value| value.object.internal_solid_infill_acceleration,
            ),
            default_acceleration,
        ),
        top_surface_acceleration: acceleration(
            object.map(|value| &value.object),
            full.process.object.top_surface_acceleration.0,
            |value| value.top_surface_acceleration.0,
        ),
        initial_layer_travel_acceleration: absolute(
            gcode.initial_layer_travel_acceleration,
            travel_acceleration,
        ),
        travel_acceleration,
        default_jerk: object.map_or(full.process.object.default_jerk.0, |value| {
            value.object.default_jerk.0
        }),
        outer_wall_jerk: object.map_or(full.process.object.outer_wall_jerk.0, |value| {
            value.object.outer_wall_jerk.0
        }),
        inner_wall_jerk: object.map_or(full.process.object.inner_wall_jerk.0, |value| {
            value.object.inner_wall_jerk.0
        }),
        top_surface_jerk: object.map_or(full.process.object.top_surface_jerk.0, |value| {
            value.object.top_surface_jerk.0
        }),
        infill_jerk: object.map_or(full.process.object.infill_jerk.0, |value| {
            value.object.infill_jerk.0
        }),
        initial_layer_jerk: object.map_or(full.process.object.initial_layer_jerk.0, |value| {
            value.object.initial_layer_jerk.0
        }),
        travel_jerk: object.map_or(full.process.object.travel_jerk.0, |value| {
            value.object.travel_jerk.0
        }),
        max_jerk_x: if use_mach_limits {
            first_float(&full.printer.machine.machine_max_jerk_x).round_ties_even()
        } else {
            0.0
        },
        max_jerk_y: if use_mach_limits {
            first_float(&full.printer.machine.machine_max_jerk_y).round_ties_even()
        } else {
            0.0
        },
        max_jerk_z: first_float(&full.printer.machine.machine_max_jerk_z),
        max_jerk_e: first_float(&full.printer.machine.machine_max_jerk_e),
        gcode_flavor: full.printer.gcode.gcode_flavor,
        gcode_comments: full.process.print.gcode_comments.0,
        extrusion_role_markers: full.process.gcode.max_volumetric_extrusion_rate_slope.0 > 0.0,
        use_relative_e_distances: gcode.use_relative_e_distances.0,
        single_extruder_multi_material: gcode.single_extruder_multi_material.0,
        max_acceleration: machine_acceleration_limit(full),
        max_travel_acceleration: if use_mach_limits
            && matches!(
                full.printer.gcode.gcode_flavor,
                crate::GCodeFlavor::Repetier
                    | crate::GCodeFlavor::MarlinFirmware
                    | crate::GCodeFlavor::RepRapFirmware
            ) {
            rounded_acceleration(first_float(
                &full.printer.machine.machine_max_acceleration_travel,
            ))
        } else {
            0
        },
        accel_to_decel_enable: gcode.accel_to_decel_enable.0,
        accel_to_decel_factor: gcode.accel_to_decel_factor.0,
        use_firmware_retraction: gcode.use_firmware_retraction.0,
        z_offset: traversal.resolved.views.full.printer.remaining.z_offset.0,
        reduce_infill_retraction: full.process.print.reduce_infill_retraction.0,
        retract_before_wipe: wipe_tail.retract_before_wipe,
        nozzle_diameter: wipe_tail.nozzle_diameter,
        seam_position: full.process.object.seam_position,
        reduce_crossing_wall: full.process.print.reduce_crossing_wall.0,
        crossing_boundary_inset: crossing::boundary_inset(traversal),
        max_travel_detour_distance: full.process.print.max_travel_detour_distance,
        // GCode.cpp:9915 / GCodeWriter.cpp:1218 (345efec64c7b).
        enable_arc_fitting: gcode.enable_arc_fitting != crate::ArcFittingMode::Disabled,
        arc_fitting_tolerance: full.process.print.resolution.0,
        ..Default::default()
    };
    super::retraction::apply(&mut options, &prelude, &traversal.resolved.views.runtime);
    // `GCode.cpp:6136/6163` — the same `region::apply` the emit loop
    // runs per phase populates every region-scoped field (speeds,
    // flow, patterns, wipe knobs, seam_gap, scarf) from the initially
    // resolved region (or `full.process.region` when `region` is
    // `None`); `nozzle_diameter` was set by `wipe::tail`.
    let nozzle = options.nozzle_diameter;
    super::super::region::apply(
        &mut options,
        &prelude.full.process.region,
        prelude.region,
        prelude.travel_speed,
        nozzle,
    );
    options
}
