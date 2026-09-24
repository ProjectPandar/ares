// Inactive post-traversal ordering and nonempty append from OrcaSlicer v2.4.2
// `PerimeterGenerator.cpp:1451-1569`.

mod reorient;
#[cfg(test)]
mod tests;
mod types;

pub(in crate::project_slice) use types::{
    AppendedPerimeterCollections, InactiveOuterBrimReordering, InactivePostCollectionBranches,
    InactiveWallReordering, PreparedPerimeterAppendObject, PreparedPerimeterAppendRecord,
    PreparedPerimeterAppendSurface, PreparedPostClassicPerimeterAppend,
};

use crate::geometry::{CoordinateScale, ExPolygon};
use crate::{ObjectOptions, ProcessBrimType, ProcessWallSequence, RegionOptions};

use super::{
    entity_collections::{
        ExtrusionEntity, ExtrusionEntityCollection, PreparedEntityCollectionObject,
        PreparedEntityCollectionRecord, PreparedEntityCollectionSurface,
        PreparedPostClassicEntityCollections,
    },
    traversal::{ClassicTraversalRecord, PostClassicTraversalPrintObject},
};

pub(in crate::project_slice) fn finish(
    prepared: PreparedPostClassicEntityCollections,
) -> PreparedPostClassicPerimeterAppend {
    let PreparedPostClassicEntityCollections {
        predecessor,
        objects,
    } = prepared;
    assert_eq!(objects.len(), predecessor.objects.len());
    let objects = objects
        .into_iter()
        .enumerate()
        .map(|(index, source)| {
            let traversal = &predecessor.objects[index];
            let source_object_index = traversal
                .predecessor
                .predecessor
                .predecessor
                .predecessor
                .object
                .identity()
                .0;
            let object_options = &predecessor
                .resolved
                .objects
                .iter()
                .find(|object| object.source_object_index == source_object_index)
                .expect("an O10 object retains its resolved source object")
                .object;
            transform_object(source, traversal, object_options, predecessor.scale)
        })
        .collect();
    PreparedPostClassicPerimeterAppend {
        predecessor,
        objects,
    }
}

fn transform_object(
    source: PreparedEntityCollectionObject,
    traversal: &PostClassicTraversalPrintObject,
    object_options: &ObjectOptions,
    scale: CoordinateScale,
) -> PreparedPerimeterAppendObject {
    assert_eq!(source.records.len(), traversal.records.len());
    let records = source
        .records
        .into_iter()
        .enumerate()
        .map(
            |(index, source)| match (source, &traversal.records[index]) {
                (None, None) => None,
                (Some(source), Some(traversal_record)) => {
                    let region = region_options(traversal, index);
                    let inactive = classify_inactive(traversal_record, object_options);
                    let wall_sequence = if index == 0
                        && object_options.brim_type == ProcessBrimType::OuterOnly
                        && object_options.brim_width.0 > 0.0
                    {
                        ProcessWallSequence::OuterInner
                    } else {
                        region.wall_sequence
                    };
                    let reorient = SurfaceReorient {
                        record: traversal_record,
                        region,
                        lower: traversal.lower_slices(index),
                        scale,
                    };
                    Some(transform_record(
                        source,
                        inactive,
                        wall_sequence,
                        index,
                        reorient,
                    ))
                }
                _ => panic!("O9/O5 optional record alignment is invariant"),
            },
        )
        .collect();
    PreparedPerimeterAppendObject { records }
}

fn transform_record(
    source: PreparedEntityCollectionRecord,
    inactive: InactivePostCollectionBranches,
    wall_sequence: ProcessWallSequence,
    layer_id: usize,
    reorient: SurfaceReorient<'_>,
) -> PreparedPerimeterAppendRecord {
    let surfaces = source
        .surfaces
        .into_iter()
        .enumerate()
        .map(|(index, mut surface)| {
            // Steep-overhang reversal runs on each surface collection
            // right after `traverse_loops` (`PerimeterGenerator.cpp:1446-1453`).
            reorient::ReorientPlan::new(reorient.record, reorient.region).apply(
                &mut surface.collection,
                &reorient.record.surfaces[index].roots,
                reorient.lower,
                reorient.scale,
            );
            transform_surface(surface, inactive, wall_sequence, layer_id)
        })
        .collect();
    PreparedPerimeterAppendRecord { surfaces }
}

