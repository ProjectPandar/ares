//! Ordinary initial writer selection and filament-start (`GCode.cpp:3144,3227,7710`).

use super::super::{tags, template};
use crate::{
    GenerationMetadata, SliceError,
    project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
};

pub(in crate::project_slice::gcode_emit) fn append_print_preamble(
    output: &mut Vec<u8>,
    state: &mut super::super::motion::EmitState,
    traversal: &PreparedPostClassicTraversal,
    metadata: GenerationMetadata,
    first_layer_bounds: Option<super::super::footprint::FirstLayerBounds>,
) -> Result<(), SliceError> {
    let is_bbl = tags::Tags::of(traversal).is_bbl();
    let multiple_extruders = state.extruders.multiple();
    let initial = state.initial_filaments.initial();
    let filament = if is_bbl {
        state.initial_filaments.non_support()
    } else {
        initial
    };
    // BBL's `init_extruder` already selected the initial extruder
    // BEFORE its filament-start render (`GCode.cpp:3144-3148`), so
    // `position` write-backs carry the extruder offset; the non-BBL
    // extruder is selected inside `set_extruder` AFTER the render
    // (`GCode.cpp:7736-7746`), so its write-back omits the offset.
    if is_bbl {
        super::super::motion::tool_selection::select(state, traversal, filament);
    }
    // BBL renders this explicitly before `;VT` (`GCode.cpp:3143-3154`);
    // compatible printers render it from the initial set_extruder call
    // (`GCode.cpp:7710-7745`).
    if let Some(source) = traversal
        .resolved
        .views
        .runtime_gcode
        .filament_start_gcode
        .0
        .get(filament)
        .or_else(|| {
            traversal
                .resolved
                .views
                .runtime_gcode
                .filament_start_gcode
                .0
                .first()
        })
        // `GCode.cpp:7710-7745` processes this through PlaceholderParser;
        // its skipper (`PlaceholderParser.cpp:2510-2520`) reduces Orca's
        // whitespace-only default sentinel to no output and no newline.
        .filter(|source| !source.trim().is_empty())
    {
        // Machine-start's `position` assignment was already written back
        // into writer state by the caller (`gcode_emit.rs`), so the
        // refresh below seeds upstream's post-assignment `m_pos`.
        let mut config =
            super::super::placeholders::base_config(traversal, metadata, first_layer_bounds)?;
        super::super::motion::refresh_writer_state(&mut config, state);
        config.insert(
            "current_extruder",
            super::super::value::Value::Integer(filament as i64),
        );
        config.insert(
            "filament_extruder_id",
            super::super::value::Value::Integer(filament as i64),
        );
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
            Ok(rendered) => match super::super::motion::apply_writer_state(name, &config, state) {
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
    if !is_bbl {
        if !multiple_extruders {
            append_pa(output, traversal, filament);
        }
        super::super::motion::tool_selection::toolchange(output, state, traversal, filament);
    }
    if is_bbl {
        output.extend_from_slice(format!(";VT{initial}\n").as_bytes());
    }
    if is_bbl || multiple_extruders {
        append_pa(output, traversal, filament);
    }
    if is_bbl {
        super::super::machine::append_flavor_preamble(output, traversal);
        output.extend_from_slice(b"M981 S1 P20000 ;open spaghetti detector\n");
    }
    Ok(())
}

fn append_pa(output: &mut Vec<u8>, traversal: &PreparedPostClassicTraversal, filament: usize) {
    let settings = &traversal.resolved.views.full;
    let is_bbl = tags::Tags::of(traversal).is_bbl();
    // Orca: set the initial pressure advance for the first filament
    // (`GCode.cpp:8048-8052`, `GCodeWriter.cpp:370-391`).
    let enabled = settings
        .filament
        .gcode
        .enable_pressure_advance
        .0
        .get(filament)
        .or_else(|| settings.filament.gcode.enable_pressure_advance.0.first())
        .is_some_and(|value| value.0);
    if enabled {
        let pa = settings
            .filament
            .gcode
            .pressure_advance
            .0
            .get(filament)
            .or_else(|| settings.filament.gcode.pressure_advance.0.first())
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
