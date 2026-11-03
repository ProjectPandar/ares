//! Model.cpp:2976-3010,3222-3255: global table and object-config speed lookup.

use crate::{
    options::{FloatOrPercent, ObjectOptionOverrides, RegionOptionOverrides},
    project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

pub(super) fn find_max_speed(traversal: &PreparedPostClassicTraversal, object_index: usize) -> f64 {
    let global = &traversal.resolved.views.full.process;
    let (source_index, _) = traversal.objects[object_index]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object
        .identity();
    let source = &traversal.project.objects()[source_index];
    let object = source.object_overrides();
    let region = source.region_overrides();
    let speeds = [
        global.region.inner_wall_speed.get_at(0),
        global.region.outer_wall_speed.get_at(0),
        global.region.sparse_infill_speed.0,
        global.region.internal_solid_infill_speed.0,
        global.region.top_surface_speed.0,
        global.object.support_speed.0,
    ];
    // Non-speed keys such as extruder also select the object-key branch.
    if object == &ObjectOptionOverrides::default() && region == &RegionOptionOverrides::default() {
        return speeds
            .into_iter()
            .fold(0.0, |max, speed| if speed > max { speed } else { max });
    }
    let inner = region
        .inner_wall_speed
        .as_ref()
        .map_or(speeds[0], |speed| speed.get_at(0));
    let mut outer = if region.inner_wall_speed.is_some() {
        speeds[1] / speeds[0] * inner
    } else {
        speeds[1]
    };
    if let Some(speed) = &region.outer_wall_speed {
        outer = speed.get_at(0);
    }
    // Model.cpp:58 value-initializes printSpeedMap; setPrintSpeedTable never
    // assigns smallPerimeterSpeed. Only an object key replaces that zero.
    let small = region
        .small_perimeter_speed
        .map_or(0.0, |speed| match speed {
            FloatOrPercent::Float(value) => value,
            FloatOrPercent::Percent(value) => value.0,
        });
    let speeds = [
        inner,
        outer,
        region
            .sparse_infill_speed
            .map_or(speeds[2], |speed| speed.0),
        region
            .internal_solid_infill_speed
            .map_or(speeds[3], |speed| speed.0),
        region.top_surface_speed.map_or(speeds[4], |speed| speed.0),
        object.support_speed.map_or(speeds[5], |speed| speed.0),
        small,
    ];
    // Retain the operand order of the upstream nested std::max calls.
    let max = speeds
        .into_iter()
        .rev()
        .fold(-1.0, |max, speed| if speed < max { max } else { speed });
    if max <= 0.0 { 250.0 } else { max }
}
