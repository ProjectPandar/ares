//! Per-layer emission helpers split from the emit driver: fan allowance,
//! print preamble, and the layer-change G-code templates.

use crate::GenerationMetadata;

use super::super::gcode_emit::{SliceError, extruders, tags, template, value};
use super::PreparedPostClassicTraversal;

pub(super) struct LayerChangeTemplate {
    max_additional_fan: f64,
    metadata: GenerationMetadata,
    first_layer_bounds: Option<super::footprint::FirstLayerBounds>,
}

impl LayerChangeTemplate {
    pub(super) fn new(
        traversal: &PreparedPostClassicTraversal,
        metadata: GenerationMetadata,
        first_layer_bounds: Option<super::footprint::FirstLayerBounds>,
    ) -> Self {
        Self {
            max_additional_fan: max_additional_fan(traversal),
            metadata,
            first_layer_bounds,
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
    state: &mut super::motion::EmitState,
    traversal: &PreparedPostClassicTraversal,
    metadata: GenerationMetadata,
    start_position: Option<&value::Value>,
    first_layer_bounds: Option<super::footprint::FirstLayerBounds>,
) -> Result<(), SliceError> {
    let is_bbl = tags::Tags::of(traversal).is_bbl();
    // BBL's `init_extruder` already selected the initial extruder
    // BEFORE its filament-start render (`GCode.cpp:3144-3148`), so
    // `position` write-backs carry the extruder offset; the non-BBL
    // extruder is selected inside `set_extruder` AFTER the render
    // (`GCode.cpp:7736-7746`), so its write-back omits the offset.
    state.extruder_selected = is_bbl;
    // BBL renders this explicitly before `;VT` (`GCode.cpp:3143-3154`);
    // compatible printers render it from the initial set_extruder call
    // (`GCode.cpp:7710-7745`).
    // compatible printers render it from the initial set_extruder call
    // (`GCode.cpp:7710-7745`).
    if let Some(source) = traversal
        .resolved
        .views
        .runtime_gcode
        .filament_start_gcode
        .0
        .first()
        // `GCode.cpp:7710-7745` processes this through PlaceholderParser;
        // its skipper (`PlaceholderParser.cpp:2510-2520`) reduces Orca's
        // whitespace-only default sentinel to no output and no newline.
        .filter(|source| !source.trim().is_empty())
    {
        // Machine-start's `position` assignment was already written back
        // into writer state by the caller (`gcode_emit.rs`), so the
        // refresh below seeds upstream's post-assignment `m_pos`.
        let _ = start_position;
        let mut config = super::placeholders::base_config(traversal, metadata, first_layer_bounds)?;
        super::motion::refresh_writer_state(&mut config, state);
        let name = "filament_start_gcode";
        // Render and write-back failures both emit only the inline
        // report (`GCode.cpp:3920-3933`).
        let mut failure_error = String::new();
        let failed = match template::render(source, &mut config) {
            Err(error) => {
                failure_error = error.to_string();
                state.record_template_failure(name, &failure_error);
                true
            }
            Ok(rendered) => match super::motion::apply_writer_state(name, &config, state) {
                Ok(()) => {
                    output.extend_from_slice(rendered.as_bytes());
                    if !rendered.ends_with('\n') {
                        output.push(b'\n');
                    }
                    false
                }
                Err(error) => {
                    failure_error = error;
                    true
                }
            },
        };
        if failed {
            let error = failure_error;
            output.extend_from_slice(
                format!(
                    "\n!!!!! Failed to process the custom G-code template {name}\n{error}!!!!! End of an error report for the custom G-code template {name}\n\n\n"
                )
                .as_bytes(),
            );
        }
    }
    // `set_extruder` finishes selecting the extruder once
    // filament-start rendered (`GCode.cpp:7746`): later template
    // write-backs (layer/before/after) apply the extruder offset.
    state.extruder_selected = true;
    if is_bbl {
        output.extend_from_slice(b";VT0\n");
    }
    // GCodeWriter::toolchange also emits the initial selection for a
    // non-BBL multi-filament configuration, even if only filament0 is used.
    let settings = &traversal.resolved.views.full;
    if !is_bbl && settings.filament.gcode.filament_diameter.0.len() > 1 {
        let runtime = &traversal.resolved.views.runtime_gcode;
        let prefix = if runtime.manual_filament_change.0 {
            "; MANUAL_TOOL_CHANGE T"
        } else {
            match runtime.gcode_flavor {
                crate::GCodeFlavor::MakerWare => "M135 T",
                crate::GCodeFlavor::Sailfish => "M108 T",
                _ => "T",
            }
        };
        let comments = settings.process.print.gcode_comments.0;
        let comment = if comments { " ; change extruder" } else { "" };
        output.extend_from_slice(format!("{prefix}0{comment}\n").as_bytes());
        if runtime.gcode_flavor.resets_absolute_e() {
            state.e_position = 0.0;
            state.private_e_position = 0.0;
        }
        if runtime.gcode_flavor.resets_absolute_e() && !runtime.use_relative_e_distances.0 {
            let comment = if comments {
                " ; reset extrusion distance"
            } else {
                ""
            };
            output.extend_from_slice(format!("G92 E0{comment}\n").as_bytes());
        }
    }
    // Orca: set the initial pressure advance for the first filament
    // (`GCode.cpp:8048-8052`, `GCodeWriter.cpp:370-391`).
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
                _ if is_bbl => {
                    format!("M900 K{value} L1000 M10 ; Override pressure advance value\n")
                }
                _ => format!("M900 K{value}; Override pressure advance value\n"),
            };
            output.extend_from_slice(line.as_bytes());
        }
    }
    if is_bbl {
        super::machine::append_flavor_preamble(output, traversal);
        output.extend_from_slice(b"M981 S1 P20000 ;open spaghetti detector\n");
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

pub(super) struct ExtrusionTotals {
    pub(super) diameter: f64,
    pub(super) density: f64,
    pub(super) used_filament: f64,
}

pub(super) struct LayerTemplateContext<'a> {
    pub(super) traversal: &'a PreparedPostClassicTraversal,
    pub(super) layer_index: usize,
    pub(super) layer_z: f64,
    /// `m_max_layer_z` (`GCode.cpp:4638`): running max of the OFFSET
    /// writer z (`print_z + z_offset`) — layers ascend, so this equals
    /// the current layer's offset z.
    pub(super) max_layer_z: f64,
    pub(super) totals: ExtrusionTotals,
    pub(super) context: &'a LayerChangeTemplate,
}

