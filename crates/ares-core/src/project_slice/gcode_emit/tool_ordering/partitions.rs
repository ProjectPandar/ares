//! `ToolOrdering.cpp:816–919`, including the raft separation-gap layer.

use super::LayerTools;
use crate::{ProcessTimelapseType, ProjectSettings};

const EPSILON: f64 = 1e-4;

pub(in crate::project_slice::gcode_emit) fn max_layer_height(
    settings: &ProjectSettings,
    max_object_layer_height: f64,
) -> f64 {
    // `ToolOrdering.cpp:137–149`; normalized option vectors duplicate their first value.
    let print = &settings.project.print;
    print
        .nozzle_diameter
        .0
        .iter()
        .enumerate()
        .map(|(index, nozzle)| {
            let height = print
                .max_layer_height
                .0
                .get(index)
                .unwrap_or(&print.max_layer_height.0[0])
                .0;
            if height == 0.0 {
                0.75 * nozzle.0
            } else {
                height
            }
        })
        .fold(f64::MAX, f64::min)
        .max(max_object_layer_height)
}

pub(super) fn fill(
    layers: &mut Vec<LayerTools>,
    settings: &ProjectSettings,
    object_bottom_z: f64,
    max_layer_height: f64,
) {
    if layers.is_empty() {
        return;
    }
    let mut last = None;
    for layer in layers.iter_mut() {
        layer.wipe_tower_partitions = layer.extruders.len();
        if let Some(&first) = layer.extruders.first() {
            layer.wipe_tower_partitions -= usize::from(last.is_none() || last == Some(first));
            last = layer.extruders.last().copied();
        }
    }
    let mut upper = 0;
    for layer in layers.iter_mut().rev() {
        upper = upper.max(layer.wipe_tower_partitions);
        layer.wipe_tower_partitions = upper;
    }
    let wrapping = settings.process.gcode.enable_wrapping_detection.0;
    for layer in layers
        .iter_mut()
        .take(settings.printer.gcode.wrapping_detection_layers.0 as usize)
    {
        layer.has_wipe_tower = wrapping;
    }
    let smooth = settings.process.print.timelapse_type == ProcessTimelapseType::Smooth;
    for layer in layers.iter_mut() {
        layer.has_wipe_tower |= (layer.has_object && (smooth || layer.wipe_tower_partitions > 0))
            || layer.print_z < object_bottom_z + EPSILON;
    }
    raft_gap(layers, object_bottom_z, max_layer_height);
    for i in 0..layers.len() - 1 {
        if layers[i].extruders.is_empty() || layers[i + 1].extruders.is_empty() {
            break;
        }
        if !layers[i + 1].has_wipe_tower
            && (layers[i + 1].extruders.first() != layers[i].extruders.last()
                || layers[i + 1].extruders.len() > 1)
        {
            layers[i + 1].has_wipe_tower = true;
        }
        let mut last_z = layers[i + 1].print_z;
        let mut j = i + 2;
        while j < layers.len() - 1 && !layers[j].has_wipe_tower {
            if layers[j + 1].print_z - last_z > max_layer_height + EPSILON {
                layers[j].has_wipe_tower = !wrapping;
                last_z = layers[j].print_z;
            }
            j += 1;
        }
    }
    let mut last_z = 0.0;
    for layer in layers.iter_mut().filter(|layer| layer.has_wipe_tower) {
        layer.wipe_tower_layer_height = layer.print_z - last_z;
        last_z = layer.print_z;
    }
}

fn raft_gap(layers: &mut Vec<LayerTools>, object_bottom_z: f64, max_layer_height: f64) {
    let Some(i) = (0..layers.len() - 1).find(|&i| {
        layers[i].print_z < object_bottom_z + EPSILON
            && layers[i + 1].print_z >= object_bottom_z + EPSILON
    }) else {
        return;
    };
    let Some(object) = (i + 1..layers.len()).find(|&j| layers[j].has_wipe_tower) else {
        return;
    };
    let gap = layers[object].print_z - layers[i].print_z;
    if gap > max_layer_height + EPSILON {
        let print_z = 0.5 * (layers[i].print_z + layers[object].print_z);
        let index = layers.partition_point(|layer| layer.print_z < print_z - EPSILON);
        if (layers[index].print_z - print_z).abs() < EPSILON {
            layers[index].has_wipe_tower = true;
        } else {
            // The new support-only layer keeps the following layer's tool/partitions.
            let layer = LayerTools {
                print_z,
                extruders: vec![layers[index].extruders[0]],
                wipe_tower_partitions: layers[index].wipe_tower_partitions,
                has_wipe_tower: true,
                ..Default::default()
            };
            layers.insert(index, layer);
        }
    }
}

pub(super) fn insert_extruder(layers: &mut [LayerTools], settings: &ProjectSettings) -> bool {
    // `ToolOrdering.cpp:323–340`: only partitioned layers need this additional tool.
    let print = &settings.process.print;
    if !print.enable_prime_tower.0 || print.wipe_tower_filament.0 == 0 {
        return false;
    }
    let tool = (print.wipe_tower_filament.0 - 1) as usize;
    let mut changed = false;
    for layer in layers {
        if layer.wipe_tower_partitions > 0 && !layer.extruders.contains(&tool) {
            layer.extruders.push(tool);
            changed = true;
        }
    }
    changed
}
