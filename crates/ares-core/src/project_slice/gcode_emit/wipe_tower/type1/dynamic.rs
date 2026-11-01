//! `GCode.cpp:847–977`: Type1 toolchange template variables.
use super::super::super::{
    motion, timelapse, tool_transition,
    value::{Config, Value},
};
use super::super::{filament::at, postprocess, result::ToolChangeResult};

pub(super) fn insert(
    config: &mut Config,
    state: &motion::EmitState,
    result: &ToolChangeResult,
    context: timelapse::Context<'_>,
    (start, z, scale): ([f32; 2], f64, crate::geometry::CoordinateScale),
) -> Result<(), crate::SliceError> {
    let metadata = result.type1.as_ref().unwrap();
    let tool = result.new_tool;
    tool_transition::type1_config(config, state, tool, context, result.print_z);
    let settings = &context.traversal.resolved.views.full;
    let filament = &settings.filament;
    let diameter = at(&filament.gcode.filament_diameter.0, tool).0;
    let absolute = |width| match width {
        crate::FloatOrPercent::Float(value) => value,
        crate::FloatOrPercent::Percent(value) => diameter * value.0 / 100.0,
    };
    let mut width = absolute(settings.process.region.outer_wall_line_width) as f32;
    if width == 0.0 {
        width = absolute(settings.process.object.line_width) as f32;
        if width == 0.0 {
            width = diameter as f32;
        }
    }
    let nozzle = (at(&settings.project.gcode.filament_map.0, tool).0 - 1) as usize;
    let flow = crate::project_slice::perimeters::flow::build_nonbridging_flow(
        crate::FloatOrPercent::Float(f64::from(width)),
        settings.process.object.layer_height.0 as f32,
        at(&settings.project.print.nozzle_diameter.0, nozzle).0 as f32,
    )?;
    let speed = (f64::from(settings.process.region.outer_wall_speed.get_at(nozzle) as f32)
        * flow.mm3_per_mm) as f32;
    config.insert(
        "outer_wall_volumetric_speed",
        Value::number(f64::from(speed.min(
            at(&filament.gcode.filament_max_volumetric_speed.0, tool).0 as f32,
        ))),
    );
    let area = (std::f64::consts::FRAC_PI_4 * diameter.powi(2)) as f32;
    let purge = if f64::from(metadata.purge) < 1e-4 {
        0.0
    } else {
        metadata.purge.max(100.0)
    };
    let length = purge / area;
    for (key, value) in [
        ("x_after_toolchange", f64::from(start[0])),
        ("y_after_toolchange", f64::from(start[1])),
        ("toolchange_z", z),
        ("flush_length", f64::from(length)),
        ("first_flush_volume", f64::from(length / 2.0)),
        ("second_flush_volume", f64::from(length / 2.0)),
    ] {
        config.insert(key, Value::number(value));
    }
    let count = ((purge / 135.0).round() as usize).min(4);
    for i in 0..4 {
        config.insert(
            format!("flush_length_{}", i + 1),
            Value::number(f64::from(if i < count {
                length / count as f32
            } else {
                0.0
            })),
        );
    }
    if !state.extruder_selected {
        config.insert("old_filament_temp", Value::Integer(210));
    }
    let temps = if state.on_first_layer {
        &filament.print.nozzle_temperature_initial_layer.0
    } else {
        &filament.print.nozzle_temperature.0
    };
    let interface = &filament.gcode;
    let temperature = at(&interface.filament_tower_interface_print_temp.0, tool).0;
    let temperature = if temperature == -1 {
        at(&filament.print.nozzle_temperature_range_high.0, tool).0
    } else {
        temperature
    };
    let contact = settings.process.print.enable_tower_interface_features.0 && result.contact;
    config.insert(
        "new_filament_temp",
        Value::Integer(i64::from(if contact {
            temperature
        } else {
            at(temps, tool).0
        })),
    );
    if contact {
        for (key, temperatures) in [
            ("temperature", &filament.print.nozzle_temperature.0),
            (
                "first_layer_temperature",
                &filament.print.nozzle_temperature_initial_layer.0,
            ),
        ] {
            let mut values = temperatures
                .iter()
                .map(|t| Value::Integer(i64::from(t.0)))
                .collect::<Vec<_>>();
            if let Some(value) = values.get_mut(tool) {
                *value = Value::Integer(i64::from(temperature));
            }
            config.insert(key, Value::List(values));
        }
    }
    config.insert(
        "is_prime_tower_interface",
        Value::option_bool(result.contact),
    );
    config.insert(
        "wipe_avoid_perimeter",
        Value::option_bool(settings.process.print.prime_tower_skip_points.0),
    );
    let cooling = &interface.filament_cooling_before_tower;
    config.insert(
        "filament_cooling_before_tower",
        Value::List(
            (0..cooling.len().max(interface.filament_type.0.len()))
                .map(|i| {
                    Value::number(if result.contact || context.layer.index == 0 {
                        0.0
                    } else {
                        match at(cooling, i) {
                            crate::Nullable::Value(value) => value.0,
                            crate::Nullable::Nil => f64::NAN,
                        }
                    })
                })
                .collect(),
        ),
    );
    let tower = state.wipe_tower.as_ref().unwrap();
    let bounds = tower.data.first_layer_bounds.unwrap();
    let offset = [
        tower.position[0] + tower.data.rib_offset[0],
        tower.position[1] + tower.data.rib_offset[1],
    ];
    let min = postprocess::transform(
        [
            scale.unscale(bounds.min().x()) as f32,
            scale.unscale(bounds.min().y()) as f32,
        ],
        offset,
        tower.angle,
    );
    let max = postprocess::transform(
        [
            scale.unscale(bounds.max().x()) as f32,
            scale.unscale(bounds.max().y()) as f32,
        ],
        offset,
        tower.angle,
    );
    let right = max[0] + 3.0;
    let left = min[0] - 3.0;
    let avoid = if right > 100.0 && right < 250.0 {
        right
    } else if left > 100.0 && left < 250.0 {
        left
    } else {
        110.0
    };
    config.insert("wipe_avoid_pos_x", Value::number(f64::from(avoid)));
    Ok(())
}
