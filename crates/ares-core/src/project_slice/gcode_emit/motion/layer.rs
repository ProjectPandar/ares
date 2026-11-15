//! GCode.cpp regional layer phases and infill chaining.

use super::{
    EmitState, LayerGeometry, MotionOptions, emit_fill_entity, emit_variable_width_entity,
    local_cursor, loop_paths::wipe_inward::WipeInwardSupport, perimeter,
};
use crate::{
    SliceError,
    project_slice::{
        fill_entities::FillExtrusionEntity,
        island_print_order::{
            IslandPrintEntity, OrderedExtrusionLayer,
            visits::{self, InstanceVisit},
        },
        perimeters::classic::shortest_path::{
            chain_and_reorder_entities, chain_and_reorder_entities_const,
        },
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
    (layer, visit, slices): (
        &mut OrderedExtrusionLayer,
        &InstanceVisit,
        Option<&[crate::geometry::ExPolygon]>,
    ),
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
    let geometry = phase.geometry;
    let mut interlude_emitted = false;
    let order = visits::island_order(
        layer,
        visit,
        slices,
        state
            .last_position_defined
            .then(|| local_cursor(state, geometry)),
    );
    for island_index in order {
        let island = &mut layer.islands[island_index];
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
        // GCode.cpp:7742-7755/8039-8053: ordinary walls, infill,
        // deferred walls, infill-first walls, then ironing.
        emit_perimeters(
            output,
            &regions,
            (geometry, first_layer, false, false),
            state,
            &options_for_region,
        )?;
        if !interlude_emitted && has_infill {
            interlude_emitted = before_first_infill(output, state)?;
        }
        for (&id, region) in &regions {
            if region.infills.is_empty() {
                continue;
            }
            let (density, _, gate) =
                options_for_region(id, &mut state.options, &mut state.current_seam_gap);
            state.small_area_gate = gate;
            emit_infills(output, &region.infills, geometry, state, density)?;
        }
        emit_perimeters(
            output,
            &regions,
            (geometry, first_layer, false, true),
            state,
            &options_for_region,
        )?;
        emit_perimeters(
            output,
            &regions,
            (geometry, first_layer, true, false),
            state,
            &options_for_region,
        )?;
        for (&id, region) in &regions {
            if region.ironing.is_empty() {
                continue;
            }
            let (density, _, gate) =
                options_for_region(id, &mut state.options, &mut state.current_seam_gap);
            state.small_area_gate = gate;
            emit_infills(output, &region.ironing, geometry, state, density)?;
        }
    }
    Ok(interlude_emitted)
}

fn emit_perimeters<D>(
    output: &mut Vec<u8>,
    regions: &std::collections::BTreeMap<usize, RegionEntities>,
    (geometry, first_layer, is_infill_first, unsupported_loops_only): (
        LayerGeometry<'_>,
        bool,
        bool,
        bool,
    ),
    state: &mut EmitState,
    options_for_region: &D,
) -> Result<(), SliceError>
where
    D: Fn(
        usize,
        &mut MotionOptions,
        &mut crate::FloatOrPercent,
    ) -> (f64, bool, super::super::small_area::Gate),
{
    // GCode.cpp:8889-8933: support is local to this region and perimeter pass.
    for (&id, region) in regions {
        if !region.had_perimeters {
            continue;
        }
        let (density, infill_first, gate) =
            options_for_region(id, &mut state.options, &mut state.current_seam_gap);
        state.small_area_gate = gate;
        let should_print = if first_layer {
            !is_infill_first
        } else {
            infill_first == is_infill_first
        };
        if !should_print {
            continue;
        }
        state.options.has_sparse_infill = density > 0.0;
        let mut wipe_support = (state.options.wipe
            && state.options.wipe_inward
            && !state.options.wipe_inward_distance.is_non_positive()
            && state.options.wipe_distance / geometry.scale.factor()
                > super::path::SOURCE_EPSILON_MM / geometry.scale.factor())
        .then(WipeInwardSupport::default);
        if !is_infill_first
            && unsupported_loops_only
            && let Some(support) = wipe_support.as_mut()
        {
            for entity in &region.perimeters {
                perimeter::preseed_wipe_support(entity, support);
            }
        }
        for entity in &region.perimeters {
            perimeter::emit_perimeter(
                output,
                (
                    entity,
                    !is_infill_first,
                    unsupported_loops_only,
                    wipe_support.as_mut(),
                ),
                geometry,
                state,
            )?;
        }
    }
    Ok(())
}

fn emit_infills(
    output: &mut Vec<u8>,
    entities: &[IslandPrintEntity],
    geometry: LayerGeometry<'_>,
    state: &mut EmitState,
    sparse_density: f64,
) -> Result<(), SliceError> {
    if entities.is_empty() {
        return Ok(());
    }
    // GCode.cpp:8988 applies the current emitted region's density.
    state.options.has_sparse_infill = sparse_density > 0.0;
    // GCode.cpp:8990–9000: borrow source entities, own reversals, and
    // order collection children only in a call-local copy.
    for mut entity in
        chain_and_reorder_entities_const(entities, local_cursor(state, geometry), geometry.scale)
    {
        match entity.as_ref() {
            IslandPrintEntity::Fill(entity) => {
                emit_fill_entity(output, entity, geometry, state)?;
            }
            IslandPrintEntity::FillCollection(_) => {
                let IslandPrintEntity::FillCollection(collection) = entity.to_mut() else {
                    unreachable!("entity remains a fill collection")
                };
                if !collection.no_sort {
                    chain_and_reorder_entities(
                        &mut collection.entities,
                        local_cursor(state, geometry),
                        geometry.scale,
                    );
                }
                for entity in &collection.entities {
                    emit_fill_entity(output, entity, geometry, state)?;
                }
            }
            IslandPrintEntity::Thin(entity) => {
                emit_variable_width_entity(output, entity, geometry, state)?;
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
