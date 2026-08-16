use crate::{FloatOrPercent, Nullable, OrcaFloat, ZHopType};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(in crate::project_slice::gcode_emit) struct MotionOptions {
    pub(in crate::project_slice::gcode_emit) filament_area: f64,
    pub(in crate::project_slice::gcode_emit) filament_flow_ratio: f64,
    pub(in crate::project_slice::gcode_emit) max_volumetric_speed: f64,
    pub(in crate::project_slice::gcode_emit) travel_feedrate: f64,
    pub(in crate::project_slice::gcode_emit) first_layer_travel_feedrate: f64,
    pub(in crate::project_slice::gcode_emit) initial_layer_speed: f64,
    pub(in crate::project_slice::gcode_emit) initial_layer_infill_speed: f64,
    pub(in crate::project_slice::gcode_emit) inner_wall_speed: f64,
    pub(in crate::project_slice::gcode_emit) outer_wall_speed: f64,
    pub(in crate::project_slice::gcode_emit) bridge_speed: f64,
    pub(in crate::project_slice::gcode_emit) internal_bridge_speed: f64,
    pub(in crate::project_slice::gcode_emit) sparse_infill_speed: f64,
    pub(in crate::project_slice::gcode_emit) internal_solid_infill_speed: f64,
    pub(in crate::project_slice::gcode_emit) top_surface_speed: f64,
    pub(in crate::project_slice::gcode_emit) gap_infill_speed: f64,
    pub(in crate::project_slice::gcode_emit) initial_layer_acceleration: u32,
    pub(in crate::project_slice::gcode_emit) default_acceleration: u32,
    pub(in crate::project_slice::gcode_emit) outer_wall_acceleration: u32,
    pub(in crate::project_slice::gcode_emit) bridge_acceleration: u32,
    pub(in crate::project_slice::gcode_emit) top_surface_acceleration: u32,
    pub(in crate::project_slice::gcode_emit) initial_layer_travel_acceleration: u32,
    pub(in crate::project_slice::gcode_emit) travel_acceleration: u32,
    pub(in crate::project_slice::gcode_emit) retraction_length: f64,
    pub(in crate::project_slice::gcode_emit) deretraction_feedrate: f64,
    pub(in crate::project_slice::gcode_emit) z_hop: f64,
    pub(in crate::project_slice::gcode_emit) retraction_feedrate: f64,
    pub(in crate::project_slice::gcode_emit) wipe: bool,
    pub(in crate::project_slice::gcode_emit) wipe_distance: f64,
    pub(in crate::project_slice::gcode_emit) retraction_minimum_travel: f64,
    pub(in crate::project_slice::gcode_emit) reduce_infill_retraction: bool,
    pub(in crate::project_slice::gcode_emit) retract_before_wipe: f64,
    pub(in crate::project_slice::gcode_emit) role_based_wipe_speed: bool,
    pub(in crate::project_slice::gcode_emit) wipe_speed: f64,
    pub(in crate::project_slice::gcode_emit) retract_when_changing_layer: bool,
    pub(in crate::project_slice::gcode_emit) spiral_lift: bool,
    pub(in crate::project_slice::gcode_emit) travel_slope_radians: f64,
    pub(in crate::project_slice::gcode_emit) enable_arc_fitting: bool,
    pub(in crate::project_slice::gcode_emit) arc_fitting_tolerance: f64,
    pub(in crate::project_slice::gcode_emit) seam_gap: f64,
}

impl MotionOptions {
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
        Self {
            filament_area: std::f64::consts::PI * filament_diameter.powi(2) * 0.25,
            filament_flow_ratio: first_nullable_float(&gcode.filament_flow_ratio, 1.0),
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
            initial_layer_acceleration: acceleration(
                object.map(|value| &value.object),
                full.process.object.initial_layer_acceleration.0,
                |value| value.initial_layer_acceleration.0,
            ),
            default_acceleration: acceleration(
                object.map(|value| &value.object),
                full.process.object.default_acceleration.0,
                |value| value.default_acceleration.0,
            ),
            outer_wall_acceleration,
            bridge_acceleration: rounded_acceleration(absolute(
                object.map_or(full.process.object.bridge_acceleration, |value| {
                    value.object.bridge_acceleration
                }),
                f64::from(outer_wall_acceleration),
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

fn acceleration(
    object: Option<&crate::ObjectOptions>,
    fallback: f64,
    value: impl Fn(&crate::ObjectOptions) -> f64,
) -> u32 {
    rounded_acceleration(object.map_or(fallback, value))
}

fn absolute(value: FloatOrPercent, base: f64) -> f64 {
    match value {
        FloatOrPercent::Float(value) => value,
        FloatOrPercent::Percent(value) => base * value.0 / 100.0,
    }
}

fn rounded_acceleration(value: f64) -> u32 {
    (value + 0.5).floor() as u32
}

pub(in crate::project_slice::gcode_emit) fn first_nullable_float(
    values: &[Nullable<OrcaFloat>],
    default: f64,
) -> f64 {
    values
        .iter()
        .find_map(|value| match value {
            Nullable::Value(value) => Some(value.0),
            Nullable::Nil => None,
        })
        .unwrap_or(default)
}
