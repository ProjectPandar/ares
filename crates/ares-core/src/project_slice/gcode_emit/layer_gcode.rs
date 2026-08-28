//! Per-layer emission helpers split from the emit driver: fan allowance,
//! print preamble, and the layer-change G-code templates.

use crate::GenerationMetadata;

use super::super::gcode_emit::{SliceError, extruders, tags, template, value};
use super::PreparedPostClassicTraversal;

pub(super) struct LayerChangeTemplate {
    max_additional_fan: f64,
    metadata: GenerationMetadata,
}

impl LayerChangeTemplate {
    pub(super) fn new(
        traversal: &PreparedPostClassicTraversal,
        metadata: GenerationMetadata,
    ) -> Self {
        Self {
            max_additional_fan: max_additional_fan(traversal),
            metadata,
        }
    }
}

fn max_additional_fan(traversal: &PreparedPostClassicTraversal) -> f64 {
    let used = extruders::collect_project_object_extruders(
        traversal.project.objects(),
        &traversal.resolved.objects,
        traversal.resolved.logical_filament_count,
    );
    let speeds = &traversal
        .resolved
        .views
        .full
        .filament
        .print
        .additional_cooling_fan_speed
        .0;
    used.into_iter()
        .flatten()
        .filter_map(|extruder| speeds.get(extruder).or_else(|| speeds.first()))
        .map(|speed| speed.0)
        .max()
        .map_or(0.0, f64::from)
}
pub(super) fn append_print_preamble(
    output: &mut Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
    metadata: GenerationMetadata,
) -> Result<(), SliceError> {
    if tags::Tags::of(traversal).is_bbl() {
        output.extend_from_slice(
            b"; filament start gcode\n;VT0\nG90\nG21\nM83 ; use relative distances for extrusion\n",
        );
        output.extend_from_slice(b"M981 S1 P20000 ;open spaghetti detector\nM106 S0\nM106 P2 S0\n");
        return Ok(());
    }
    // Compatible flavor renders the preset's filament start template
    // (`GCode.cpp` filament-start handling).
    if let Some(source) = traversal
        .resolved
        .views
        .runtime_gcode
        .filament_start_gcode
        .0
        .first()
    {
        let mut config = super::placeholders::base_config(traversal, metadata)?;
        config.insert("position", emitted_position(output));
        let rendered = template::render(source, &config).map_err(|error| {
            SliceError::InvalidInput(format!(
                "invalid project filament-start G-code template: {error}"
            ))
        })?;
        output.extend_from_slice(rendered.as_bytes());
        if !rendered.ends_with('\n') {
            output.push(b'\n');
        }
    }
    // Orca: set the initial pressure advance for the first filament
    // (`GCode.cpp:8048-8052`, `GCodeWriter.cpp:370-391`).
    let settings = &traversal.resolved.views.full;
    let enabled = settings
        .filament
        .gcode
        .enable_pressure_advance
        .0
        .first()
        .is_some_and(|value| value.0);
    if enabled {
        let pa = settings
            .filament
            .gcode
            .pressure_advance
            .0
            .first()
            .map_or(0.0, |value| value.0);
        if pa >= 0.0 {
            let value = format_pa(pa);
            let line = match settings.printer.gcode.gcode_flavor {
                crate::GCodeFlavor::Klipper => {
                    format!(
                        "SET_PRESSURE_ADVANCE ADVANCE={value}; Override pressure advance value\n"
                    )
                }
                crate::GCodeFlavor::RepRapFirmware => {
                    format!("M572 D0 S{value}; Override pressure advance value\n")
                }
                crate::GCodeFlavor::Repetier => {
                    format!("M233 X{value} Y{value} ; Override pressure advance value\n")
                }
                _ if tags::Tags::of(traversal).is_bbl() => {
                    format!("M900 K{value} L1000 M10 ; Override pressure advance value\n")
                }
                _ => format!("M900 K{value}; Override pressure advance value\n"),
            };
            output.extend_from_slice(line.as_bytes());
        }
    }
    Ok(())
}

/// `std::setprecision(4)`: four significant digits, trailing zeros trimmed
/// by the ostream default formatting.
fn format_pa(value: f64) -> String {
    if value == 0.0 {
        return "0".to_owned();
    }
    let magnitude = value.abs().log10().floor() as i32;
    let decimals = (3 - magnitude).max(0) as usize;
    let formatted = format!("{value:.decimals$}");
    formatted
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_owned()
}

