//! `ToolOrdering::reorder_extruders_for_minimum_flush_volume`.

mod groups;
mod solve;

use std::collections::BTreeMap;

use super::LayerTools;
use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;
use crate::{ProjectSettings, WipeTowerType};

pub(super) fn apply(layers: &mut [LayerTools], traversal: &PreparedPostClassicTraversal) {
    if layers.is_empty() {
        return;
    }
    let settings = &traversal.resolved.views.full;
    let bbl = settings
        .printer
        .remaining
        .printer_model
        .0
        .starts_with("Bambu Lab");
    let count = settings.filament.gcode.filament_colour.0.len();
    let maps = (0..count)
        .map(|id| {
            if bbl {
                settings.project.gcode.filament_map.0[id].0 as usize - 1
            } else {
                0
            }
        })
        .collect::<Vec<_>>();
    let matrices = matrices(settings, count, bbl);
    let custom = custom_sequences(layers, settings);
    groups::reorder(layers, &maps, &matrices, &custom);
}

fn matrices(settings: &ProjectSettings, count: usize, bbl: bool) -> Vec<Vec<Vec<f32>>> {
    let project = &settings.project.print;
    let printer = &settings.printer.gcode;
    let nozzles = project.nozzle_diameter.0.len();
    let use_matrix = (printer.purge_in_prime_tower.0 && printer.single_extruder_multi_material.0)
        || bbl
        || printer.wipe_tower_type == WipeTowerType::Type1;
    (0..nozzles)
        .map(|nozzle| nozzle_matrix(settings, count, nozzle, use_matrix))
        .collect()
}

fn nozzle_matrix(
    settings: &ProjectSettings,
    count: usize,
    nozzle: usize,
    use_matrix: bool,
) -> Vec<Vec<f32>> {
    let project = &settings.project.print;
    let stride = project.flush_volumes_matrix.0.len() / project.nozzle_diameter.0.len();
    let multiplier = project
        .flush_multiplier
        .0
        .get(nozzle)
        .map_or(1.0, |value| value.0);
    (0..count)
        .map(|row| {
            (0..count)
                .map(|column| {
                    let value = if use_matrix {
                        project.flush_volumes_matrix.0[nozzle * stride + row * count + column]
                            as f32
                    } else {
                        settings.process.print.prime_volume.0 as f32
                    };
                    // C++ promotes the float matrix entry for its double multiplier.
                    (f64::from(value) * multiplier) as f32
                })
                .collect()
        })
        .collect()
}

fn custom_sequences(
    layers: &[LayerTools],
    settings: &ProjectSettings,
) -> BTreeMap<usize, Vec<usize>> {
    let project = &settings.project.print;
    let configured = &project.other_layers_print_sequence.0;
    let count = project.other_layers_print_sequence_nums.0 as usize;
    let ranges = if count == 0 || configured.is_empty() {
        Vec::new()
    } else {
        configured
            .chunks_exact(configured.len() / count)
            .collect::<Vec<_>>()
    };
    let mut custom = BTreeMap::new();
    for (index, layer) in layers.iter().enumerate() {
        if index == 0 {
            if !layer.extruders.is_empty() {
                custom.insert(index, layer.extruders.clone());
            }
            continue;
        }
        let layer_number = index as i32 + 1;
        let Some(row) = ranges
            .iter()
            .rev()
            .find(|row| layer_number >= row[0].0 && layer_number <= row[1].0)
        else {
            continue;
        };
        let sequence = row[2..]
            .iter()
            .map(|id| (id.0 as u32).wrapping_sub(1) as usize)
            .filter(|id| layer.extruders.contains(id))
            .collect::<Vec<_>>();
        assert_eq!(sequence.len(), layer.extruders.len());
        custom.insert(index, sequence);
    }
    custom
}
