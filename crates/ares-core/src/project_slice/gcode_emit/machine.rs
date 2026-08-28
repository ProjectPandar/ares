use crate::{
    GenerationMetadata, SliceError,
    project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

use super::{footprint, template, value};

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
    (super::tags::Tags::of(traversal).is_bbl())
        .then(|| output.extend_from_slice(b"M106 S0\nM106 P2 S0\n"));
}

fn append_machine_envelope(output: &mut Vec<u8>, traversal: &PreparedPostClassicTraversal) {
    let machine = &traversal.resolved.views.full.printer.machine;
    let flavor = traversal.resolved.views.full.printer.gcode.gcode_flavor;
    let rrf = flavor == crate::GCodeFlavor::RepRapFirmware;
    let factor: f64 = if rrf { 60.0 } else { 1.0 };
    output.extend_from_slice(
        format!(
            "M201 X{} Y{} Z{} E{}\n",
            first(&machine.machine_max_acceleration_x),
            first(&machine.machine_max_acceleration_y),
            first(&machine.machine_max_acceleration_z),
            first(&machine.machine_max_acceleration_e),
        )
        .as_bytes(),
    );
    output.extend_from_slice(
        format!(
            "M203 X{} Y{} Z{} E{}\n",
            first(&machine.machine_max_speed_x) * factor,
            first(&machine.machine_max_speed_y) * factor,
            first(&machine.machine_max_speed_z) * factor,
            first(&machine.machine_max_speed_e) * factor,
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
                first(&machine.machine_max_acceleration_extruding),
                travel_acceleration,
            )
            .as_bytes(),
        );
    } else if flavor == crate::GCodeFlavor::MarlinFirmware {
        output.extend_from_slice(
            format!(
                "M204 P{} R{} T{} ; sets acceleration (P, T) and retract acceleration (R), mm/sec^2\n",
                first(&machine.machine_max_acceleration_extruding),
                first(&machine.machine_max_acceleration_retracting),
                travel_acceleration,
            )
            .as_bytes(),
        );
    } else {
        output.extend_from_slice(
            format!(
                "M204 P{} R{} T{}\n",
                first(&machine.machine_max_acceleration_extruding),
                first(&machine.machine_max_acceleration_retracting),
                travel_acceleration,
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
) -> Result<i32, SliceError> {
    let template = &traversal.resolved.views.runtime_gcode.machine_start_gcode.0;
    if template.is_empty() {
        return Ok(0);
    }
    let mut config = super::placeholders::base_config(traversal, metadata);
    config.insert("next_extruder", value::Value::number(0.0));
    config.insert("next_hotend", value::Value::number(-1.0));
    config.insert("initial_no_support_extruder", value::Value::number(0.0));
    config.insert("initial_no_support_hotend", value::Value::number(-1.0));
    config.insert("overall_chamber_temperature", value::Value::number(0.0));
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
    if let Some((min_x, min_y, size_x, size_y)) = footprint::first_layer_bounds(traversal) {
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
    let filament_count = traversal.resolved.logical_filament_count;
    let first_filaments = value::Value::List(
        (0..filament_count)
            .map(|index| value::Value::number(index as f64))
            .collect(),
    );
    config.insert("first_non_support_filaments", first_filaments.clone());
    config.insert("first_filaments", first_filaments);
    let custom = super::tags::Tags::of(traversal).custom() + "\n";
    output.extend_from_slice(custom.as_bytes());
    let bed_cache = append_first_layer_bed_temperature(output, traversal);
    let rendered = template::render(template, &config).map_err(|error| {
        SliceError::InvalidInput(format!("invalid project G-code template: {error}"))
    })?;
    output.extend_from_slice(rendered.as_bytes());
    if !rendered.ends_with('\n') {
        output.push(b'\n');
    }
    if !super::tags::Tags::of(traversal).is_bbl() {
        append_flavor_preamble(output, traversal);
    }
    Ok(bed_cache)
}

/// `_print_first_layer_bed_temperature` (`GCode.cpp:4023-4048`, called at
/// `GCode.cpp:3120-3124` for non-Klipper flavors): the bed temperature is
/// always pushed into the writer cache, but only written when the custom
/// start G-code does not set it itself. Returns the cached bed temperature.
fn append_first_layer_bed_temperature(
    output: &mut Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
) -> i32 {
    let settings = &traversal.resolved.views.full;
    if settings.printer.gcode.gcode_flavor == crate::GCodeFlavor::Klipper {
        return 0;
    }
    let first = first_layer_bed_temperature(traversal);
    let template = &traversal.resolved.views.runtime_gcode.machine_start_gcode.0;
    if !template.contains("M140") && !template.contains("M190") {
        output.extend_from_slice(
            format!("M190 S{first} ; set bed temperature and wait for it to be reached\n")
                .as_bytes(),
        );
    }
    first
}

fn first_layer_bed_temperature(traversal: &PreparedPostClassicTraversal) -> i32 {
    let settings = &traversal.resolved.views.full;
    use crate::ProjectBedType;
    let filament = &settings.filament.print;
    let temps: &[crate::OrcaInts] = match settings.project.print.curr_bed_type {
        ProjectBedType::DefaultPlate => &[],
        ProjectBedType::SupertackPlate => {
            std::slice::from_ref(&filament.supertack_plate_temp_initial_layer)
        }
        ProjectBedType::CoolPlate => std::slice::from_ref(&filament.cool_plate_temp_initial_layer),
        ProjectBedType::EngineeringPlate => {
            std::slice::from_ref(&filament.eng_plate_temp_initial_layer)
        }
        ProjectBedType::HighTempPlate => {
            std::slice::from_ref(&filament.hot_plate_temp_initial_layer)
        }
        ProjectBedType::TexturedPeiPlate => {
            std::slice::from_ref(&filament.textured_plate_temp_initial_layer)
        }
        ProjectBedType::TexturedCoolPlate => {
            std::slice::from_ref(&filament.textured_cool_plate_temp_initial_layer)
        }
    };
    let temps: Vec<i32> = temps
        .iter()
        .flat_map(|values| values.0.iter().map(|value| value.0))
        .collect();
    match settings.printer.gcode.bed_temperature_formula {
        crate::BedTemperatureFormula::FirstFilament => temps.first().copied().unwrap_or(0),
        crate::BedTemperatureFormula::HighestTemp => temps.iter().copied().max().unwrap_or(0),
    }
}

/// `GCodeWriter::preamble` (`GCodeWriter.cpp:82-104`): absolute XYZ
/// coordinates, millimeters, and the extruder distance mode for the
/// flavors that support it. BBL machines carry their own sequence in the
/// filament start block instead.
fn append_flavor_preamble(output: &mut Vec<u8>, traversal: &PreparedPostClassicTraversal) {
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
            output.extend_from_slice(b"M82 ; use absolute distances for extrusion\n");
        }
    }
}
