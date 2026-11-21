//! `WipeTower.cpp:77–116`, `WipeTower.hpp:745–774`.
use super::{filament::at, sparse_layers_skipped};
use crate::ProjectSettings;

pub(super) fn enabled(settings: &ProjectSettings) -> bool {
    settings
        .process
        .print
        .wipe_tower_sparse_layers_combination
        .0
        && !sparse_layers_skipped(settings)
        && settings.process.print.timelapse_type != crate::ProcessTimelapseType::Smooth
        && !settings.process.gcode.enable_wrapping_detection.0
}

pub(super) fn combine(
    settings: &ProjectSettings,
    layers: impl Iterator<Item = (f32, Option<(usize, usize)>)>,
    first_layer: usize,
    initial_tool: usize,
) -> Vec<(f32, bool)> {
    let layers = layers.collect::<Vec<_>>();
    let mut tool = layers
        .iter()
        .find_map(|(_, change)| change.map(|(old, _)| old))
        .unwrap_or(initial_tool);
    let project = &settings.project.print;
    let caps = layers
        .iter()
        .map(|(_, change)| {
            let nozzle = crate::options::mapped_extruder_id(settings, tool);
            let height = at(&project.max_layer_height.0, nozzle).0 as f32;
            if let Some((_, new)) = change {
                tool = *new;
            }
            if height > 0.0 {
                height
            } else {
                0.75 * at(&project.nozzle_diameter.0, nozzle).0 as f32
            }
        })
        .collect::<Vec<_>>();
    let mut pending = 0.0_f32;
    layers
        .iter()
        .enumerate()
        .map(|(index, &(height, change))| {
            if change.is_some() || index <= first_layer {
                pending = 0.0;
                return (height, false);
            }
            let merged = pending + height;
            let next_takes_it = layers.get(index + 1).is_some_and(|&(next, change)| {
                change.is_none() && merged + next <= caps[index + 1] + 1e-4_f32
            });
            if next_takes_it {
                pending = merged;
                (height, true)
            } else {
                pending = 0.0;
                (merged, false)
            }
        })
        .collect()
}

/// `GCode.cpp:1079–1091`: declare the result height if the tower body did not.
pub(super) fn append_height_tag(
    output: &mut Vec<u8>,
    state: &mut super::super::motion::EmitState,
    settings: &ProjectSettings,
    height: f32,
    gcode: &str,
) {
    if !enabled(settings)
        || state
            .last_height
            .is_some_and(|last| f64::from((last - height).abs()) <= 1e-4)
        || gcode.contains(&state.tags.height(""))
    {
        return;
    }
    state.last_height = Some(height);
    let text = super::super::format_processor_float(f64::from(height));
    output.extend_from_slice(state.tags.height(&text).as_bytes());
    output.push(b'\n');
}
