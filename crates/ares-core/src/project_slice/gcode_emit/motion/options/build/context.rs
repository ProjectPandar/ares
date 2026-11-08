//! `from_traversal` prelude — the `let`-bindings that resolve the config views
//! and speeds used by the `MotionOptions` struct literal. Split from `build.rs`
//! for the <400 LOC rule; destructured back into identically-named locals so the
//! field expressions stay verbatim.

use super::region;
use super::retraction;
use crate::options::{
    FilamentRetractOverrideOptions, GCodeOptions, ProjectSettings, RegionOptions,
};
use crate::project::effective_config::types::ResolvedProjectObject;
use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;

/// The resolved inputs the `MotionOptions` field expressions draw from.
pub(super) struct Prelude<'a> {
    pub full: &'a ProjectSettings,
    pub gcode: &'a GCodeOptions,
    pub retract_overrides: &'a FilamentRetractOverrideOptions,
    pub object: Option<&'a ResolvedProjectObject>,
    pub region: Option<&'a RegionOptions>,
    pub filament: usize,
    pub physical: usize,
    pub filament_diameter: f64,
    pub travel_speed: f64,
    pub travel_acceleration: f64,
    pub outer_wall_acceleration: f64,
    pub default_acceleration: f64,
    pub retraction_speed: f64,
    pub deretraction_speed: f64,
}

pub(super) fn prelude(traversal: &PreparedPostClassicTraversal, filament: usize) -> Prelude<'_> {
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
        .get(filament)
        .or_else(|| gcode.filament_diameter.0.first())
        .map_or(1.75, |value| value.0);
    let nozzle_index = crate::options::mapped_extruder_id(full, filament);
    let travel_speed = gcode.travel_speed.get_at(nozzle_index);
    let travel_acceleration = object.map_or(
        full.process.object.travel_acceleration.get_at(nozzle_index),
        |value| value.object.travel_acceleration.get_at(nozzle_index),
    );
    let accelerations = region::accelerations(full, object, nozzle_index);
    let (retraction_speed, deretraction_speed) = retraction::speeds(gcode, filament);
    Prelude {
        full,
        gcode,
        retract_overrides,
        object,
        region,
        filament,
        physical: gcode
            .filament_map
            .0
            .get(filament)
            .map_or(0, |value| (value.0 - 1) as usize),
        filament_diameter,
        travel_speed,
        travel_acceleration,
        outer_wall_acceleration: accelerations.outer_wall_acceleration,
        default_acceleration: accelerations.default_acceleration,
        retraction_speed,
        deretraction_speed,
    }
}
