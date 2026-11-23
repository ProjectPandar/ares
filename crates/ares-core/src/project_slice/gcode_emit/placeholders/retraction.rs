use crate::options::{
    OrcaFloats, RetractLiftEnforce, ZHopType, project_config_views::ProjectConfigViews,
};

use super::value::{Config, Value};

/// `PrintApply.cpp:1665` exposes filament overrides to templates without
/// changing the full configuration used for serialization.
pub(super) fn insert(config: &mut Config, views: &ProjectConfigViews) {
    let gcode = &views.runtime_gcode;
    let print = &views.runtime.project.print;
    for (name, values) in [
        ("deretraction_speed", &gcode.deretraction_speed),
        (
            "retract_length_toolchange",
            &gcode.retract_length_toolchange,
        ),
        ("retract_lift_above", &gcode.retract_lift_above),
        ("retract_lift_below", &gcode.retract_lift_below),
        ("retract_restart_extra", &gcode.retract_restart_extra),
        (
            "retract_restart_extra_toolchange",
            &gcode.retract_restart_extra_toolchange,
        ),
        ("retraction_length", &gcode.retraction_length),
        ("retract_length", &gcode.retraction_length),
        (
            "retraction_minimum_travel",
            &print.retraction_minimum_travel,
        ),
        ("retraction_speed", &gcode.retraction_speed),
        ("wipe_distance", &print.wipe_distance),
        ("z_hop", &gcode.z_hop),
    ] {
        config.insert(name, floats(values));
    }
    for (name, values) in [
        ("retract_after_wipe", &gcode.retract_after_wipe),
        ("retract_before_wipe", &gcode.retract_before_wipe),
    ] {
        config.insert(
            name,
            Value::List(
                values
                    .0
                    .iter()
                    .map(|value| Value::number(value.0))
                    .collect(),
            ),
        );
    }
    for (name, values) in [
        (
            "retract_when_changing_layer",
            &print.retract_when_changing_layer,
        ),
        ("wipe", &print.wipe),
    ] {
        config.insert(
            name,
            Value::List(
                values
                    .0
                    .iter()
                    .map(|value| Value::option_bool(value.0))
                    .collect(),
            ),
        );
    }

    // `PlaceholderParser.cpp:1361` exposes enumeration vectors as integers.
    config.insert(
        "z_hop_types",
        Value::List(
            gcode
                .z_hop_types
                .0
                .iter()
                .map(|value| {
                    Value::Integer(match value {
                        ZHopType::Auto => 0,
                        ZHopType::Normal => 1,
                        ZHopType::Slope => 2,
                        ZHopType::Spiral => 3,
                    })
                })
                .collect(),
        ),
    );
    config.insert(
        "retract_lift_enforce",
        Value::List(
            gcode
                .retract_lift_enforce
                .0
                .iter()
                .map(|value| {
                    Value::Integer(match value {
                        RetractLiftEnforce::AllSurfaces => 0,
                        RetractLiftEnforce::TopOnly => 1,
                        RetractLiftEnforce::BottomOnly => 2,
                        RetractLiftEnforce::TopAndBottom => 3,
                    })
                })
                .collect(),
        ),
    );
}

fn floats(values: &OrcaFloats) -> Value {
    Value::List(
        values
            .0
            .iter()
            .map(|value| Value::number(value.0))
            .collect(),
    )
}
