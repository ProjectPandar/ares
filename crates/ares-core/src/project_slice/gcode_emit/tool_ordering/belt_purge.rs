//! `BeltPurge.cpp:146–295`: allocate wiping after tool ordering without a classic tower.

use super::{
    ToolOrdering, object_options,
    wiping::{self, CollectionId, Layer, WipingExtrusions},
};
use crate::project_slice::{
    extrusion_islands::{LayerExtrusionIslands, PreparedPostExtrusionIslands},
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
        for (z, tool) in self
            .layers
            .iter()
            .take_while(|layer| top.is_none_or(|z| layer.print_z <= z + EPSILON))
            .flat_map(|layer| {
                layer
                    .extruders
                    .iter()
                    .map(move |&tool| (layer.print_z, tool))
            })
        {
            if tool != current {
                last_change = Some(z);
                current = tool;
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
        // BeltPurge.cpp:187–196,260–271: convert both operands before
        // multiplying the requested entry of the single-nozzle matrix.
        let purge_volume = |old: usize, new: usize| {
            if use_matrix {
                matrix[old * count + new] as f32 * multiplier
            } else {
                prime
            }
        };
        current = first;
        for tools in &mut self.layers {
            let mut objects = wiping::objects(traversal, original, support, tools.print_z);
            let layer = Layer {
                objects: &objects,
                tools: &tools.extruders,
                soluble: &soluble,
                support_filaments: &support_filaments,
            };
            let mut wiping = mark_changes(&layer, &mut current, purge_volume);
            let retained = &mut original.objects[prism];
            let physical = &traversal.objects[prism].physical_layers()[..retained.len()];
            if let Some(index) = wiping::object_layer_index(physical, tools.print_z) {
                drop_unclaimed_fills(
                    prism,
                    &mut objects,
                    &wiping,
                    &mut retained[index],
                    &mut ordered[prism][index],
                );
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

fn mark_changes(
    layer: &Layer<'_>,
    current: &mut usize,
    volume: impl Fn(usize, usize) -> f32,
) -> WipingExtrusions {
    let mut wiping = WipingExtrusions::new(layer);
    for &next in layer.tools {
        if next == *current {
            continue;
        }
        wiping.mark(layer, (*current, next), volume(*current, next));
        *current = next;
    }
    wiping
}

// BeltPurge.cpp:274–295,376–398: drop fills before order-only overrides
// are added. Keep collection IDs across all three views.
fn drop_unclaimed_fills(
    prism: usize,
    objects: &mut [wiping::Object],
    wiping: &WipingExtrusions,
    original: &mut LayerExtrusionIslands,
    ordered: &mut OrderedExtrusionLayer,
) {
    let claimed = |id| wiping.entity(prism, id, 0).is_some();
    if let Some(regions) = objects
        .iter_mut()
        .find(|object| object.id == prism)
        .and_then(|object| object.regions.as_mut())
    {
        for region in regions {
            region.infills.retain(|fill| claimed(fill.id));
        }
    }
    original
        .wiping_volumes
        .infills
        .retain(|fill| claimed(CollectionId::Infill(fill.id)));
    retain_fills(ordered, |id| claimed(CollectionId::Infill(id)));
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
