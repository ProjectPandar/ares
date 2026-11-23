use super::Project;
use crate::{
    OrcaInt,
    gcode_processor::CustomGCodeType,
    plater::SlotChange,
    project::{model_settings::Metadata, plate_config::parse_int_list},
};

impl Project {
    pub(crate) fn remap_slot_references(&mut self, change: SlotChange) {
        for object in &mut self.objects {
            remap_assignment(&mut object.region_overrides.extruder, change);
            for selector in [
                &mut object.object_overrides.support_filament,
                &mut object.object_overrides.support_interface_filament,
            ] {
                *selector = selector.and_then(|slot| change.surviving(slot.0).map(OrcaInt));
            }
            for volume in &mut object.volumes {
                remap_assignment(&mut volume.region_overrides.extruder, change);
                volume.mesh.mmu_segmentation_facets.remap_states(|slot| {
                    u8::try_from(change.assignment(i32::from(slot), 0)).unwrap_or_default()
                });
            }
        }
        for info in self.plates_custom_gcodes.values_mut() {
            info.gcodes.retain_mut(|item| {
                if item.kind != CustomGCodeType::ToolChange {
                    return true;
                }
                let Ok(slot) = i32::try_from(item.extruder) else {
                    return true;
                };
                match change.surviving(slot).or_else(|| match change {
                    SlotChange::Remove { replacement, .. } => replacement,
                    SlotChange::Insert { .. } => None,
                }) {
                    Some(slot) => {
                        item.extruder = slot as u32;
                        true
                    }
                    None => false,
                }
            });
        }
        for plate in &mut self.documents.model_settings.plates {
            remap_plate_metadata(&mut plate.metadata, change);
        }
    }
}

fn remap_plate_metadata(metadata: &mut [Metadata], change: SlotChange) {
    let ranges = metadata
        .iter()
        .rfind(|item| item.key == "other_layers_print_sequence_nums")
        .and_then(|item| item.value.parse().ok())
        .unwrap_or_default();
    for item in metadata {
        let updated = match item.key.as_str() {
            "first_layer_print_sequence" | "other_layers_print_sequence" => {
                let Some(ids) = parse_int_list(&item.value) else {
                    continue;
                };
                remap_print_order(&item.key, &ids, ranges, change)
            }
            key @ ("filament_maps"
            | "filament_map"
            | "filament_nozzle_map"
            | "filament_volume_map") => {
                let Some(mut values) = parse_int_list(&item.value) else {
                    continue;
                };
                match change {
                    SlotChange::Insert { at, .. } => {
                        let default = i32::from(matches!(key, "filament_maps" | "filament_map"));
                        values.insert((at as usize - 1).min(values.len()), default);
                    }
                    SlotChange::Remove { at, .. } if at as usize <= values.len() => {
                        values.remove(at as usize - 1);
                    }
                    SlotChange::Remove { .. } => continue,
                }
                Some(values)
            }
            _ => continue,
        };
        if let Some(values) = updated {
            item.value = values
                .iter()
                .map(i32::to_string)
                .collect::<Vec<_>>()
                .join(" ");
        }
    }
}

pub(super) fn remap_print_order(
    key: &str,
    values: &[i32],
    ranges: usize,
    change: SlotChange,
) -> Option<Vec<i32>> {
    let mut remapped = Vec::new();
    let order = |ids: &[i32]| {
        let mut slots = ids
            .iter()
            .filter_map(|slot| change.surviving(*slot))
            .collect::<Vec<_>>();
        if let SlotChange::Insert { at, .. } = change {
            slots.push(at);
        }
        slots
    };
    if key == "first_layer_print_sequence" {
        if !values.is_empty() && values[0] != 0 {
            return Some(order(values));
        }
    } else if ranges != 0 && values.len().is_multiple_of(ranges) && values.len() / ranges >= 2 {
        for range in values.chunks_exact(values.len() / ranges) {
            remapped.extend_from_slice(&range[..2]);
            remapped.extend(order(&range[2..]));
        }
        return Some(remapped);
    }
    None
}

fn remap_assignment(slot: &mut Option<OrcaInt>, change: SlotChange) {
    *slot = slot.map(|slot| OrcaInt(change.assignment(slot.0, 1)));
}