fn transform_surface(
    mut source: PreparedEntityCollectionSurface,
    inactive: InactivePostCollectionBranches,
    wall_sequence: ProcessWallSequence,
    layer_id: usize,
) -> PreparedPerimeterAppendSurface {
    reorder_walls(&mut source.collection, wall_sequence, layer_id);
    PreparedPerimeterAppendSurface {
        source_index: source.source_index,
        inactive,
        appended: append_nonempty(source.collection),
    }
}

fn region_options(
    traversal: &PostClassicTraversalPrintObject,
    record_index: usize,
) -> &RegionOptions {
    let prelude = &traversal.predecessor.predecessor.predecessor.predecessor;
    let input = prelude.object.records[record_index]
        .as_ref()
        .expect("an O10 collection has an aligned perimeter input");
    prelude.object.region_options(input)
}

/// Borrow bundle keeping `transform_record` below the clippy argument
/// limit while the steep-overhang reversal needs record, region, lower
/// slices and scale at the surface seam.
struct SurfaceReorient<'a> {
    record: &'a ClassicTraversalRecord,
    region: &'a RegionOptions,
    lower: Option<&'a [ExPolygon]>,
    scale: CoordinateScale,
}

fn classify_inactive(
    record: &ClassicTraversalRecord,
    object: &ObjectOptions,
) -> InactivePostCollectionBranches {
    let layer_id = match record.branch {
        super::traversal::PendingPathBranch::OverhangClipping { layer_id, .. }
        | super::traversal::PendingPathBranch::OrdinaryUnsplit { layer_id, .. } => layer_id,
    };
    let outer_brim = if layer_id != 0 {
        InactiveOuterBrimReordering::LaterLayer {
            layer_id,
            brim_type: object.brim_type,
            brim_width: object.brim_width.0,
        }
    } else if object.brim_type != ProcessBrimType::OuterOnly {
        InactiveOuterBrimReordering::DifferentBrimType {
            brim_type: object.brim_type,
            brim_width: object.brim_width.0,
        }
    } else {
        InactiveOuterBrimReordering::WidthNotPositive {
            brim_width: object.brim_width.0,
        }
    };
    InactivePostCollectionBranches {
        wall_reordering: InactiveWallReordering::InnerOuter { outer_brim },
    }
}

fn reorder_walls(
    collection: &mut ExtrusionEntityCollection,
    sequence: ProcessWallSequence,
    layer_id: usize,
) {
    // `ExtrusionEntityCollection::reverse` (`ExtrusionEntityCollection.cpp:
    // 65-72`): reverse every non-loop entity in place, then reverse the
    // entity order. Rust `Vec::reverse` alone would not flip a `Path`/`Multi-
    // Path`'s internal direction.
    fn reverse_collection(collection: &mut ExtrusionEntityCollection) {
        for entity in &mut collection.entities {
            if !entity.is_loop() {
                entity.reverse();
            }
        }
        collection.entities.reverse();
    }
    match sequence {
        ProcessWallSequence::InnerOuter => {}
        // `PerimeterGenerator.cpp:1467` (`is_outer_wall_first` / first-layer
        // outer-brim reverse).
        ProcessWallSequence::OuterInner => reverse_collection(collection),
        ProcessWallSequence::InnerOuterInner if layer_id == 0 => {}
        ProcessWallSequence::InnerOuterInner => {
            // `PerimeterGenerator.cpp:1470-1560` sandwich mode: reverse all,
            // then for each island bring first-internal (inset_idx==1) ahead
            // of second-internal+ (inset_idx>=2), then reorder to
            // outer-first within each island.
            reverse_collection(collection);
            if collection.entities.len() > 2 {
                sandwich_reorder(&mut collection.entities);
            }
        }
    }
}

