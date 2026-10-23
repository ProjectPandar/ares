//! `GCode.cpp::WipeTowerIntegration::{prime,tool_change}` Type2 dispatch.
use super::{
    super::{motion, timelapse},
    emit::{self, Append},
};
use crate::SliceError;
use std::rc::Rc;

pub(in crate::project_slice::gcode_emit) fn tool_change(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    tool: (usize, bool),
    context: timelapse::Context<'_>,
    geometry: motion::LayerGeometry<'_>,
) -> Result<(), SliceError> {
    if let Some((layer, z)) = state.wipe_tower.as_mut().unwrap().first_wall() {
        let data = Rc::clone(&state.wipe_tower.as_ref().unwrap().data);
        let result = &data.changes[layer][0];
        super::type1::append(
            output,
            state,
            Append {
                result,
                new_tool: Some(result.new_tool),
                z,
                context,
                geometry,
            },
        )?;
    }
    let needs_change = !state.extruder_selected || state.extruders.active_id() != tool.0;
    let tower = state
        .wipe_tower
        .as_mut()
        .expect("tower layer retains integration");
    let Some((layer, change, z)) = tower.next(needs_change, tool.1) else {
        return Ok(());
    };
    let data = Rc::clone(&tower.data);
    let result = &data.changes[layer][change];
    let append = if data.type1 {
        super::type1::append
    } else {
        emit::append
    };
    append(
        output,
        state,
        Append {
            result,
            new_tool: Some(tool.0),
            z,
            context,
            geometry,
        },
    )
}

pub(in crate::project_slice::gcode_emit) fn finalize(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    mut context: timelapse::Context<'_>,
    geometry: motion::LayerGeometry<'_>,
    layer_count: usize,
) -> Result<(), SliceError> {
    let tower = state.wipe_tower.as_ref().expect("finalizing active tower");
    let data = Rc::clone(&tower.data);
    if data.type1 {
        return Ok(());
    }
    let z = f64::from(data.final_purge.print_z);
    if (state.writer_z.unwrap_or(0.0) - z).abs() > 1e-4 {
        context.layer.index += 1;
        let printer = &context.traversal.resolved.views.full.printer.gcode;
        if matches!(
            printer.gcode_flavor,
            crate::GCodeFlavor::MakerWare | crate::GCodeFlavor::Sailfish
        ) && !printer.disable_m73.0
            && layer_count > 0
        {
            let percent = (100.0 * context.layer.index as f64 / layer_count as f64 + 0.5)
                .floor()
                .min(99.0) as usize;
            let comment = if state.options.gcode_comments {
                " ; update progress"
            } else {
                ""
            };
            output.extend_from_slice(format!("M73 P{percent}{comment}\n").as_bytes());
        }
        motion::tower::change_layer(output, state, z);
    }
    emit::append(
        output,
        state,
        Append {
            result: &data.final_purge,
            new_tool: None,
            z: None,
            context,
            geometry,
        },
    )
}

pub(in crate::project_slice::gcode_emit) fn initialize(
    output: &mut Vec<u8>,
    state: &mut motion::EmitState,
    context: timelapse::Context<'_>,
    geometry: motion::LayerGeometry<'_>,
) -> Result<(), SliceError> {
    let Some(tower) = &state.wipe_tower else {
        return Ok(());
    };
    let priming = tower.priming;
    let data = Rc::clone(&tower.data);
    let settings = &context.traversal.resolved.views.full;
    let z =
        settings.process.print.initial_layer_print_height.0 + settings.printer.remaining.z_offset.0;
    motion::tower::writer_z(output, state, z, "Move to the first layer height", false);
    if priming != super::integration::Priming::Disabled {
        for result in &data.priming {
            if !result.extrusions.is_empty() {
                emit::append(
                    output,
                    state,
                    Append {
                        result,
                        new_tool: Some(result.new_tool),
                        z: None,
                        context,
                        geometry,
                    },
                )?;
            }
        }
        if matches!(
            settings.printer.gcode.gcode_flavor,
            crate::GCodeFlavor::MarlinLegacy | crate::GCodeFlavor::MarlinFirmware
        ) {
            motion::tower::retract(output, state);
            output.extend_from_slice(b"M300 S800 P500\n");
            output.extend_from_slice(if priming == super::integration::Priming::Overlapping {
                b"M1 Remove priming towers and click button.\n"
            } else {
                b"M1 S10\n"
            });
        }
    }
    Ok(())
}
