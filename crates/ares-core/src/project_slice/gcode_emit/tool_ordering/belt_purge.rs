//! `BeltPurge.cpp:146–295`: allocate wiping after tool ordering without a classic tower.

use super::{
    ToolOrdering, object_options,
    wiping::{self, CollectionId, Layer, WipingExtrusions},
};
use crate::project_slice::{
    extrusion_islands::PreparedPostExtrusionIslands,
    island_print_order::{IslandPrintEntity, OrderedExtrusionLayer},
    support_material::InstalledSupportLayer,
};

const EPSILON: f64 = 1e-4;

impl ToolOrdering {
    pub(in crate::project_slice::gcode_emit) fn plan_belt_purge(
        &mut self,
        original: &mut PreparedPostExtrusionIslands,
        ordered: &mut [Vec<OrderedExtrusionLayer>],
        support: &[Vec<InstalledSupportLayer>],
    ) -> Result<(), crate::SliceError> {
        let traversal = &original
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .predecessor;
        let settings = &traversal.resolved.views.full;
        let printer = &settings.printer.remaining;
        let config = &settings.filament.gcode;
        // BeltPurge.cpp:48–61; Print.cpp:3598–3628.
        if !printer.belt_printer.0
            || !printer.enable_belt_purge_tower.0
            || settings.process.print.spiral_mode.0
            || settings.process.print.print_sequence == crate::ProcessPrintSequence::ByObject
            || config.filament_diameter.0.len() <= 1
        {
            return Ok(());
        }
        let is_prism = |index| object_options(traversal, index).belt_purge_tower_object.0;
        let Some(prism) = (0..ordered.len()).find(|&index| is_prism(index)) else {
            return Ok(());
        };
        let first = self
            .first_extruder
            .ok_or_else(|| crate::SliceError::Slicing {
                message:
                    "The print is empty. The model is not printable with current print settings."
                        .to_owned(),
                plate_index: traversal.project.first_plate_id(),
                layer_height: f64::from(settings.process.object.layer_height.0 as f32),
                wall_loops: settings.process.region.wall_loops.0,
                sparse_infill_density: f64::from(
                    settings.process.region.sparse_infill_density.0 as f32,
                ),
            })?;
        // A print-wide no-change return precedes all trimming, fill removal and repair.
        if self
            .layers
            .iter()
            .flat_map(|layer| &layer.extruders)
            .all(|&tool| tool == first)
        {
            return Ok(());
        }

        // BeltPurge.cpp:198–256,353–368: keep the ordering, but remove the
        // prism's layers above the last real change at or below the model top.
        let top = traversal
            .objects
            .iter()
            .enumerate()
            .filter(|&(index, _)| !is_prism(index))
            .flat_map(|(index, object)| {
                [
                    object.physical_layers().last().map(|layer| layer.print_z),
                    support[index].last().map(|layer| layer.print_z),
                ]
            })
            .flatten()
            .max_by(f64::total_cmp);
        let mut current = first;
        let mut last_change = None;
        for layer in self
            .layers
            .iter()
            .take_while(|layer| top.is_none_or(|z| layer.print_z <= z + EPSILON))
        {
            for &tool in &layer.extruders {
                if tool != current {
                    last_change = Some(layer.print_z);
                    current = tool;
                }
            }
        }
        if let Some(z) = last_change {
            let keep = traversal.objects[prism]
                .physical_layers()
                .partition_point(|layer| layer.print_z <= z + EPSILON);
            ordered[prism].truncate(keep);
            original.objects[prism].truncate(keep);
        }

        let count = config.filament_colour.0.len();
        let flags = |values: &[crate::OrcaBool]| {
            (0..count)
                .map(|tool| values.get(tool).unwrap_or(&values[0]).0)
                .collect::<Vec<_>>()
        };
        let soluble = flags(&config.filament_soluble.0);
        let support_filaments = flags(&config.filament_is_support.0);
        let matrix = &settings.project.print.flush_volumes_matrix.0;
        let multiplier = settings.project.print.flush_multiplier.0[0].0 as f32;
        let prime = settings.process.print.prime_volume.0 as f32;
        let use_matrix = settings.printer.gcode.single_extruder_multi_material.0
            && settings.printer.gcode.purge_in_prime_tower.0;
        current = first;
        for tools in &mut self.layers {
            let mut objects = wiping::objects(traversal, original, support, tools.print_z);
            let layer = Layer {
                objects: &objects,
                tools: &tools.extruders,
                soluble: &soluble,
                support_filaments: &support_filaments,
            };
            let mut wiping = WipingExtrusions::new(&layer);
            for &next in &tools.extruders {
                if next != current {
                    // BeltPurge.cpp:187–196,260–271: nozzle zero of the
                    // single-nozzle matrix; convert both operands before multiplying.
                    let volume = if use_matrix {
                        matrix[current * count + next] as f32 * multiplier
                    } else {
                        prime
                    };
                    wiping.mark(&layer, (current, next), volume);
                    current = next;
                }
            }
            // BeltPurge.cpp:274–295,376–398: distinguish volume claims from
            // order-only overrides. Keep collection IDs across all three views.
            let retained = &mut original.objects[prism];
            let physical = &traversal.objects[prism].physical_layers()[..retained.len()];
            if let Some(index) = wiping::object_layer_index(physical, tools.print_z) {
                let claimed = |id| wiping.entity(prism, id, 0).is_some();
                if let Some(object) = objects.iter_mut().find(|object| object.id == prism) {
                    for region in object.regions.iter_mut().flatten() {
                        region.infills.retain(|fill| claimed(fill.id));
                    }
                }
                retained[index]
                    .wiping_volumes
                    .infills
                    .retain(|fill| claimed(CollectionId::Infill(fill.id)));
                retain_fills(&mut ordered[prism][index], |id| {
                    claimed(CollectionId::Infill(id))
                });
            }
            wiping.ensure_order(&Layer {
                objects: &objects,
                tools: &tools.extruders,
                soluble: &soluble,
                support_filaments: &support_filaments,
            });
            tools.wiping = wiping;
        }
        Ok(())
    }
}

fn retain_fills(layer: &mut OrderedExtrusionLayer, keep: impl Fn(usize) -> bool) {
    for island in &mut layer.islands {
        let mut ids = island.infill_collections.iter().copied();
        island.entities.retain(|entity| {
            matches!(entity, IslandPrintEntity::Perimeter(_))
                || keep(ids.next().expect("fill retains its source collection"))
        });
        let mut ids = island.infill_collections.iter().copied();
        island
            .infill_regions
            .retain(|_| keep(ids.next().expect("fill retains its source collection")));
        island.infill_collections.retain(|&id| keep(id));
    }
}
