use crate::{
    GenerationMetadata, SliceError,
    project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

use super::{footprint, template, value};

pub(super) fn append_limits(output: &mut Vec<u8>, traversal: &PreparedPostClassicTraversal) {
    let machine = &traversal.resolved.views.full.printer.machine;
    let travel_acceleration = if traversal.resolved.views.full.printer.gcode.gcode_flavor
        == crate::GCodeFlavor::MarlinLegacy
    {
        machine.machine_max_acceleration_extruding.clone()
    } else {
        machine.machine_max_acceleration_travel.clone()
    };
    output.extend_from_slice(b"M73 P0 R0\n");
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
            first(&machine.machine_max_speed_x),
            first(&machine.machine_max_speed_y),
            first(&machine.machine_max_speed_z),
            first(&machine.machine_max_speed_e),
        )
        .as_bytes(),
    );
    output.extend_from_slice(
        format!(
            "M204 P{} R{} T{}\n",
            first(&machine.machine_max_acceleration_extruding),
            first(&machine.machine_max_acceleration_retracting),
            first(&travel_acceleration),
        )
        .as_bytes(),
    );
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
    let junction_deviation = first(&machine.machine_max_junction_deviation);
    if traversal.resolved.views.full.printer.gcode.gcode_flavor
        == crate::GCodeFlavor::MarlinFirmware
        && junction_deviation > 0.0
    {
        output.extend_from_slice(format!("M205 J{junction_deviation:.3}\n").as_bytes());
    }
    (super::tags::Tags::of(traversal).is_bbl())
        .then(|| output.extend_from_slice(b"M106 S0\nM106 P2 S0\n"));
}

pub(super) fn first(values: &crate::OrcaFloats) -> f64 {
    values.0.first().map_or(0.0, |value| value.0)
}

pub(super) fn append_start(
    output: &mut Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
    metadata: GenerationMetadata,
) -> Result<(), SliceError> {
    let template = &traversal.resolved.views.runtime_gcode.machine_start_gcode.0;
    if template.is_empty() {
        return Ok(());
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
    Ok(())
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
