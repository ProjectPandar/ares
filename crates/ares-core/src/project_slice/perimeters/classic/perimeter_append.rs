// Inactive post-traversal ordering and nonempty append from OrcaSlicer v2.4.2
// `PerimeterGenerator.cpp:1451-1569`.

pub(in crate::project_slice) mod reorient;

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
                    // First physical layer, not the compatible-region
                    // record slot (`PerimeterGenerator.cpp:1858-1865`
                    // `is_outer_wall_first` / first-layer outer-brim reverse
                    // over the whole entity table, so island order and
                    // per-island wall order flip together).
                    // `outer_walls_first_for_brim` (`:3204-3211`): layer 0,
                    // `brim_width > 0`, outer-kind brim. Upstream's third
                    // kind `btLeadingEdgeOnly` (belt printers) is not
                    // decoded by Ares yet — it fails closed at config
                    // decode — so the arm is unreachable here.
                    let wall_sequence = if traversal_record.layer_id == 0
                        && matches!(
                            object_options.brim_type,
                            ProcessBrimType::OuterOnly | ProcessBrimType::OuterAndInner
                        )
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
                        traversal_record.layer_id,
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
            let roots = &reorient.record.surfaces[index].roots;
            // PerimeterGenerator.cpp:1861-1863: current first-depth contour tree.
            let narrow_wall_hole_island = roots.len() == 1
                && roots[0].children.len() == 1
                && roots[0].children[0].is_narrow_wall_hole;
            // Steep-overhang reversal runs on each surface collection
            // right after `traverse_loops` (`PerimeterGenerator.cpp:1446-1453`).
            reorient::ReorientPlan::new(reorient.record, reorient.region).apply(
                &mut surface.collection,
                roots,
                reorient.lower,
                reorient.scale,
            );
            transform_surface(
                surface,
                inactive,
                wall_sequence,
                layer_id,
                narrow_wall_hole_island,
            )
        })
        .collect();
    PreparedPerimeterAppendRecord { surfaces }
}

fn transform_surface(
    mut source: PreparedEntityCollectionSurface,
    inactive: InactivePostCollectionBranches,
    wall_sequence: ProcessWallSequence,
    layer_id: usize,
    narrow_wall_hole_island: bool,
) -> PreparedPerimeterAppendSurface {
    reorder_walls(
        &mut source.collection,
        wall_sequence,
        layer_id,
        narrow_wall_hole_island,
    );
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
    } else if !matches!(
        object.brim_type,
        ProcessBrimType::OuterOnly | ProcessBrimType::OuterAndInner
    ) {
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
    narrow_wall_hole_island: bool,
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
        // `PerimeterGenerator.cpp:1864-1865` (`is_outer_wall_first` / first-layer
        // outer-brim reverse).
        ProcessWallSequence::OuterInner => reverse_collection(collection),
        ProcessWallSequence::InnerOuterInner if layer_id == 0 || narrow_wall_hole_island => {}
        ProcessWallSequence::InnerOuterInner => {
            // `PerimeterGenerator.cpp:1867-1870`: eligible sandwich mode reverses
            // entities external-to-internal (:1868), then `sandwich_reorder`
            // pulls first-internal ahead of second-internal+ and reorders
            // each island deepest-first.
            reverse_collection(collection);
            if collection.entities.len() > 2 {
                sandwich_reorder(&mut collection.entities);
            }
        }
    }
}

/// `PerimeterGenerator.cpp:1483-1560` — the two-phase InnerOuterInner
/// reorder. Phase 1 pulls every `inset_idx==1` (first internal) ahead of
/// `inset_idx>=2` within each island; phase 2 scans each island for
/// outer/first/second-internal and emits it as deepest internals first,
/// then the outside-in head — so the second-internal and deeper walls print
/// before the outer wall and first internal.
fn sandwich_reorder(entities: &mut Vec<ExtrusionEntity>) {
    // Phase 1 (`PerimeterGenerator.cpp:1483-1507`): `reordered_idx` is the
    // permutation of `entities` indices that pulls each island's
    // `inset_idx==1` (first internal) ahead of `inset_idx>=2`; `skipped`
    // accumulates the second-internal+ and flushes at each new outer.
    let inset = |i: usize| entities[i].inset_idx();
    let mut reordered_idx: Vec<usize> = Vec::with_capacity(entities.len());
    let mut skipped_idx: Vec<usize> = Vec::new();
    let mut found_second_internal = false;
    for i in 0..entities.len() {
        match inset(i) {
            0 => {
                if found_second_internal {
                    reordered_idx.append(&mut skipped_idx);
                }
                reordered_idx.push(i);
            }
            1 => reordered_idx.push(i),
            _ => {
                skipped_idx.push(i);
                found_second_internal = true;
            }
        }
    }
    reordered_idx.append(&mut skipped_idx);

    // Phase 2 (`PerimeterGenerator.cpp:1509-1562`): per-island scan for
    // outer/first/second-internal over the reordered view; a completed
    // island is written back into `entities[position..=max_internal]` via the
    // index permutation (upstream `entities.replace`). An island that lacks
    // all three of outer/first/second-internal keeps its ORIGINAL reversed
    // order — only completed spans are replaced.
    let mut order: Vec<usize> = (0..entities.len()).collect();
    let mut position = 0usize;
    while position < reordered_idx.len() {
        let mut outer = -1isize;
        let mut first_internal = -1isize;
        let mut second_internal = -1isize;
        let mut max_internal = reordered_idx.len() as isize - 1;
        let mut arr_i = position;
        while arr_i < reordered_idx.len() {
            let cur = inset(reordered_idx[arr_i]);
            match cur {
                0 if outer == -1 => outer = arr_i as isize,
                1 if first_internal == -1 && (arr_i as isize) > outer && outer != -1 => {
                    first_internal = arr_i as isize;
                }
                2 if second_internal == -1 && (arr_i as isize) > first_internal && outer != -1 => {
                    second_internal = arr_i as isize;
                }
                _ => {}
            }
            if outer > -1 && first_internal > -1 && second_internal > -1 && cur == 0 {
                arr_i -= 1;
                max_internal = arr_i as isize;
                break;
            }
            arr_i += 1;
        }
        if outer > -1 && first_internal > -1 && second_internal > -1 {
            // inner_outer_extrusions order: deepest internals
            // [second_internal..=max_internal] first (reversed), then the
            // outside-in head [position..second_internal) — written into
            // `order[position..=max_internal]`.
            let mut island: Vec<usize> = Vec::new();
            island.extend(
                (position as isize..=max_internal)
                    .rev()
                    .filter(|&arr_j| arr_j >= second_internal)
                    .map(|arr_j| arr_j as usize),
            );
            let mut arr_j = position;
            while (arr_j as isize) < second_internal {
                island.push(arr_j);
                arr_j += 1;
            }
            for (slot, &src) in (position..=max_internal as usize).zip(island.iter()) {
                order[slot] = reordered_idx[src];
            }
            position = arr_i + 1;
        } else {
            break;
        }
    }
    // Reorder `entities` by the computed permutation (`order[k]` = source
    // index into the ORIGINAL reversed `entities`). Incomplete trailing
    // islands keep `order[k]==k` (original order), matching upstream's
    // replace-only-completed-spans.
    let mut taken: Vec<Option<ExtrusionEntity>> =
        std::mem::take(entities).into_iter().map(Some).collect();
    for slot in 0..order.len() {
        entities.push(taken[order[slot]].take().unwrap());
    }
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
