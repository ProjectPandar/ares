// Classic infill-boundary construction from OrcaSlicer v2.4.2
// `PerimeterGenerator.cpp:1628-1691` and its directly reached helpers.

mod geometry;
mod preflight;

mod types;

pub(in crate::project_slice) use types::{
    PreparedInfillBoundaryObject, PreparedInfillBoundaryRecord, PreparedPostClassicInfillBoundary,
};

use crate::{
    SliceError,
    project_slice::{
        incomplete_sink,
        perimeters::classic::gap_extrusion::{
            PreparedGapExtrusionObject, PreparedGapExtrusionRecord, PreparedPostClassicGapExtrusion,
        },
    },
};
use types::{StagedObject, StagedRecord, ValidatedProject};

pub(in crate::project_slice) fn finish(
    prepared: PreparedPostClassicGapExtrusion,
) -> Result<PreparedPostClassicInfillBoundary, SliceError> {
    let validated = match preflight::validate(&prepared) {
        Ok(validated) => validated,
        Err(error) => {
            consume_predecessor(prepared);
            return Err(error);
        }
    };
    let staged = match stage(&prepared, &validated) {
        Ok(staged) => staged,
        Err(error) => {
            consume_predecessor(prepared);
            return Err(error);
        }
    };
    let PreparedPostClassicGapExtrusion {
        predecessor,
        objects,
    } = prepared;
    let objects = objects
        .into_iter()
        .zip(staged)
        .map(|(source, staged)| move_object(source, staged))
        .collect();
    Ok(PreparedPostClassicInfillBoundary {
        predecessor,
        objects,
    })
}

fn stage(
    prepared: &PreparedPostClassicGapExtrusion,
    validated: &ValidatedProject,
) -> Result<Vec<StagedObject>, SliceError> {
    prepared
        .objects
        .iter()
        .zip(&prepared.predecessor.objects)
        .zip(&validated.objects)
        .map(|((source, traversal), validated)| {
            let top_split = &traversal.predecessor.predecessor.predecessor;
            assert_eq!(source.records.len(), top_split.records.len());
            assert_eq!(source.records.len(), validated.records.len());
            let records = source
                .records
                .iter()
                .zip(&top_split.records)
                .zip(&validated.records)
                .enumerate()
                .map(
                    |(index, ((source, top), validated))| match (source, top, validated) {
                        (None, None, None) => Ok(None),
                        (Some(source), Some(top), Some(validated)) => {
                            let extra =
                                super::extra_perimeters::context(prepared, traversal, index);
                            geometry::stage_record(
                                &source.surfaces,
                                &top.surfaces,
                                validated,
                                extra.as_ref(),
                                &top_split.predecessor.records[index]
                                    .as_ref()
                                    .unwrap()
                                    .counterbore_fill,
                            )
                            .map(Some)
                        }
                        _ => panic!("O15 staged record alignment is invariant"),
                    },
                )
                .collect::<Result<Vec<_>, _>>()?;
            Ok(StagedObject { records })
        })
        .collect()
}

fn move_object(
    source: PreparedGapExtrusionObject,
    staged: StagedObject,
) -> PreparedInfillBoundaryObject {
    let records = source
        .records
        .into_iter()
        .zip(staged.records)
        .map(|(source, staged)| match (source, staged) {
            (None, None) => None,
            (Some(source), Some(staged)) => Some(move_record(source, staged)),
            _ => panic!("O15 moved record alignment is invariant"),
        })
        .collect();
    PreparedInfillBoundaryObject { records }
}

fn move_record(
    mut source: PreparedGapExtrusionRecord,
    staged: StagedRecord,
) -> PreparedInfillBoundaryRecord {
    assert_eq!(source.surfaces.len(), staged.surface_count);
    for (surface, mut extra) in source.surfaces.iter_mut().zip(staged.extra_perimeters) {
        if !extra.is_empty() {
            let collection = surface
                .appended
                .collections
                .last_mut()
                .expect("extra perimeters have an island collection");
            extra.append(&mut collection.entities);
            collection.entities = extra;
        }
    }
    PreparedInfillBoundaryRecord {
        surfaces: source.surfaces,
        fill_surfaces: staged.fill_surfaces,
        fill_no_overlap: staged.fill_no_overlap,
        overlap: staged.overlap,
    }
}

fn consume_predecessor(prepared: PreparedPostClassicGapExtrusion) {
    let PreparedPostClassicGapExtrusion {
        predecessor,
        objects,
    } = prepared;
    for object in objects {
        incomplete_sink::consume_gap_extrusion_object(object);
    }
    incomplete_sink::consume_boxed_post_classic_traversal(predecessor);
}
