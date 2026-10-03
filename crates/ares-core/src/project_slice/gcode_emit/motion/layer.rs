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

pub(in crate::project_slice::gcode_emit) fn emit_layer<F, D>(
    output: &mut Vec<u8>,
    layer: &mut OrderedExtrusionLayer,
    (geometry, density_for_region): (LayerGeometry<'_>, D),
    state: &mut EmitState,
    mut before_first_infill: F,
) -> Result<bool, SliceError>
where
    F: FnMut(&mut Vec<u8>, &mut EmitState) -> Result<bool, SliceError>,
    D: Fn(usize) -> f64,
{
    let mut interlude_emitted = false;
    for island in &mut layer.islands {
        let entities = std::mem::take(&mut island.entities);
        let has_infill = entities
            .iter()
            .any(|entity| !matches!(entity, IslandPrintEntity::Perimeter(_)));
        let infill_first = matches!(
            entities.iter().find(|entity| !is_ironing_entity(entity)),
            Some(
                IslandPrintEntity::Fill(_)
                    | IslandPrintEntity::FillCollection(_)
                    | IslandPrintEntity::Thin(_)
            )
        );
        let mut perimeters = Vec::new();
        let mut regions = std::collections::BTreeMap::<usize, (Vec<_>, Vec<_>)>::new();
        let mut region_ids = std::mem::take(&mut island.infill_regions).into_iter();
        for entity in entities {
            if matches!(entity, IslandPrintEntity::Perimeter(_)) {
                perimeters.push(entity);
                continue;
            }
            let region = region_ids.next().expect("infill retains its source region");
            let (infills, ironing) = regions.entry(region).or_default();
            if is_ironing_entity(&entity) {
                ironing.push(entity);
            } else {
                infills.push(entity);
            }
        }
        // GCode.cpp:6150-6178 chains each region independently.
        if infill_first {
            if !interlude_emitted && has_infill {
                interlude_emitted = before_first_infill(output, state)?;
            }
            for (&region, (infills, _)) in &mut regions {
                emit_infills(output, infills, geometry, state, density_for_region(region));
            }
            for perimeter in perimeters {
                perimeter::emit_perimeter(output, perimeter, geometry, state);
            }
        } else {
            for perimeter in perimeters {
                perimeter::emit_perimeter(output, perimeter, geometry, state);
            }
            if !interlude_emitted && has_infill {
                interlude_emitted = before_first_infill(output, state)?;
            }
            for (&region, (infills, _)) in &mut regions {
                emit_infills(output, infills, geometry, state, density_for_region(region));
            }
        }
        // GCode.cpp:5453-5468 emits ironing after infill-first walls.
        for (&region, (_, ironing)) in &mut regions {
            emit_infills(output, ironing, geometry, state, density_for_region(region));
        }
    }
    Ok(interlude_emitted)
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
