use crate::{
    GenerationMetadata, SliceError,
    project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

use super::{footprint, motion, template};

mod envelope;
mod exhaust_fan;
pub(super) mod first_filaments;
mod start_config;
mod temperature;

/// The first-layer nozzle temperature of one filament
/// (`_print_first_layer_extruder_temperatures`, `GCode.cpp:3338`).
pub(in crate::project_slice::gcode_emit) fn initial_layer_nozzle_temperature(
    settings: &crate::ProjectSettings,
    tool: usize,
) -> i32 {
    temperature::filament_int(
        &settings.filament.print.nozzle_temperature_initial_layer,
        tool,
    )
}

/// The first-layer bed temperature
/// (`_print_first_layer_bed_temperature`, `GCode.cpp:3337`).
pub(in crate::project_slice::gcode_emit) fn first_layer_bed_temperature(
    traversal: &PreparedPostClassicTraversal,
    first_printing_filament: usize,
    first_layer_tools: &[usize],
) -> i32 {
    temperature::first_layer_bed_temperature(traversal, first_printing_filament, first_layer_tools)
}

/// The first-line M73 placeholder consumed by the G-code processor
/// (`GCode.cpp:2700-2701`).
pub(super) fn append_first_line_m73(output: &mut Vec<u8>) {
    output.extend_from_slice(b"M73 P0 R0\n");
}

pub(super) fn append_limits(
    output: &mut Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
    initial_extruder_id: usize,
    used_filaments: &[usize],
) {
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
        envelope::append_machine_envelope(output, traversal, used_filaments);
    }
    // Disable fan for printers with an auxiliary fan
    // (`GCode.cpp:2822-2826`, `Config.hpp:624-627`: get_at(initial_extruder_id)).
    let settings = &traversal.resolved.views.full;
    let close_fan = &settings.filament.print.close_fan_the_first_x_layers.0;
    if settings.printer.gcode.auxiliary_fan.0
        && close_fan
            .get(initial_extruder_id)
            .unwrap_or(&close_fan[0])
            .0
            != 0
    {
        let comments = settings.process.print.gcode_comments.0;
        let fan_comment = if comments { " ; disable fan" } else { "" };
        let aux_comment = if comments {
            " ; disable additional fan "
        } else {
            ""
        };
        output.extend_from_slice(
            format!("M106 S0{fan_comment}\nM106 P2 S0{aux_comment}\n").as_bytes(),
        );
    }
}

