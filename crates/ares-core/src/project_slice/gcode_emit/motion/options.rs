use crate::{FloatOrPercent, Nullable, OrcaFloat};

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
    pub(in crate::project_slice::gcode_emit) sparse_infill_speed: f64,
    pub(in crate::project_slice::gcode_emit) internal_solid_infill_speed: f64,
    pub(in crate::project_slice::gcode_emit) top_surface_speed: f64,
    pub(in crate::project_slice::gcode_emit) gap_infill_speed: f64,
    pub(in crate::project_slice::gcode_emit) initial_layer_acceleration: u32,
    pub(in crate::project_slice::gcode_emit) default_acceleration: u32,
    pub(in crate::project_slice::gcode_emit) outer_wall_acceleration: u32,
    pub(in crate::project_slice::gcode_emit) top_surface_acceleration: u32,
}

impl MotionOptions {
    pub(in crate::project_slice::gcode_emit) fn from_traversal(
        traversal: &crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
    ) -> Self {
        let full = &traversal.resolved.views.full;
        let gcode = &traversal.resolved.views.runtime_gcode;
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
            outer_wall_acceleration: acceleration(
                object.map(|value| &value.object),
                full.process.object.outer_wall_acceleration.0,
                |value| value.outer_wall_acceleration.0,
            ),
            top_surface_acceleration: acceleration(
                object.map(|value| &value.object),
                full.process.object.top_surface_acceleration.0,
                |value| value.top_surface_acceleration.0,
            ),
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
