// Pre-medial Classic gap domain from OrcaSlicer v2.4.2
// `PerimeterGenerator.cpp:1573-1581,1583-1585`.

#[cfg(test)]
mod tests;
mod types;

pub(in crate::project_slice) use types::{
    PreMedialGapDomain, PreparedGapDomainObject, PreparedGapDomainRecord, PreparedGapDomainSurface,
    PreparedPostClassicGapDomain,
};

use crate::{
    SliceError,
    geometry::{ClipperError, ExPolygon, JoinType, difference_ex, offset2_ex, opening_ex},
    project_slice::incomplete_sink,
};

use super::{
    perimeter_append::{
        PreparedPerimeterAppendObject, PreparedPerimeterAppendRecord,
        PreparedPerimeterAppendSurface, PreparedPostClassicPerimeterAppend,
    },
    types::ClassicPreludeRecord,
};

const INSET_OVERLAP_TOLERANCE: f64 = 0.4;
const CLIPPER_SAFETY_OFFSET: f64 = 10.0;
const MITER_LIMIT: f64 = 3.0;

pub(in crate::project_slice) fn finish(
    prepared: PreparedPostClassicPerimeterAppend,
) -> Result<PreparedPostClassicGapDomain, SliceError> {
    let staged = match stage(&prepared) {
        Ok(staged) => staged,
        Err(error) => {
            let PreparedPostClassicPerimeterAppend {
                predecessor,
                objects,
            } = prepared;
            for object in objects {
                incomplete_sink::consume_perimeter_append_object(object);
            }
            incomplete_sink::consume_boxed_post_classic_traversal(predecessor);
            return Err(error);
        }
    };
    let PreparedPostClassicPerimeterAppend {
        predecessor,
        objects,
    } = prepared;
    let objects = objects
        .into_iter()
        .zip(staged)
        .map(|(source, staged)| move_object(source, staged))
        .collect();
    Ok(PreparedPostClassicGapDomain {
        predecessor,
        objects,
    })
}

fn stage(prepared: &PreparedPostClassicPerimeterAppend) -> Result<Vec<StagedObject>, SliceError> {
    assert_eq!(prepared.objects.len(), prepared.predecessor.objects.len());
    prepared
        .objects
        .iter()
        .zip(&prepared.predecessor.objects)
        .map(|(source, traversal)| {
            let onion = &traversal.predecessor.predecessor;
            let prelude = &onion.predecessor.predecessor;
            assert_eq!(source.records.len(), onion.records.len());
            assert_eq!(source.records.len(), prelude.records.len());
            let records = source
                .records
                .iter()
                .zip(&onion.records)
                .zip(&prelude.records)
                .map(
                    |((source, onion), prelude)| match (source, onion, prelude) {
                        (None, None, None) => Ok(None),
                        (Some(source), Some(onion), Some(prelude)) => {
                            stage_record(source, onion, prelude).map(Some)
                        }
                        _ => panic!("O11 predecessor record alignment is invariant"),
                    },
                )
                .collect::<Result<Vec<_>, SliceError>>()?;
            Ok(StagedObject { records })
        })
        .collect()
}

fn stage_record(
    source: &PreparedPerimeterAppendRecord,
    onion: &super::onion::ClassicOnionRecord,
    prelude: &ClassicPreludeRecord,
) -> Result<StagedRecord, SliceError> {
    assert_eq!(source.surfaces.len(), onion.surfaces.len());
    assert_eq!(source.surfaces.len(), prelude.surfaces.len());
    let surfaces = source
        .surfaces
        .iter()
        .zip(&onion.surfaces)
        .zip(&prelude.surfaces)
        .map(|((source, onion), prelude_surface)| {
            assert_eq!(source.source_index, onion.source_index);
            assert_eq!(source.source_index, prelude_surface.source_index);
            prepare_pre_medial(
                &onion.gaps,
                prelude.perimeter_width,
                prelude.external_width,
                prelude.perimeter_spacing,
                prelude.surface_simplify_resolution,
            )
            .map(|pre_medial| StagedSurface { pre_medial })
        })
        .collect::<Result<Vec<_>, SliceError>>()?;
    Ok(StagedRecord { surfaces })
}

fn prepare_pre_medial(
    gaps: &[ExPolygon],
    perimeter_width: i64,
    external_width: i64,
    perimeter_spacing: i64,
    surface_simplify_resolution: f64,
) -> Result<Option<PreMedialGapDomain>, SliceError> {
    if gaps.is_empty() {
        return Ok(None);
    }

    let min =
        0.2_f64 * perimeter_width.min(external_width) as f64 * (1.0_f64 - INSET_OVERLAP_TOLERANCE);
    let max = 2.0_f64 * perimeter_spacing as f64;
    let opened = opening_ex(gaps, (min / 2.0) as f32, JoinType::Miter, MITER_LIMIT)
        .map_err(geometry_error)?;
    let offset = offset2_ex(
        gaps,
        -((max / 2.0) as f32),
        (max / 2.0 + CLIPPER_SAFETY_OFFSET) as f32,
        JoinType::Miter,
        MITER_LIMIT,
    )
    .map_err(geometry_error)?;
    let mut expolygons = difference_ex(&opened, &offset).map_err(geometry_error)?;
    for expolygon in &mut expolygons {
        expolygon.douglas_peucker(surface_simplify_resolution);
    }
    Ok(Some(PreMedialGapDomain {
        min,
        max,
        expolygons,
    }))
}

fn move_object(
    source: PreparedPerimeterAppendObject,
    staged: StagedObject,
) -> PreparedGapDomainObject {
    assert_eq!(source.records.len(), staged.records.len());
    let records = source
        .records
        .into_iter()
        .zip(staged.records)
        .map(|(source, staged)| match (source, staged) {
            (None, None) => None,
            (Some(source), Some(staged)) => Some(move_record(source, staged)),
            _ => panic!("O11 staged record alignment is invariant"),
        })
        .collect();
    PreparedGapDomainObject { records }
}

fn move_record(
    source: PreparedPerimeterAppendRecord,
    staged: StagedRecord,
) -> PreparedGapDomainRecord {
    assert_eq!(source.surfaces.len(), staged.surfaces.len());
    let surfaces = source
        .surfaces
        .into_iter()
        .zip(staged.surfaces)
        .map(|(source, staged)| move_surface(source, staged))
        .collect();
    PreparedGapDomainRecord { surfaces }
}

fn move_surface(
    source: PreparedPerimeterAppendSurface,
    staged: StagedSurface,
) -> PreparedGapDomainSurface {
    PreparedGapDomainSurface {
        source_index: source.source_index,
        inactive: source.inactive,
        appended: source.appended,
        pre_medial: staged.pre_medial,
    }
}

fn geometry_error(_: ClipperError) -> SliceError {
    SliceError::InvalidInput(
        "Classic gap-domain geometry is outside the supported Clipper range".to_owned(),
    )
}

struct StagedObject {
    records: Vec<Option<StagedRecord>>,
}

struct StagedRecord {
    surfaces: Vec<StagedSurface>,
}

struct StagedSurface {
    pre_medial: Option<PreMedialGapDomain>,
}
