use crate::{
    GenerationMetadata, SliceError,
    project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

use super::{footprint, template, value};

mod exhaust_fan;
mod first_filaments;
mod temperature;

/// The first-line M73 placeholder consumed by the G-code processor
/// (`GCode.cpp:2700-2701`).
pub(super) fn append_first_line_m73(output: &mut Vec<u8>) {
    output.extend_from_slice(b"M73 P0 R0\n");
}

pub(super) fn append_limits(output: &mut Vec<u8>, traversal: &PreparedPostClassicTraversal) {
    let flavor = traversal.resolved.views.full.printer.gcode.gcode_flavor;
    // `print_machine_envelope` runs for Marlin-family flavors only, after the
    // machine start G-code (`GCode.cpp:3939-3944`).
    if matches!(
        flavor,
        crate::GCodeFlavor::MarlinLegacy
            | crate::GCodeFlavor::MarlinFirmware
            | crate::GCodeFlavor::RepRapFirmware
    ) && traversal
        .resolved
        .views
        .full
        .printer
        .machine
        .emit_machine_limits_to_gcode
        .0
    {
        append_machine_envelope(output, traversal);
    }
    // Disable fan for printers with an auxiliary fan
    // (`GCode.cpp:2822-2826`: set_fan(0) + set_additional_fan(0)).
    let settings = &traversal.resolved.views.full;
    let close_fan = settings
        .filament
        .print
        .close_fan_the_first_x_layers
        .0
        .first()
        .is_some_and(|value| value.0 > 0);
    if settings.printer.gcode.auxiliary_fan.0 && close_fan {
        output.extend_from_slice(b"M106 S0\nM106 P2 S0\n");
    }
}

fn append_machine_envelope(output: &mut Vec<u8>, traversal: &PreparedPostClassicTraversal) {
    let machine = &traversal.resolved.views.full.printer.machine;
    let flavor = traversal.resolved.views.full.printer.gcode.gcode_flavor;
    let rrf = flavor == crate::GCodeFlavor::RepRapFirmware;
    let factor: f64 = if rrf { 60.0 } else { 1.0 };
    // Upstream converts the envelope integers with `int(value + 0.5)`
    // (`GCode.cpp:3945-3980`).
    let half_up = |value: f64| (value + 0.5).floor() as i64;
    output.extend_from_slice(
        format!(
            "M201 X{} Y{} Z{} E{}\n",
            half_up(first(&machine.machine_max_acceleration_x)),
            half_up(first(&machine.machine_max_acceleration_y)),
            half_up(first(&machine.machine_max_acceleration_z)),
            half_up(first(&machine.machine_max_acceleration_e)),
        )
        .as_bytes(),
    );
    output.extend_from_slice(
        format!(
            "M203 X{} Y{} Z{} E{}\n",
            half_up(first(&machine.machine_max_speed_x) * factor),
            half_up(first(&machine.machine_max_speed_y) * factor),
            half_up(first(&machine.machine_max_speed_z) * factor),
            half_up(first(&machine.machine_max_speed_e) * factor),
        )
        .as_bytes(),
    );
    // Legacy Marlin exports travel acceleration equal to printing
    // acceleration; other flavors use the travel value.
    let travel_acceleration = if flavor == crate::GCodeFlavor::MarlinLegacy {
        first(&machine.machine_max_acceleration_extruding)
    } else {
        first(&machine.machine_max_acceleration_travel)
    };
    if rrf {
        output.extend_from_slice(
            format!(
                "M204 P{} T{} ; sets acceleration (P, T), mm/sec^2\n",
                half_up(first(&machine.machine_max_acceleration_extruding)),
                half_up(travel_acceleration),
            )
            .as_bytes(),
        );
    } else if flavor == crate::GCodeFlavor::MarlinFirmware {
        output.extend_from_slice(
            format!(
                "M204 P{} R{} T{} ; sets acceleration (P, T) and retract acceleration (R), mm/sec^2\n",
                half_up(first(&machine.machine_max_acceleration_extruding)),
                half_up(first(&machine.machine_max_acceleration_retracting)),
                half_up(travel_acceleration),
            )
            .as_bytes(),
        );
    } else {
        output.extend_from_slice(
            format!(
                "M204 P{} R{} T{}\n",
                half_up(first(&machine.machine_max_acceleration_extruding)),
                half_up(first(&machine.machine_max_acceleration_retracting)),
                half_up(travel_acceleration),
            )
            .as_bytes(),
        );
    }
    if rrf {
        output.extend_from_slice(
            format!(
                "M566 X{:.2} Y{:.2} Z{:.2} E{:.2} ; sets the jerk limits, mm/min\n",
                first(&machine.machine_max_jerk_x) * factor,
                first(&machine.machine_max_jerk_y) * factor,
                first(&machine.machine_max_jerk_z) * factor,
                first(&machine.machine_max_jerk_e) * factor,
            )
            .as_bytes(),
        );
    } else {
        output.extend_from_slice(
            format!(
                "M205 X{:.2} Y{:.2} Z{:.2} E{:.2} ; sets the jerk limits, mm/sec\n",
                first(&machine.machine_max_jerk_x),
                first(&machine.machine_max_jerk_y),
                first(&machine.machine_max_jerk_z),
                first(&machine.machine_max_jerk_e),
            )
            .as_bytes(),
        );
    }
    let junction_deviation = first(&machine.machine_max_junction_deviation);
    if flavor == crate::GCodeFlavor::MarlinFirmware && junction_deviation > 0.0 {
        output.extend_from_slice(format!("M205 J{junction_deviation:.3}\n").as_bytes());
    }
}

pub(super) fn first(values: &crate::OrcaFloats) -> f64 {
    values.0.first().map_or(0.0, |value| value.0)
}

/// Transition from the first to the second layer
/// (`GCode.cpp:4777-4830`): power-loss recovery (Marlin only), then the
/// other-layers bed temperature. The acceleration/jerk reset is handled by
/// `begin_layer`, which re-arms the default values for layer two.
pub(super) fn append_second_layer_transition(
    output: &mut Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
    cached_bed_temperature: i32,
) {
    let settings = &traversal.resolved.views.full;
    use crate::ProjectBedType;
    // Nozzle temperature transition (`GCode.cpp:4800-4810`): M104 when the
    // second-layer temperature differs from the first-layer one. The tool
    // parameter is dropped for single-extruder printers
    // (`GCodeWriter.cpp:155-164`).
    for tool in 0..traversal.resolved.logical_filament_count {
        let temperature =
            temperature::filament_int(&settings.filament.print.nozzle_temperature, tool);
        let initial = temperature::filament_int(
            &settings.filament.print.nozzle_temperature_initial_layer,
            tool,
        );
        if temperature > 0 && temperature != initial {
            let tool_suffix = if traversal.resolved.logical_filament_count > 1 {
                format!(" T{tool}")
            } else {
                String::new()
            };
            output.extend_from_slice(format!("M104 S{temperature}{tool_suffix}\n").as_bytes());
        }
    }
    let filament = &settings.filament.print;
    let temps: &[crate::OrcaInts] = match settings.project.print.curr_bed_type {
        ProjectBedType::DefaultPlate => &[],
        ProjectBedType::SupertackPlate => std::slice::from_ref(&filament.supertack_plate_temp),
        ProjectBedType::CoolPlate => std::slice::from_ref(&filament.cool_plate_temp),
        ProjectBedType::EngineeringPlate => std::slice::from_ref(&filament.eng_plate_temp),
        ProjectBedType::HighTempPlate => std::slice::from_ref(&filament.hot_plate_temp),
        ProjectBedType::TexturedPeiPlate => std::slice::from_ref(&filament.textured_plate_temp),
        ProjectBedType::TexturedCoolPlate => {
            std::slice::from_ref(&filament.textured_cool_plate_temp)
        }
    };
    let temps: Vec<i32> = temps
        .iter()
        .flat_map(|values| values.0.iter().map(|value| value.0))
        .collect();
    // `get_highest_bed_temperature` / first-filament selection
    // (`GCode.cpp:4817-4822`). `GCodeWriter::set_bed_temperature` skips the
    // write when unchanged from its zero-initialized cache
    // (`GCodeWriter.cpp:170-175`).
    let bed_temp = match settings.printer.gcode.bed_temperature_formula {
        crate::BedTemperatureFormula::FirstFilament => temps.first().copied().unwrap_or(0),
        crate::BedTemperatureFormula::HighestTemp => temps.iter().copied().max().unwrap_or(0),
    };
    if bed_temp != cached_bed_temperature {
        output.extend_from_slice(format!("M140 S{bed_temp} ; set bed temperature\n").as_bytes());
    }
}

pub(super) fn append_start(
    output: &mut Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
    metadata: GenerationMetadata,
    first_layer_bounds: Option<footprint::FirstLayerBounds>,
) -> Result<(i32, Option<value::Value>), SliceError> {
    let template = &traversal.resolved.views.runtime_gcode.machine_start_gcode.0;
    let (rendered, position) = if template.is_empty() {
        (String::new(), None)
    } else {
        let mut config = self_start_config(traversal, metadata, first_layer_bounds)?;
        let rendered = template::render(template, &mut config).map_err(|error| {
            SliceError::InvalidInput(format!("invalid project G-code template: {error}"))
        })?;
        // `GCode.cpp:3118-3140` lets explicit template assignments update
        // GCodeWriter position, but never parses G0/G1 text back into it.
        (rendered, config.get("position").cloned())
    };
    let bed_cache = temperature::append_startup(output, traversal, &rendered);
    let custom = super::tags::Tags::of(traversal).custom() + "\n";
    output.extend_from_slice(custom.as_bytes());
    temperature::append_chamber_startup(output, traversal, &rendered);
    // `GCodeOutputStream::writeln` (`GCode.cpp:6266-6270`) writes nothing for
    // an empty rendered template and otherwise appends exactly one newline.
    if !rendered.is_empty() {
        output.extend_from_slice(rendered.as_bytes());
        if !rendered.ends_with('\n') {
            output.push(b'\n');
        }
    }
    exhaust_fan::append_print_start(output, traversal);
    if !super::tags::Tags::of(traversal).is_bbl() {
        append_flavor_preamble(output, traversal);
    }
    Ok((bed_cache, position))
}

fn self_start_config(
    traversal: &PreparedPostClassicTraversal,
    metadata: GenerationMetadata,
    first_layer_bounds: Option<footprint::FirstLayerBounds>,
) -> Result<super::value::Config, SliceError> {
    let mut config = super::placeholders::base_config(traversal, metadata, first_layer_bounds)?;
    config.insert("next_extruder", value::Value::number(0.0));
    config.insert("next_hotend", value::Value::number(-1.0));
    config.insert("initial_no_support_extruder", value::Value::number(0.0));
    config.insert("initial_no_support_hotend", value::Value::number(-1.0));
    config.insert(
        "overall_chamber_temperature",
        value::Value::number(f64::from(temperature::overall_chamber_temperature(
            traversal,
        ))),
    );
    config.insert(
        "hold_chamber_temp_for_flat_print",
        value::Value::Bool(false),
    );
    let max_z = traversal
        .objects
        .first()
        .into_iter()
        .flat_map(|object| object.records.iter())
        .filter_map(|record| record.as_ref())
        .map(|record| record.layer_height)
        .sum::<f64>();
    config.insert("max_print_z", value::Value::number(max_z));
    config.insert("max_layer_z", value::Value::number(max_z));
    config.insert(
        "total_layer_count",
        value::Value::number(
            traversal
                .objects
                .first()
                .map_or(0, |object| object.records.len()) as f64,
        ),
    );
    config.insert("layer_num", value::Value::number(0.0));
    if let Some((min_x, min_y, size_x, size_y)) = first_layer_bounds {
        config.insert(
            "first_layer_print_min",
            value::Value::List(vec![
                value::Value::number(min_x),
                value::Value::number(min_y),
            ]),
        );
        config.insert(
            "first_layer_print_size",
            value::Value::List(vec![
                value::Value::number(size_x),
                value::Value::number(size_y),
            ]),
        );
    }
    let (first_filaments, first_non_support_filaments) = first_filaments::resolve(traversal);
    config.insert("first_tools", first_filaments.clone());
    config.insert("first_filaments", first_filaments);
    config.insert(
        "first_non_support_tools",
        first_non_support_filaments.clone(),
    );
    config.insert("first_non_support_filaments", first_non_support_filaments);
    Ok(config)
}

/// `GCodeWriter::preamble` (`GCodeWriter.cpp:82-104`): absolute XYZ
/// coordinates, millimeters, and the extruder distance mode for the
/// flavors that support it. Compatible printers emit it after machine start;
/// BBL delays it until after filament start and initial pressure advance.
pub(super) fn append_flavor_preamble(
    output: &mut Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
) {
    use crate::options::GCodeFlavor;
    let gcode = &traversal.resolved.views;
    output.extend_from_slice(b"G90\nG21\n");
    let marlin_family = matches!(
        gcode.full.printer.gcode.gcode_flavor,
        GCodeFlavor::MarlinLegacy
            | GCodeFlavor::MarlinFirmware
            | GCodeFlavor::Klipper
            | GCodeFlavor::RepRapSprinter
            | GCodeFlavor::RepRapFirmware
            | GCodeFlavor::Repetier
            | GCodeFlavor::Teacup
    );
    if marlin_family {
        if gcode.runtime_gcode.use_relative_e_distances.0 {
            output.extend_from_slice(b"M83 ; use relative distances for extrusion\n");
        } else {
            output.extend_from_slice(b"M82 ; use absolute distances for extrusion\nG92 E0\n");
        }
    }
}
