// Inactive post-traversal ordering and nonempty append from OrcaSlicer v2.4.2
// `PerimeterGenerator.cpp:1451-1569`.

#[cfg(test)]
mod tests;
mod types;

pub(in crate::project_slice) use types::{
    AppendedPerimeterCollections, InactiveOuterBrimReordering, InactiveOverhangReorientation,
    InactivePostCollectionBranches, InactiveWallReordering, PreparedPerimeterAppendObject,
    PreparedPerimeterAppendRecord, PreparedPerimeterAppendSurface,
    PreparedPostClassicPerimeterAppend,
};

use crate::{ObjectOptions, ProcessBrimType, ProcessWallSequence, RegionOptions};

use super::{
    entity_collections::{
        ExtrusionEntityCollection, PreparedEntityCollectionObject, PreparedEntityCollectionRecord,
        PreparedEntityCollectionSurface, PreparedPostClassicEntityCollections,
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
            transform_object(source, traversal, object_options)
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
                    let inactive = classify_inactive(traversal_record, region, object_options);
                    Some(transform_record(source, inactive))
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
) -> PreparedPerimeterAppendRecord {
    let surfaces = source
        .surfaces
        .into_iter()
        .map(|surface| transform_surface(surface, inactive))
        .collect();
    PreparedPerimeterAppendRecord { surfaces }
}

fn transform_surface(
    source: PreparedEntityCollectionSurface,
    inactive: InactivePostCollectionBranches,
) -> PreparedPerimeterAppendSurface {
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

fn classify_inactive(
    record: &ClassicTraversalRecord,
    region: &RegionOptions,
    object: &ObjectOptions,
) -> InactivePostCollectionBranches {
    assert!(!record.overhang_reverse.configured);
    assert_eq!(region.wall_sequence, ProcessWallSequence::InnerOuter);
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
        assert!(object.brim_width.0 <= 0.0);
        InactiveOuterBrimReordering::WidthNotPositive {
            brim_width: object.brim_width.0,
        }
    };
    InactivePostCollectionBranches {
        overhang_reorientation: InactiveOverhangReorientation::Disabled {
            overhang_reverse_internal_only: region.overhang_reverse_internal_only.0,
        },
        wall_reordering: InactiveWallReordering::InnerOuter { outer_brim },
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