/// Renders `before_layer_change_gcode` with `layer_num`, `layer_z`, and
/// `max_layer_z` in scope (`GCode.cpp:4631-4641`).
pub(super) fn append_before_layer_change_gcode(
    output: &mut Vec<u8>,
    state: &mut super::motion::EmitState,
    template_context: LayerTemplateContext<'_>,
) -> Result<(), SliceError> {
    let LayerTemplateContext {
        traversal,
        layer_index,
        layer_z,
        max_layer_z,
        totals,
        context,
    } = template_context;
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
    if template.trim().is_empty() {
        output.push(b'\n');
        return Ok(());
    }
    let mut config =
        super::placeholders::base_config(traversal, context.metadata, context.first_layer_bounds)?;
    // `ppi.update_from_gcodewriter(m_writer)` refreshes `position`/
    // E-counters before each template (`GCode.cpp:3896`).
    super::motion::refresh_writer_state(&mut config, state);
    config.insert("layer_num", value::Value::number(layer_index as f64));
    config.insert("layer_z", value::Value::number(layer_z));
    // Orca's extruded_volume_total is `Extruder::used_filament() *
    // filament_crossection()` — the writer's cumulative extrusion, not a
    // G-code re-scan that would include template output.
    let volume = totals.used_filament * totals.diameter.powi(2) * 0.25 * std::f64::consts::PI;
    config.insert("extruded_volume_total", value::Value::number(volume));
    config.insert(
        "extruded_weight_total",
        value::Value::number(volume * totals.density * 0.001),
    );
    config.insert("max_layer_z", value::Value::number(max_layer_z));
    // Render or write-back failure both reduce to the inline report —
    // the processed text is discarded (`GCode.cpp:3920-3933` returns the
    // block instead of the template output; `slic3r_errors` via
    // `record_template_failure` abort the export at the end).
    let name = "before_layer_change_gcode";
    let mut failure_error = String::new();
    let rendered = match template::render(template, &mut config) {
        Ok(rendered) => Some(rendered),
        Err(error) => {
            failure_error = error.to_string();
            state.record_template_failure(name, &failure_error);
            None
        }
    };
    let failed = rendered.is_none()
        || super::motion::apply_writer_state(name, &config, state)
            .map_err(|error| failure_error = error)
            .is_err();
    if failed {
        let error = failure_error;
        output.extend_from_slice(
            format!(
                "\n!!!!! Failed to process the custom G-code template {name}\n{error}!!!!! End of an error report for the custom G-code template {name}\n\n\n"
            )
            .as_bytes(),
        );
        return Ok(());
    }
    output.extend_from_slice(rendered.unwrap().as_bytes());
    output.push(b'\n');
    Ok(())
}

pub(super) fn append_layer_change(
    output: &mut Vec<u8>,
    state: &mut super::motion::EmitState,
    template_context: LayerTemplateContext<'_>,
) -> Result<(), SliceError> {
    let LayerTemplateContext {
        traversal,
        layer_index,
        layer_z,
        max_layer_z: _,
        totals,
        context,
    } = template_context;
    let template = &traversal.resolved.views.runtime_gcode.layer_change_gcode.0;
    if !template.is_empty() {
        let mut config = super::placeholders::base_config(
            traversal,
            context.metadata,
            context.first_layer_bounds,
        )?;
        super::motion::refresh_writer_state(&mut config, state);
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
        let volume = totals.used_filament * totals.diameter.powi(2) * 0.25 * std::f64::consts::PI;
        config.insert("extruded_volume_total", value::Value::number(volume));
        config.insert(
            "extruded_weight_total",
            value::Value::number(volume * totals.density * 0.001),
        );
        // Render or write-back failure both produce only the inline
        // report (`GCode.cpp:3920-3933`).
        let mut failure_error = String::new();
        let rendered = match template::render(template, &mut config) {
            Ok(rendered) => Some(rendered),
            Err(error) => {
                failure_error = error.to_string();
                state.record_template_failure("layer_change_gcode", &failure_error);
                None
            }
        };
        let name = "layer_change_gcode";
        let failed = rendered.is_none()
            || super::motion::apply_writer_state(name, &config, state)
                .map_err(|error| failure_error = error)
                .is_err();
        if failed {
            let error = failure_error;
            output.extend_from_slice(
                format!(
                    "\n!!!!! Failed to process the custom G-code template {name}\n{error}!!!!! End of an error report for the custom G-code template {name}\n\n\n"
                )
                .as_bytes(),
            );
        } else {
            output.extend_from_slice(rendered.unwrap().as_bytes());
            output.push(b'\n');
        }
    }
    output.extend_from_slice(b";_SET_FAN_SPEED_CHANGING_LAYER\n");
    Ok(())
}
