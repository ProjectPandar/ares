//! `GCode.cpp:2404–2467`: last-extrusion gaps and the critical layer warning.
use super::{LayerToPrint, raft_schedule};
use crate::project_slice::{
    island_print_order::OrderedExtrusionLayer,
    perimeters::classic::traversal::PreparedPostClassicTraversal,
    support_material::InstalledSupportLayer,
};
use crate::{ObjectOptions, PrintWarning, SlicingNotificationType, WarningLevel};

pub(super) fn collect(
    merged: &[(f64, LayerToPrint)],
    traversal: &PreparedPostClassicTraversal,
    layers: (&[Vec<OrderedExtrusionLayer>], &[Vec<InstalledSupportLayer>]),
    raft: Option<&raft_schedule::RaftSchedule>,
    options: &[ObjectOptions],
) -> Vec<PrintWarning> {
    let (objects, support) = layers;
    let mut last: Vec<Option<(f64, bool)>> = vec![None; objects.len()];
    let mut ranges = vec![Vec::new(); objects.len()];
    for &(z, entry) in merged {
        let object = entry.object_index;
        let has_object = entry.object_layer.is_some_and(|layer| {
            objects[object][layer]
                .islands
                .iter()
                .any(|island| !island.entities.is_empty())
        });
        let has_support = entry
            .support_layer
            .is_some_and(|layer| !support[object][layer].support_fills.is_empty());
        let has_raft = entry.raft_layer.is_some_and(|layer| {
            raft.expect("raft entry retains its adapter").layers[object][layer]
                .islands
                .iter()
                .any(|island| !island.entities.is_empty())
        });
        let support_present = entry.support_layer.is_some() || entry.raft_layer.is_some();
        let has_extrusions = has_object || has_support || has_raft;
        if !has_object && !support_present {
            continue;
        }
        let config = &options[object];
        let top = config.support_top_z_distance.0;
        let bottom = if config.support_bottom_z_distance.0 == 0.0 {
            top
        } else {
            config.support_bottom_z_distance.0
        };
        let top = (top / config.layer_height.0).ceil() * config.layer_height.0;
        let bottom = (bottom / config.layer_height.0).ceil() * config.layer_height.0;
        let height = if let Some(layer) = entry.object_layer {
            traversal.objects[object].physical_layers()[layer].height
        } else if let Some(layer) = entry.support_layer {
            support[object][layer].height
        } else {
            raft.expect("raft entry retains its adapter").plans[object][entry
                .raft_layer
                .expect("non-object/non-support entry is raft")]
            .z
            .height
        };
        let mut extra_gap = if support_present { bottom } else { top };
        if last[object].is_some_and(|(_, is_support)| is_support) {
            extra_gap = extra_gap.max(config.raft_contact_distance.0);
        }
        let previous_z = last[object].map_or(0.0, |(z, _)| z);
        let maximal_print_z = previous_z + height + extra_gap.max(0.0);
        if has_extrusions && z > maximal_print_z + 2.0 * super::EPSILON {
            ranges[object].push((previous_z, z));
        }
        if has_extrusions {
            last[object] = Some((z, support_present));
        }
    }
    let mut warnings = Vec::new();
    for (object, ranges) in ranges.into_iter().enumerate() {
        if ranges.is_empty() {
            continue;
        }
        let mut message = String::new();
        for (from, to) in ranges.into_iter().take(5) {
            use std::fmt::Write;
            let from = super::super::super::motion::format::ostream(from);
            let to = super::super::super::motion::format::ostream(to);
            writeln!(
                message,
                "The object has empty layers between {from} and {to} and can’t be printed."
            )
            .expect("writing to String");
        }
        let (resolved, _) = traversal.print_object_config(object);
        let name = traversal.project.objects()[resolved.source_object_index].name();
        message.push_str(&format!("Object: {name}\nParts of the object at these heights may be too thin or the object may have a faulty mesh."));
        warnings.push(PrintWarning {
            level: WarningLevel::Critical,
            message_type: SlicingNotificationType::EmptyGcodeLayers,
            message,
        });
    }
    warnings
}
