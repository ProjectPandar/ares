//! GCode.cpp regional layer phases and infill chaining.

use super::{
    EmitState, LayerGeometry, MotionOptions, emit_fill_entity, emit_variable_width_entity,
    local_cursor, perimeter,
};
use crate::{
    SliceError,
    project_slice::{
        fill_entities::FillExtrusionEntity,
        island_print_order::{IslandPrintEntity, OrderedExtrusionLayer},
    },
};

#[derive(Default)]
struct RegionEntities {
    perimeters: Vec<IslandPrintEntity>,
    had_perimeters: bool,
    infills: Vec<IslandPrintEntity>,
    ironing: Vec<IslandPrintEntity>,
}

pub(in crate::project_slice::gcode_emit) fn emit_layer<F, D>(
    output: &mut Vec<u8>,
    layer: &mut OrderedExtrusionLayer,
    (phase, first_layer, options_for_region): (super::shared_reversals::EmissionPhase<'_>, bool, D),
    state: &mut EmitState,
    mut before_first_infill: F,
) -> Result<bool, SliceError>
where
    F: FnMut(&mut Vec<u8>, &mut EmitState) -> Result<bool, SliceError>,
    D: Fn(
        usize,
        &mut MotionOptions,
        &mut crate::FloatOrPercent,
    ) -> (f64, bool, super::super::small_area::Gate),
{
    let super::shared_reversals::EmissionPhase { geometry, reversal } = phase;
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
                let region = regions.entry(region).or_default();
                region.perimeters.push(entity);
                region.had_perimeters = true;
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
            if !region.had_perimeters {
                continue;
            }
            let (density, infill_first, gate) =
                options_for_region(id, &mut state.options, &mut state.current_seam_gap);
            state.small_area_gate = gate;
            if first_layer || !infill_first {
                emit_perimeters(output, &mut region.perimeters, geometry, state, density)?;
            }
        }
        if !interlude_emitted && has_infill {
            interlude_emitted = before_first_infill(output, state)?;
        }
        for (&id, region) in &mut regions {
            if region.infills.is_empty() {
                continue;
            }
            let (density, _, gate) =
                options_for_region(id, &mut state.options, &mut state.current_seam_gap);
            state.small_area_gate = gate;
            emit_infills(
                output,
                &mut region.infills,
                super::shared_reversals::EmissionPhase { geometry, reversal },
                state,
                density,
            )?;
        }
        for (&id, region) in &mut regions {
            // GCode.cpp:6135-6140 applies even when this phase skips the
            // original perimeter collection; our first phase drains it.
            if !region.had_perimeters {
                continue;
            }
            let (density, _, gate) =
                options_for_region(id, &mut state.options, &mut state.current_seam_gap);
            state.small_area_gate = gate;
            emit_perimeters(output, &mut region.perimeters, geometry, state, density)?;
        }
        for (&id, region) in &mut regions {
            if region.ironing.is_empty() {
                continue;
            }
            let (density, _, gate) =
                options_for_region(id, &mut state.options, &mut state.current_seam_gap);
            state.small_area_gate = gate;
            emit_infills(
                output,
                &mut region.ironing,
                super::shared_reversals::EmissionPhase { geometry, reversal },
                state,
                density,
            )?;
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
) -> Result<(), SliceError> {
    if entities.is_empty() {
        return Ok(());
    }
    state.options.has_sparse_infill = sparse_density > 0.0;
    for entity in entities.drain(..) {
        perimeter::emit_perimeter(output, entity, geometry, state)?;
    }
    Ok(())
}

fn emit_infills(
    output: &mut Vec<u8>,
    entities: &mut Vec<IslandPrintEntity>,
    phase: super::shared_reversals::EmissionPhase<'_>,
    state: &mut EmitState,
    sparse_density: f64,
) -> Result<(), SliceError> {
    if entities.is_empty() {
        return Ok(());
    }
    // GCode.cpp:6163/7635-7636 applies the current emitted region's density.
    state.options.has_sparse_infill = sparse_density > 0.0;
    // GCode.cpp:6164 runs chain_and_reorder_extrusion_entities over the
    // shared layer entities: reversals persist to later islands/instances
    // of the same source object (also covers the ironing pass above and
    // `extrude_support`, GCode.cpp:6201).
    let geometry = phase.geometry;
    state
        .shared_reversals
        .chain_infill(phase.reversal, entities, local_cursor(state, geometry));
    for entity in entities.drain(..) {
        match entity {
            IslandPrintEntity::Fill(entity) => {
                emit_fill_entity(output, &entity, geometry, state)?;
            }
            IslandPrintEntity::FillCollection(collection) => {
                let collection = collection.chained_path_from(local_cursor(state, geometry));
                for entity in &collection.entities {
                    emit_fill_entity(output, entity, geometry, state)?;
                }
            }
            IslandPrintEntity::Thin(entity) => {
                emit_variable_width_entity(output, &entity, geometry, state)?;
            }
            IslandPrintEntity::Perimeter(_) => {
                unreachable!("infill phase contains only infill entities")
            }
        }
    }
    Ok(())
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
