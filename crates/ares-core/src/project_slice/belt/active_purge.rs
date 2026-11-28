use crate::{
    project::effective_config::types::BoundedResolvedProjectConfig,
    project_slice::looped_intersections::LoopedPrintObject,
};

use super::EPSILON;

pub fn align_active_purge_plans(
    objects: &mut [LoopedPrintObject],
    resolved: &BoundedResolvedProjectConfig,
) {
    let printer = &resolved.views.full.printer.remaining;
    if !printer.enable_belt_purge_tower.0 || resolved.logical_filament_count <= 1 {
        return;
    }
    let marked = resolved
        .objects
        .iter()
        .filter(|object| object.object.belt_purge_tower_object.0)
        .map(|object| object.source_object_index)
        .collect::<Vec<_>>();
    if marked.len() != 1 {
        return;
    }
    let Some(prism) = objects.iter().position(|object| {
        marked.contains(&object.plan.source_object_index) && !object.plan.layers.is_empty()
    }) else {
        return;
    };
    let Some((_, reference)) = objects
        .iter()
        .enumerate()
        .filter(|(index, object)| *index != prism && !object.plan.layers.is_empty())
        .max_by(|(_, left), (_, right)| {
            left.plan
                .layers
                .last()
                .unwrap()
                .print_z
                .total_cmp(&right.plan.layers.last().unwrap().print_z)
        })
    else {
        return;
    };
    let height = objects[prism].plan.slicing_parameters.layer_height;
    if height <= EPSILON {
        return;
    }
    let residue = |z: f64| z - (z / height).floor() * height;
    let target = residue(reference.plan.layers[0].print_z);
    for object in objects {
        if object.plan.layers.is_empty() {
            continue;
        }
        let mut delta = target - residue(object.plan.layers[0].print_z);
        if delta > 0.5 * height {
            delta -= height;
        } else if delta <= -0.5 * height {
            delta += height;
        }
        if delta.abs() < EPSILON {
            continue;
        }
        for layer in &mut object.plan.layers {
            layer.print_z += delta;
        }
        // BeltPurge.cpp:325-345 shifts more than Layer::print_z.  The
        // support/brim generators and belt-floor clipping consume these plan
        // fields after slicing, while SupportFrame uses print_z_offset as the
        // local-frame equivalent of m_belt_global_z_offset.
        if let Some(slice) = &mut object.plan.belt_slice {
            slice.print_z_offset += delta;
            if let Some(floor) = &mut slice.floor {
                floor.shift += delta;
            }
        }
    }
}