pub(super) fn emitted_position(output: &[u8]) -> value::Value {
    let mut position = [0.0_f64; 3];
    let mut absolute = true;
    for line in String::from_utf8_lossy(output).lines() {
        let code = line.split_once(';').map_or(line, |(code, _)| code).trim();
        match code {
            "G90" => {
                absolute = true;
                continue;
            }
            "G91" => {
                absolute = false;
                continue;
            }
            "G28" => {
                position = [0.0; 3];
                continue;
            }
            _ => {}
        }
        let Some(command) = code.split_ascii_whitespace().next() else {
            continue;
        };
        if !matches!(command, "G0" | "G1" | "G2" | "G3" | "G92") {
            continue;
        }
        for token in code.split_ascii_whitespace().skip(1) {
            let Some(axis) = token.bytes().next() else {
                continue;
            };
            let Some(index) = [b'X', b'Y', b'Z']
                .iter()
                .position(|candidate| *candidate == axis)
            else {
                continue;
            };
            let Ok(value) = token[1..].parse::<f64>() else {
                continue;
            };
            if absolute || command == "G92" {
                position[index] = value;
            } else {
                position[index] += value;
            }
        }
    }
    value::Value::List(position.into_iter().map(value::Value::number).collect())
}

/// Renders `before_layer_change_gcode` with `layer_num`, `layer_z`, and
/// `max_layer_z` in scope (`GCode.cpp:4631-4641`).
pub(super) fn append_before_layer_change_gcode(
    output: &mut Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
    layer_index: usize,
    layer_z: f64,
    metadata: GenerationMetadata,
) -> Result<(), SliceError> {
    let template = traversal
        .resolved
        .views
        .runtime_gcode
        .before_layer_change_gcode
        .0
        .as_str();
    if template.is_empty() {
        return Ok(());
    }
    let mut config = super::placeholders::base_config(traversal, metadata)?;
    config.insert("layer_num", value::Value::number((layer_index + 1) as f64));
    config.insert("layer_z", value::Value::number(layer_z));
    let filament = &traversal.resolved.views.full.filament.gcode;
    let diameter = filament
        .filament_diameter
        .0
        .first()
        .map_or(1.75, |diameter| diameter.0);
    let density = filament
        .filament_density
        .0
        .first()
        .map_or(1.24, |density| density.0);
    let length = super::finish::account_used_filament(output);
    let volume = length * diameter.powi(2) * 0.25 * std::f64::consts::PI;
    config.insert("extruded_volume_total", value::Value::number(volume));
    config.insert(
        "extruded_weight_total",
        value::Value::number(volume * density * 0.001),
    );
    config.insert("max_layer_z", value::Value::number(layer_z));
    let rendered = template::render(template, &config).map_err(|error| {
        SliceError::InvalidInput(format!(
            "invalid project before-layer-change G-code template: {error}"
        ))
    })?;
    output.extend_from_slice(rendered.as_bytes());
    output.push(b'\n');
    Ok(())
}

pub(super) fn append_layer_change(
    output: &mut Vec<u8>,
    traversal: &PreparedPostClassicTraversal,
    layer_index: usize,
    layer_z: f64,
    context: &LayerChangeTemplate,
) -> Result<(), SliceError> {
    let template = &traversal.resolved.views.runtime_gcode.layer_change_gcode.0;
    if !template.is_empty() {
        let mut config = super::placeholders::base_config(traversal, context.metadata)?;
        config.insert("current_extruder", value::Value::number(0.0));
        config.insert("layer_num", value::Value::number(layer_index as f64));
        config.insert("layer_z", value::Value::number(layer_z));
        config.insert("overall_chamber_temperature", value::Value::number(0.0));
        if let Some(value) = config
            .get("temperature_vitrification")
            .and_then(|value| value.index(0))
            .cloned()
        {
            config.insert("min_vitrification_temperature", value);
        }
        config.insert(
            "max_additional_fan",
            value::Value::number(context.max_additional_fan),
        );
        // The upstream parser exposes the running extrusion totals to the
        // layer-change template as well (`GCode.cpp:1652, 1689`); they are
        // re-scanned from the emitted G-code, which is the accumulated
        // extrusion of every layer printed so far.
        let filament = &traversal.resolved.views.full.filament.gcode;
        let diameter = filament
            .filament_diameter
            .0
            .first()
            .map_or(1.75, |diameter| diameter.0);
        let density = filament
            .filament_density
            .0
            .first()
            .map_or(1.24, |density| density.0);
        let length = super::finish::account_used_filament(output);
        let volume = length * diameter.powi(2) * 0.25 * std::f64::consts::PI;
        config.insert("extruded_volume_total", value::Value::number(volume));
        config.insert(
            "extruded_weight_total",
            value::Value::number(volume * density * 0.001),
        );
        let rendered = template::render(template, &config).map_err(|error| {
            SliceError::InvalidInput(format!(
                "invalid project layer-change G-code template: {error}"
            ))
        })?;
        output.extend_from_slice(rendered.as_bytes());
        output.push(b'\n');
    }
    output.extend_from_slice(b";_SET_FAN_SPEED_CHANGING_LAYER\n");
    Ok(())
}