/// Transition from the first to the second layer
/// (`GCode.cpp:4777-4830`): power-loss recovery (Marlin only), then the
/// other-layers bed temperature. The acceleration/jerk reset is handled by
/// `begin_layer`, which re-arms the default values for layer two.
pub(super) fn append_second_layer_transition(
    output: &mut Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
    state: &motion::EmitState,
    bed_cache: &mut i32,
) {
    let settings = &traversal.resolved.views.full;
    use crate::ProjectBedType;
    // Nozzle temperature transition (`GCode.cpp:4800-4812`): M104 when the
    // second-layer temperature differs from the first-layer one. The tool
    // parameter is dropped for single-extruder printers
    // (`GCodeWriter.cpp:155-164`); under SEMM (or ooze prevention) upstream
    // sets the temperature for the CURRENT extruder only, skipping the
    // rest (`GCode.cpp:4804-4807`).
    let runtime = &traversal.resolved.views.runtime_gcode;
    let single_extruder_multi_material = runtime.single_extruder_multi_material.0;
    // `GCodeWriter::multiple_extruders` (GCodeWriter.cpp:78) is true when a
    // used extruder id exceeds zero — the writer then keeps the tool
    // suffix (`GCodeWriter.cpp:159-164`).
    let multiple_extruders = state.extruders.multiple() && !single_extruder_multi_material;
    let ooze_prevention = settings.process.print.ooze_prevention.0;
    for tool in 0..traversal.resolved.logical_filament_count {
        if (single_extruder_multi_material || ooze_prevention)
            && tool != state.extruders.active_id()
        {
            continue;
        }
        let temperature =
            temperature::filament_int(&settings.filament.print.nozzle_temperature, tool);
        let initial = temperature::filament_int(
            &settings.filament.print.nozzle_temperature_initial_layer,
            tool,
        );
        if temperature > 0 && temperature != initial {
            let tool_suffix = if multiple_extruders {
                format!(" T{tool}")
            } else {
                String::new()
            };
            // `GCodeWriter::set_temperature` defaults the comment to "set
            // nozzle temperature" for the non-waiting M104 emission
            // (`GCodeWriter.cpp:133`).
            output.extend_from_slice(
                format!("M104 S{temperature}{tool_suffix} ; set nozzle temperature\n").as_bytes(),
            );
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
    // `GCodeWriter::set_bed_temperature` (`GCodeWriter.cpp:168-175`)
    // refreshes the writer cache on every changed value, so the next
    // sequential-object transition compares against the live value.
    if bed_temp != *bed_cache {
        output.extend_from_slice(format!("M140 S{bed_temp} ; set bed temperature\n").as_bytes());
        *bed_cache = bed_temp;
    }
}

pub(super) fn append_start(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    traversal: &PreparedPostClassicTraversal,
    metadata: GenerationMetadata,
    first_layer_bounds: Option<footprint::FirstLayerBounds>,
) -> Result<i32, SliceError> {
    let template = &traversal.resolved.views.runtime_gcode.machine_start_gcode.0;
    let rendered = if template.is_empty() {
        String::new()
    } else {
        let mut config =
            start_config::self_start_config(traversal, state, metadata, first_layer_bounds)?;
        let name = "machine_start_gcode";
        let result =
            template::render_for_filament(template, &mut config, state.initial_filaments.initial())
                .map_err(|error| error.to_string())
                .and_then(|rendered| {
                    motion::apply_writer_state(name, &config, &rendered, state).map(|()| rendered)
                });
        match result {
            Ok(rendered) => rendered,
            Err(error) => {
                state.record_template_failure(name, &error);
                format!(
                    "\n!!!!! Failed to process the custom G-code template {name}\n{error}!!!!! End of an error report for the custom G-code template {name}\n\n\n"
                )
            }
        }
    };
    state.start_gcode_filament = super::tool_transition::last_start_filament(&rendered);
    let bed_cache = temperature::append_startup(
        output,
        traversal,
        &rendered,
        state.initial_filaments.initial_bed_temperature(),
    );
    let custom = super::tags::Tags::of(traversal).custom() + "\n";
    output.extend_from_slice(custom.as_bytes());
    temperature::append_chamber_startup(output, traversal, &rendered, state.extruders.ids());
    // `GCodeOutputStream::writeln` (`GCode.cpp:6266-6270`) writes nothing for
    // an empty rendered template and otherwise appends exactly one newline.
    if !rendered.is_empty() {
        output.extend_from_slice(rendered.as_bytes());
        if !rendered.ends_with('\n') {
            output.push(b'\n');
        }
    }
    exhaust_fan::append_print_start(output, traversal, state.extruders.ids());
    if !super::tags::Tags::of(traversal).is_bbl() {
        append_flavor_preamble(output, traversal);
    }
    Ok(bed_cache)
}

pub(super) fn append_completion_controls(
    output: &mut Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
    state: &motion::EmitState,
) {
    temperature::append_chamber_shutdown(output, traversal, state.extruders.ids());
    exhaust_fan::append_print_end(output, traversal, state.extruders.ids());
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
            output.extend_from_slice(b"M82 ; use absolute distances for extrusion\nG92 E0");
            if gcode.full.process.print.gcode_comments.0 {
                output.extend_from_slice(b" ; reset extrusion distance");
            }
            output.push(b'\n');
        }
    }
}
