//! `GCode.cpp:1118–1396`, `WipeTowerIntegration::append_tcr2`.
use super::super::{
    motion, placeholders, pressure_advance, template, timelapse, tool_transition, value::Value,
};
use super::{filament::at, interface_temperature, postprocess, result::ToolChangeResult};
use crate::SliceError;

pub(super) struct Append<'a> {
    pub(super) result: &'a ToolChangeResult,
    pub(super) new_tool: Option<usize>,
    pub(super) z: Option<f64>,
    pub(super) context: timelapse::Context<'a>,
    pub(super) geometry: motion::LayerGeometry<'a>,
}

pub(super) fn append(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    input: Append<'_>,
) -> Result<(), SliceError> {
    let Append {
        result,
        new_tool,
        z,
        context,
        geometry,
    } = input;
    let tower = state.wipe_tower.as_ref().expect("active tower integration");
    let (position, angle) = (tower.position, tower.angle);
    let (translation, rotation) = if result.priming {
        ([0.0; 2], 0.0)
    } else {
        (position, angle)
    };
    let start = postprocess::transform(result.start, translation, rotation);
    let end = postprocess::transform(result.end, translation, rotation);
    let mut code = postprocess::moves(result, translation, rotation, &tower.offsets, tower.semm);
    motion::tower::unlift(output, state);
    let current_z = state.writer_z.unwrap_or(0.0);
    let z = z.unwrap_or(current_z);
    let needs_change = !state.extruder_selected || new_tool != Some(state.extruders.active_id());
    let will_go_down = (z - current_z).abs() >= 1e-4;
    let settings = &context.traversal.resolved.views.full;
    let ramming = settings.printer.gcode.single_extruder_multi_material.0
        || at(
            &settings.filament.gcode.filament_multitool_ramming.0,
            result.initial_tool,
        )
        .0;
    let should_travel = !result.priming
        && (result.force_travel
            || !needs_change
            || will_go_down
            || ramming
            || settings.printer.gcode.tool_change_on_wipe_tower.0);
    if should_travel || state.layer_change_travel_pending {
        motion::tower::travel_to(output, state, start, geometry)?;
    }
    if will_go_down {
        motion::tower::change_z(
            output,
            state,
            z,
            "Travel down to the last wipe tower layer.",
        );
    }
    let mut change = Vec::new();
    let mut deretraction = Vec::new();
    let mut override_temperature = None;
    if result.priming || (new_tool.is_some() && needs_change) {
        if ramming {
            state.wipe_path.clear();
            state.wipe_start = None;
        }
        let tool = new_tool.expect("priming retains the new tool");
        if settings.process.print.enable_tower_interface_features.0 && result.contact {
            let temperature = at(
                &settings
                    .filament
                    .gcode
                    .filament_tower_interface_print_temp
                    .0,
                tool,
            )
            .0;
            let temperature = if temperature == -1 {
                at(
                    &settings.filament.print.nozzle_temperature_range_high.0,
                    tool,
                )
                .0
            } else {
                temperature
            };
            override_temperature = (temperature > 0).then_some(temperature);
        }
        tool_transition::tower(
            &mut change,
            state,
            context,
            tool_transition::TowerChange {
                filament: tool,
                print_z: f64::from(result.print_z),
                priming: result.priming,
                temperature_override: override_temperature,
            },
        )?;
        if settings.process.print.enable_prime_tower.0 {
            motion::tower::restore_z(&mut deretraction, state, z);
        }
    }
    let mut change = String::from_utf8(change).expect("toolchange G-code is UTF-8");
    if let Some(temperature) = override_temperature {
        let tool = new_tool.expect("temperature override belongs to a toolchange");
        let temperatures = if state.on_first_layer || result.print_z.abs() < 1e-4 {
            &settings.filament.print.nozzle_temperature_initial_layer.0
        } else {
            &settings.filament.print.nozzle_temperature.0
        };
        (change, code) = interface_temperature::rewrite(
            &change,
            &code,
            tool,
            at(temperatures, tool).0,
            temperature,
        );
    }
    let mut config = placeholders::base_config(
        context.traversal,
        context.metadata,
        context.first_layer_bounds,
        Some(&state.template_rng),
        state.total_layer_count,
    )?;
    motion::refresh_writer_state(&mut config, state);
    config.insert("change_filament_gcode", Value::String(change));
    config.insert(
        "deretraction_from_wipe_tower_generator",
        Value::String(String::from_utf8(deretraction).expect("writer G-code is UTF-8")),
    );
    config.insert(
        "layer_num",
        Value::Integer(if result.priming {
            -1
        } else {
            context.layer.index as i64
        }),
    );
    config.insert("layer_z", Value::number(f64::from(result.print_z)));
    config.insert("toolchange_z", Value::number(z));
    let name = "tcr_rotated_gcode";
    let rendered =
        template::render_for_filament(&code, &mut config, new_tool.unwrap_or(usize::MAX))
            .and_then(|text| motion::apply_writer_state(name, &config, state).map(|()| text));
    match rendered {
        Ok(text) => {
            // Config.cpp::unescape_string_cstyle; generated results end in a newline.
            let mut chars = text.chars();
            let mut unescaped = String::with_capacity(text.len());
            while let Some(ch) = chars.next() {
                unescaped.push(if ch == '\\' {
                    match chars.next().expect("tower result ends in newline") {
                        'r' => '\r',
                        'n' => '\n',
                        other => other,
                    }
                } else {
                    ch
                });
            }
            output.extend_from_slice(unescaped.as_bytes());
        }
        Err(error) => {
            state.record_template_failure(name, &error);
            output.extend_from_slice(format!("\n!!!!! Failed to process the custom G-code template {name}\n{error}!!!!! End of an error report for the custom G-code template {name}\n\n\n").as_bytes());
        }
    }
    if let Some(tool) = new_tool {
        pressure_advance::append(output, state, context.traversal, tool);
    }
    let wipe = result
        .wipe_path
        .iter()
        .filter(|_| !will_go_down)
        .map(|&point| postprocess::transform(point, position, angle));
    motion::tower::finish(state, geometry.scale, end, wipe);
    if will_go_down {
        motion::tower::change_z(
            output,
            state,
            current_z,
            "Travel back up to the topmost object layer.",
        );
    }
    Ok(())
}
