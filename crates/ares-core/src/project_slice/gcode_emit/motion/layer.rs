//! GCode.cpp regional layer phases and infill chaining.

use super::{
    EmitState, LayerGeometry, emit_fill_entity, emit_variable_width_entity, local_cursor, perimeter,
};
use crate::{
    SliceError,
    project_slice::{
        fill_entities::FillExtrusionEntity,
        island_print_order::{IslandPrintEntity, OrderedExtrusionLayer},
        perimeters::classic::shortest_path::chain_and_reorder_entities,
    },
};

#[derive(Default)]
struct RegionEntities {
    perimeters: Vec<IslandPrintEntity>,
    infills: Vec<IslandPrintEntity>,
    ironing: Vec<IslandPrintEntity>,
}

pub(in crate::project_slice::gcode_emit) fn emit_layer<F, D>(
    output: &mut Vec<u8>,
    layer: &mut OrderedExtrusionLayer,
    (geometry, first_layer, options_for_region): (LayerGeometry<'_>, bool, D),
    state: &mut EmitState,
    mut before_first_infill: F,
) -> Result<bool, SliceError>
where
    F: FnMut(&mut Vec<u8>, &mut EmitState) -> Result<bool, SliceError>,
    D: Fn(usize) -> (f64, bool, super::super::small_area::Gate),
{
    let mut interlude_emitted = false;
    for island in &mut layer.islands {
        let entities = std::mem::take(&mut island.entities);
        let has_infill = entities
            .iter()
            .any(|entity| !matches!(entity, IslandPrintEntity::Perimeter(_)));
        let mut regions = std::collections::BTreeMap::<usize, RegionEntities>::new();
        let mut region_ids = std::mem::take(&mut island.infill_regions).into_iter();
        let mut perimeter_ids = std::mem::take(&mut island.perimeter_regions).into_iter();
        for entity in entities {
            if matches!(entity, IslandPrintEntity::Perimeter(_)) {
                let region = perimeter_ids
                    .next()
                    .expect("perimeter retains its source region");
                regions.entry(region).or_default().perimeters.push(entity);
                continue;
            }
            let region = region_ids.next().expect("infill retains its source region");
            let region = regions.entry(region).or_default();
            if is_ironing_entity(&entity) {
                region.ironing.push(entity);
            } else {
                region.infills.push(entity);
            }
        }
        // GCode.cpp:5450-5468,6131-6145: wall-first regions, all infills,
        // infill-first region walls, then ironing. First layer is wall-first.
        for (&id, region) in &mut regions {
            let (density, infill_first, gate) = options_for_region(id);
            state.small_area_gate = gate;
            if first_layer || !infill_first {
                emit_perimeters(output, &mut region.perimeters, geometry, state, density);
            }
        }
        if !interlude_emitted && has_infill {
            interlude_emitted = before_first_infill(output, state)?;
        }
        for (&id, region) in &mut regions {
            state.small_area_gate = options_for_region(id).2;
            emit_infills(
                output,
                &mut region.infills,
                geometry,
                state,
                options_for_region(id).0,
            );
        }
        for (&id, region) in &mut regions {
            state.small_area_gate = options_for_region(id).2;
            emit_perimeters(
                output,
                &mut region.perimeters,
                geometry,
                state,
                options_for_region(id).0,
            );
        }
        for (&id, region) in &mut regions {
            state.small_area_gate = options_for_region(id).2;
            emit_infills(
                output,
                &mut region.ironing,
                geometry,
                state,
                options_for_region(id).0,
            );
        }
    }
    Ok(interlude_emitted)
}

fn emit_perimeters(
    output: &mut Vec<u8>,
    entities: &mut Vec<IslandPrintEntity>,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
    sparse_density: f64,
) {
    if entities.is_empty() {
        return;
    }
    state.options.has_sparse_infill = sparse_density > 0.0;
    for entity in entities.drain(..) {
        perimeter::emit_perimeter(output, entity, geometry, state);
    }
}

fn emit_infills(
    output: &mut Vec<u8>,
    entities: &mut Vec<IslandPrintEntity>,
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
    sparse_density: f64,
) {
    if entities.is_empty() {
        return;
    }
    // GCode.cpp:6163/7635-7636 applies the current emitted region's density.
    state.options.has_sparse_infill = sparse_density > 0.0;
    chain_and_reorder_entities(entities, local_cursor(state, geometry));
    for entity in entities.drain(..) {
        match entity {
            IslandPrintEntity::Fill(entity) => {
                emit_fill_entity(output, &entity, geometry, state);
            }
            IslandPrintEntity::FillCollection(collection) => {
                let collection = collection.chained_path_from(local_cursor(state, geometry));
                for entity in &collection.entities {
                    emit_fill_entity(output, entity, geometry, state);
                }
            }
            IslandPrintEntity::Thin(entity) => {
                emit_variable_width_entity(output, &entity, geometry, state);
            }
            IslandPrintEntity::Perimeter(_) => {
                unreachable!("infill phase contains only infill entities")
            }
        }
    }
}

fn is_ironing_entity(entity: &IslandPrintEntity) -> bool {
    let path_is_ironing = |entity: &FillExtrusionEntity| matches!(entity, FillExtrusionEntity::Path(path) | FillExtrusionEntity::Oriented(path) if path.role == crate::ExtrusionRole::Ironing);
    match entity {
        IslandPrintEntity::Fill(entity) => path_is_ironing(entity),
        IslandPrintEntity::FillCollection(collection) => {
            collection.entities.first().is_some_and(path_is_ironing)
        }
        IslandPrintEntity::Perimeter(_) | IslandPrintEntity::Thin(_) => false,
    }
}
