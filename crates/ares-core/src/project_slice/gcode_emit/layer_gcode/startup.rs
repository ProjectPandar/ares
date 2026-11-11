//! Ordinary initial writer selection and filament-start (`GCode.cpp:3144,3227,7710`).

use super::super::{motion, tags, template};
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
    if !is_bbl && multiple_extruders {
        return super::super::tool_transition::initial(
            output,
            state,
            traversal,
            metadata,
            first_layer_bounds,
        );
    }
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
        let mut config = super::super::placeholders::base_config(
            traversal,
            metadata,
            first_layer_bounds,
            Some(&state.template_rng),
            state.total_layer_count,
        )?;
        super::super::motion::refresh_writer_state(&mut config, state);
        config.insert(
            "current_extruder",
            super::super::value::Value::Integer(filament as i64),
        );
        config.insert(
            "filament_extruder_id",
            super::super::value::Value::Integer(filament as i64),
        );
        config.insert(
            "current_nozzle_id",
            super::super::value::Value::Integer(if is_bbl {
                state.initial_filaments.non_support_nozzle_id()
            } else {
                state.initial_filaments.nozzle_id(filament)
            }),
        );
        // Before the first change_layer, m_layer_index is -1
        // (`GCode.hpp:193`). BBL startup overlays only layer_num
        // (`GCode.cpp:3149`); ordinary set_extruder also overlays the
        // writer-domain Z minus z_offset and m_max_layer_z (:7728-7730).
        config.insert("layer_num", super::super::value::Value::Integer(-1));
        if !is_bbl {
            config.insert(
                "layer_z",
                super::super::value::Value::number(
                    state.writer_z.unwrap_or(0.0) - state.options.z_offset,
                ),
            );
            config.insert(
                "max_layer_z",
                super::super::value::Value::number(f64::from(state.max_layer_z)),
            );
        }
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
            Ok(rendered) => match motion::apply_writer_state(name, &config, &rendered, state) {
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
            super::super::pressure_advance::append(output, state, traversal, filament);
        }
        super::super::motion::tool_selection::toolchange(output, state, traversal, filament, "");
    }
    if is_bbl {
        let multiple_nozzles = super::super::tool_ordering::nozzle_count(traversal)
            .into_iter()
            .any(|count| count > 1);
        if multiple_nozzles {
            output.extend_from_slice(format!(";VT{initial} H-1\n").as_bytes());
        } else {
            output.extend_from_slice(format!(";VT{initial}\n").as_bytes());
        }
    }
    if is_bbl || multiple_extruders {
        super::super::pressure_advance::append(output, state, traversal, filament);
    }
    if is_bbl {
        super::super::machine::append_start_temperature_wait(output, traversal, state);
        super::super::machine::append_flavor_preamble(output, traversal);
        output.extend_from_slice(b"M981 S1 P20000 ;open spaghetti detector\n");
    }
    Ok(())
}