/// `PerimeterGenerator.cpp:1483-1560` — the two-phase InnerOuterInner
/// reorder. Phase 1 pulls every `inset_idx==1` (first internal) ahead of
/// `inset_idx>=2` within each island; phase 2 walks each island reordering
/// `outer,first_internal,second_internal` so the deepest internal walls
/// print innermost-last.
fn sandwich_reorder(entities: &mut Vec<ExtrusionEntity>) {
    // Phase 1: reordered gets outer(0)+first-internal(1) in order, then the
    // skipped (>=2) appended at each island boundary; skipped accumulates
    // second-internal+ per island.
    let mut reordered: Vec<Option<ExtrusionEntity>> = Vec::with_capacity(entities.len());
    let mut skipped: Vec<Option<ExtrusionEntity>> = Vec::new();
    let mut found_second_internal = false;
    for entity in std::mem::take(entities) {
        match entity.inset_idx() {
            0 => {
                if found_second_internal {
                    reordered.append(&mut skipped);
                }
                reordered.push(Some(entity));
            }
            1 => reordered.push(Some(entity)),
            _ => {
                skipped.push(Some(entity));
                found_second_internal = true;
            }
        }
    }
    reordered.append(&mut skipped);

    // Phase 2: per-island outer/first/second-internal scan, then place the
    // island as [outer, first_internal, ...deepest-first internals...].
    let mut result: Vec<ExtrusionEntity> = Vec::with_capacity(reordered.len());
    let mut position = 0usize;
    while position < reordered.len() {
        let mut outer = -1isize;
        let mut first_internal = -1isize;
        let mut second_internal = -1isize;
        let mut max_internal = reordered.len() as isize - 1;
        let mut arr_i = position;
        while arr_i < reordered.len() {
            let inset = reordered[arr_i].as_ref().unwrap().inset_idx();
            match inset {
                0 => {
                    if outer == -1 {
                        outer = arr_i as isize;
                    }
                }
                1 => {
                    if first_internal == -1 && (arr_i as isize) > outer && outer != -1 {
                        first_internal = arr_i as isize;
                    }
                }
                2 => {
                    if second_internal == -1 && (arr_i as isize) > first_internal && outer != -1 {
                        second_internal = arr_i as isize;
                    }
                }
                _ => {}
            }
            if outer > -1 && first_internal > -1 && second_internal > -1 && inset == 0 {
                arr_i -= 1;
                max_internal = arr_i as isize;
                break;
            }
            arr_i += 1;
        }
        if outer > -1 && first_internal > -1 && second_internal > -1 {
            // inner_outer_extrusions: deepest internals
            // [second_internal..=max_internal] first (reversed), then the
            // outside-in [position..second_internal) head.
            let mut island: Vec<usize> = Vec::new();
            let mut arr_j = max_internal;
            while arr_j >= position as isize {
                if arr_j >= second_internal {
                    island.push(arr_j as usize);
                }
                arr_j -= 1;
            }
            let mut arr_j = position;
            while (arr_j as isize) < second_internal {
                island.push(arr_j);
                arr_j += 1;
            }
            for idx in island {
                result.push(reordered[idx].take().unwrap());
            }
            position = arr_i + 1;
        } else {
            break;
        }
    }
    // Entities not consumed by a completed island (a trailing island without
    // all three of outer/first/second-internal) stay in their reversed order
    // — upstream only `entities.replace`s the scanned span.
    for entity in reordered.into_iter().skip(position).flatten() {
        result.push(entity);
    }
    *entities = result;
}

fn append_nonempty(collection: ExtrusionEntityCollection) -> AppendedPerimeterCollections {
    AppendedPerimeterCollections {
        collections: if collection.entities.is_empty() {
            Vec::new()
        } else {
            vec![collection]
        },
    }
}
