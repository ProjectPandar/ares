//! Builds `MotionOptions` from the resolved project configuration
//! (`GCodeWriter::apply_print_config` equivalents).

use super::MotionOptions;
use super::first_nullable_float;
use super::helpers::{absolute, acceleration, first_float, rounded_acceleration};
use crate::{Nullable, ZHopType, options::InternalBridgeFanSpeed};

impl MotionOptions {
    /// `GCodeWriter::apply_print_config` caps print acceleration by the
    /// machine extruding limit (Klipper additionally clamps by per-axis X/Y
    /// limits, `GCodeWriter.cpp:33-45`).
    fn machine_acceleration_limit(full: &crate::options::ProjectSettings) -> u32 {
        let flavor = full.printer.gcode.gcode_flavor;
        if !matches!(
            flavor,
            crate::GCodeFlavor::MarlinLegacy
                | crate::GCodeFlavor::MarlinFirmware
                | crate::GCodeFlavor::Klipper
                | crate::GCodeFlavor::RepRapFirmware
        ) {
            return 0;
        }
        let mut limit = rounded_acceleration(first_float(
            &full.printer.machine.machine_max_acceleration_extruding,
        ));
        if flavor == crate::GCodeFlavor::Klipper {
            let axis_limit = [
                &full.printer.machine.machine_max_acceleration_x,
                &full.printer.machine.machine_max_acceleration_y,
            ]
            .into_iter()
            .map(|axis| rounded_acceleration(first_float(axis)))
            .filter(|axis_limit| *axis_limit > 0)
            .min()
            .unwrap_or(u32::MAX);
            limit = limit.min(axis_limit);
        }
        limit
    }
    pub(in crate::project_slice::gcode_emit) fn from_traversal(
        traversal: &crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
    ) -> Self {
        let full = &traversal.resolved.views.full;
        let gcode = &traversal.resolved.views.runtime_gcode;
        let retract_overrides = &full.filament.retract_overrides;
        let object = traversal.resolved.objects.first();
        let region = object
            .and_then(|object| object.layer_candidates.first())
            .and_then(|layer| layer.model_parts.first())
            .map(|part| &part.region);
        let filament_diameter = gcode
            .filament_diameter
            .0
            .first()
            .map_or(1.75, |value| value.0);
        let travel_speed = gcode.travel_speed.0;
        let travel_acceleration = object
            .map_or(full.process.object.travel_acceleration.0, |value| {
                value.object.travel_acceleration.0
            });
        let bridge_speed = region.map_or(full.process.region.bridge_speed.0, |value| {
            value.bridge_speed.0
        });
        let outer_wall_acceleration = acceleration(
            object.map(|value| &value.object),
            full.process.object.outer_wall_acceleration.0,
            |value| value.outer_wall_acceleration.0,
        );
        let default_acceleration = acceleration(
            object.map(|value| &value.object),
            full.process.object.default_acceleration.0,
            |value| value.default_acceleration.0,
        );
        Self {
            filament_area: std::f64::consts::PI * filament_diameter.powi(2) * 0.25,
            filament_flow_ratio: first_nullable_float(&gcode.filament_flow_ratio, 1.0),
            print_flow_ratio: region.map_or(full.process.region.print_flow_ratio.0, |value| {
                value.print_flow_ratio.0
            }),
            max_volumetric_speed: gcode
                .filament_max_volumetric_speed
                .0
                .first()
                .map_or(0.0, |value| value.0),
            travel_feedrate: travel_speed * 60.0,
            first_layer_travel_feedrate: absolute(gcode.initial_layer_travel_speed, travel_speed)
                * 60.0,
            initial_layer_speed: full.process.print.initial_layer_speed.0,
            initial_layer_infill_speed: full.process.print.initial_layer_infill_speed.0,
            inner_wall_speed: region.map_or(full.process.region.inner_wall_speed.0, |value| {
                value.inner_wall_speed.0
            }),
            outer_wall_speed: region.map_or(full.process.region.outer_wall_speed.0, |value| {
                value.outer_wall_speed.0
            }),
            bridge_speed,
            internal_bridge_speed: absolute(
                region.map_or(full.process.region.internal_bridge_speed, |value| {
                    value.internal_bridge_speed
                }),
                bridge_speed,
            ),
            sparse_infill_speed: region
                .map_or(full.process.region.sparse_infill_speed.0, |value| {
                    value.sparse_infill_speed.0
                }),
            skirt_speed: full.process.print.skirt_speed.0,
            internal_solid_infill_speed: region
                .map_or(full.process.region.internal_solid_infill_speed.0, |value| {
                    value.internal_solid_infill_speed.0
                }),
            top_surface_speed: region.map_or(full.process.region.top_surface_speed.0, |value| {
                value.top_surface_speed.0
            }),
            gap_infill_speed: region.map_or(full.process.region.gap_infill_speed.0, |value| {
                value.gap_infill_speed.0
            }),
            enable_overhang_speed: region
                .map_or(full.process.region.enable_overhang_speed.0, |value| {
                    value.enable_overhang_speed.0
                }),
            slowdown_for_curled_perimeters: region.map_or(
                full.process.region.slowdown_for_curled_perimeters.0,
                |value| value.slowdown_for_curled_perimeters.0,
            ),
            overhang_speed_bands: [
                Some(
                    region.map_or(full.process.region.overhang_1_4_speed, |value| {
                        value.overhang_1_4_speed
                    }),
                ),
                Some(
                    region.map_or(full.process.region.overhang_2_4_speed, |value| {
                        value.overhang_2_4_speed
                    }),
                ),
                Some(
                    region.map_or(full.process.region.overhang_3_4_speed, |value| {
                        value.overhang_3_4_speed
                    }),
                ),
                Some(
                    region.map_or(full.process.region.overhang_4_4_speed, |value| {
                        value.overhang_4_4_speed
                    }),
                ),
            ],
            enable_overhang_bridge_fan: full
                .filament
                .print
                .enable_overhang_bridge_fan
                .0
                .first()
                .is_some_and(|value| value.0),
            overhang_fan_speed: full
                .filament
                .print
                .overhang_fan_speed
                .0
                .first()
                .map_or(100, |value| value.0.clamp(0, 100) as u8),
            overhang_fan_threshold: full
                .filament
                .print
                .overhang_fan_threshold
                .first()
                .copied()
                .unwrap_or_default(),
            internal_bridge_fan_speed: full
                .filament
                .print
                .internal_bridge_fan_speed
                .0
                .first()
                .map_or_else(InternalBridgeFanSpeed::fallback, |value| {
                    if value.0 < 0 {
                        InternalBridgeFanSpeed::fallback()
                    } else {
                        InternalBridgeFanSpeed::new(value.0.clamp(0, 100) as u8)
                    }
                }),
            close_fan_first_layers: full
                .filament
                .print
                .close_fan_the_first_x_layers
                .0
                .first()
                .map_or(0, |value| value.0.max(0) as usize),
            full_fan_speed_layer: full
                .filament
                .print
                .full_fan_speed_layer
                .0
                .first()
                .map_or(0, |value| value.0.max(0) as usize),
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
            bridge_acceleration: rounded_acceleration(absolute(
                object.map_or(full.process.object.bridge_acceleration, |value| {
                    value.object.bridge_acceleration
                }),
                f64::from(outer_wall_acceleration),
            )),
            sparse_infill_acceleration: rounded_acceleration(absolute(
                object.map_or(full.process.object.sparse_infill_acceleration, |value| {
                    value.object.sparse_infill_acceleration
                }),
                f64::from(default_acceleration),
            )),
            internal_solid_infill_acceleration: rounded_acceleration(absolute(
                object.map_or(
                    full.process.object.internal_solid_infill_acceleration,
                    |value| value.object.internal_solid_infill_acceleration,
                ),
                f64::from(default_acceleration),
            )),
            top_surface_acceleration: acceleration(
                object.map(|value| &value.object),
                full.process.object.top_surface_acceleration.0,
                |value| value.top_surface_acceleration.0,
            ),
            initial_layer_travel_acceleration: rounded_acceleration(absolute(
                gcode.initial_layer_travel_acceleration,
                travel_acceleration,
            )),
            travel_acceleration: rounded_acceleration(travel_acceleration),
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
            max_jerk_x: first_float(&full.printer.machine.machine_max_jerk_x),
            max_jerk_y: first_float(&full.printer.machine.machine_max_jerk_y),
            max_jerk_z: first_float(&full.printer.machine.machine_max_jerk_z),
            max_jerk_e: first_float(&full.printer.machine.machine_max_jerk_e),
            gcode_flavor: full.printer.gcode.gcode_flavor,
            max_acceleration: Self::machine_acceleration_limit(full),
            max_travel_acceleration: if matches!(
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
            retraction_length: gcode
                .retraction_length
                .0
                .first()
                .map_or(0.0, |value| value.0),
            deretraction_feedrate: gcode
                .deretraction_speed
                .0
                .first()
                .map_or(0.0, |value| value.0)
                * 60.0,
            z_hop: gcode.z_hop.0.first().map_or(0.0, |value| value.0),
            retraction_feedrate: gcode
                .retraction_speed
                .0
                .first()
                .map_or(0.0, |value| value.0)
                * 60.0,
            wipe: retract_overrides
                .filament_wipe
                .iter()
                .find_map(|value| match value {
                    Nullable::Value(value) => Some(value.0),
                    Nullable::Nil => None,
                })
                .unwrap_or_else(|| {
                    full.project
                        .print
                        .wipe
                        .0
                        .first()
                        .is_some_and(|value| value.0)
                }),
            wipe_distance: first_nullable_float(
                &retract_overrides.filament_wipe_distance,
                full.project
                    .print
                    .wipe_distance
                    .0
                    .first()
                    .map_or(0.0, |value| value.0),
            ),
            retraction_minimum_travel: first_nullable_float(
                &retract_overrides.filament_retraction_minimum_travel,
                full.project
                    .print
                    .retraction_minimum_travel
                    .0
                    .first()
                    .map_or(0.0, |value| value.0),
            ),
            reduce_infill_retraction: full.process.print.reduce_infill_retraction.0,
            retract_before_wipe: gcode
                .retract_before_wipe
                .0
                .first()
                .map_or(0.0, |value| value.0 / 100.0),
            role_based_wipe_speed: full.process.region.role_based_wipe_speed.0,
            wipe_speed: absolute(
                region.map_or(full.process.region.wipe_speed, |value| value.wipe_speed),
                travel_speed,
            ),
            retract_when_changing_layer: traversal
                .resolved
                .views
                .runtime
                .project
                .print
                .retract_when_changing_layer
                .0
                .first()
                .is_some_and(|value| value.0),
            spiral_lift: gcode
                .z_hop_types
                .0
                .first()
                .is_some_and(|value| matches!(value, ZHopType::Auto | ZHopType::Spiral)),
            travel_slope_radians: gcode
                .travel_slope
                .0
                .first()
                .map_or(0.0, |value| value.0.to_radians()),
            seam_gap: absolute(
                region.map_or(full.process.region.seam_gap, |value| value.seam_gap),
                full.project
                    .print
                    .nozzle_diameter
                    .0
                    .first()
                    .map_or(0.4, |value| value.0),
            ),
            enable_arc_fitting: gcode.enable_arc_fitting.0,
            arc_fitting_tolerance: full.process.print.resolution.0,
        }
    }
}
