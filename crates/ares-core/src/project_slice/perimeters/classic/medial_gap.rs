// Classic gap medial axis from OrcaSlicer v2.4.2
// `PerimeterGenerator.cpp:1586`, `ExPolygon.cpp:261-369`, and
// `Geometry/MedialAxis.cpp:458-707`.

mod types;

pub(in crate::project_slice) use types::{
    MedialGapDomain, PreparedMedialGapObject, PreparedMedialGapRecord, PreparedMedialGapSurface,
    PreparedPostClassicMedialGap,
};

use crate::{
    SliceError,
    geometry::{CoordinateScale, ThickPolyline, medial_axis},
    project_slice::incomplete_sink,
};

use super::gap_domain::{
    PreparedGapDomainObject, PreparedGapDomainRecord, PreparedGapDomainSurface,
    PreparedPostClassicGapDomain,
};

const VORONOI_ERROR: &str = "Classic medial-axis Voronoi construction failed";

#[cfg(test)]
thread_local! {
    static FINISH_INVOCATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static ERROR_CLEANUP_PROBE_ALIVE: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

#[cfg(test)]
pub(in crate::project_slice) fn finish_invocations() -> usize {
    FINISH_INVOCATIONS.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(in crate::project_slice) fn error_cleanup_probe_alive() -> Option<bool> {
    ERROR_CLEANUP_PROBE_ALIVE.with(std::cell::Cell::get)
}

pub(in crate::project_slice) fn finish(
    prepared: PreparedPostClassicGapDomain,
) -> Result<PreparedPostClassicMedialGap, SliceError> {
    #[cfg(test)]
    FINISH_INVOCATIONS.with(|invocations| invocations.set(invocations.get() + 1));

    let staged = match stage(&prepared) {
        Ok(staged) => staged,
        Err(error) => {
            #[cfg(test)]
            ERROR_CLEANUP_PROBE_ALIVE.with(|alive| {
                alive.set(Some(prepared.predecessor.drop_probe_is_alive()));
            });
            let PreparedPostClassicGapDomain {
                predecessor,
                objects,
            } = prepared;
            for object in objects {
                incomplete_sink::consume_gap_domain_object(object);
            }
            incomplete_sink::consume_boxed_post_classic_traversal(predecessor);
            return Err(error);
        }
    };
    let PreparedPostClassicGapDomain {
        predecessor,
        objects,
    } = prepared;
    let objects = objects
        .into_iter()
        .zip(staged)
        .map(|(source, staged)| move_object(source, staged))
        .collect();
    Ok(PreparedPostClassicMedialGap {
        predecessor,
        objects,
    })
}

fn stage(prepared: &PreparedPostClassicGapDomain) -> Result<Vec<StagedObject>, SliceError> {
    let scale = prepared.predecessor.scale;
    prepared
        .objects
        .iter()
        .map(|object| {
            let records = object
                .records
                .iter()
                .map(|record| {
                    record
                        .as_ref()
                        .map(|record| stage_record(record, scale))
                        .transpose()
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(StagedObject { records })
        })
        .collect()
}

fn stage_record(
    record: &PreparedGapDomainRecord,
    scale: CoordinateScale,
) -> Result<StagedRecord, SliceError> {
    let surfaces = record
        .surfaces
        .iter()
        .map(|surface| stage_surface(surface, scale))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(StagedRecord { surfaces })
}

fn stage_surface(
    surface: &PreparedGapDomainSurface,
    scale: CoordinateScale,
) -> Result<StagedSurface, SliceError> {
    let polylines = surface
        .pre_medial
        .as_ref()
        .map(|domain| {
            let mut output = Vec::new();
            for expolygon in &domain.expolygons {
                output.extend(
                    medial_axis(expolygon, domain.min, domain.max, scale)
                        .map_err(|_| SliceError::InvalidInput(VORONOI_ERROR.to_owned()))?,
                );
            }
            Ok(output)
        })
        .transpose()?;
    Ok(StagedSurface { polylines })
}

fn move_object(source: PreparedGapDomainObject, staged: StagedObject) -> PreparedMedialGapObject {
    let records = source
        .records
        .into_iter()
        .zip(staged.records)
        .map(|(source, staged)| match (source, staged) {
            (Some(source), Some(staged)) => Some(move_record(source, staged)),
            (None, None) => None,
            _ => panic!("O13 staged record alignment is invariant"),
        })
        .collect();
    PreparedMedialGapObject { records }
}

fn move_record(source: PreparedGapDomainRecord, staged: StagedRecord) -> PreparedMedialGapRecord {
    let surfaces = source
        .surfaces
        .into_iter()
        .zip(staged.surfaces)
        .map(|(source, staged)| move_surface(source, staged))
        .collect();
    PreparedMedialGapRecord { surfaces }
}

fn move_surface(
    source: PreparedGapDomainSurface,
    staged: StagedSurface,
) -> PreparedMedialGapSurface {
    let medial = match (source.pre_medial, staged.polylines) {
        (Some(predecessor), Some(polylines)) => Some(MedialGapDomain {
            predecessor,
            polylines,
        }),
        (None, None) => None,
        _ => panic!("O13 staged surface alignment is invariant"),
    };
    PreparedMedialGapSurface {
        source_index: source.source_index,
        inactive: source.inactive,
        appended: source.appended,
        medial,
    }
}

struct StagedObject {
    records: Vec<Option<StagedRecord>>,
}

struct StagedRecord {
    surfaces: Vec<StagedSurface>,
}

struct StagedSurface {
    polylines: Option<Vec<ThickPolyline>>,
}
