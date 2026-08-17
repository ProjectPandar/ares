// Classic gap extrusion from OrcaSlicer v2.4.2 `PerimeterGenerator.cpp:1604-1624`
// and its reached variable-width, flow, entity, and open-offset helpers.

pub(in crate::project_slice) mod coverage;
mod entity;
mod preflight;
#[cfg(test)]
mod tests;
mod types;
mod variable_width;

pub(in crate::project_slice) use entity::{GapFillCollection, GapFillEntity};
pub(in crate::project_slice) use types::{
    PreparedGapExtrusionObject, PreparedGapExtrusionRecord, PreparedGapExtrusionSurface,
    PreparedPostClassicGapExtrusion,
};

use crate::{
    SliceError,
    geometry::{ExPolygon, ThickPolyline, difference_ex_polygons},
    project_slice::incomplete_sink,
};

use super::{
    materialize::{ExtrusionPath, ExtrusionRole, Point3, Polyline3},
    medial_gap::{
        PreparedMedialGapObject, PreparedMedialGapRecord, PreparedMedialGapSurface,
        PreparedPostClassicMedialGap,
    },
};
use preflight::{ValidatedObject, ValidatedRecord};

const FLOW_ERROR: &str = "Classic variable-width gap flow is invalid";
const GEOMETRY_ERROR: &str =
    "Classic gap-extrusion geometry is outside the supported Clipper range";

#[cfg(test)]
thread_local! {
    static STAGE_SURFACE_INVOCATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(in crate::project_slice) fn reset_stage_surface_invocations() {
    STAGE_SURFACE_INVOCATIONS.with(|count| count.set(0));
}

#[cfg(test)]
pub(in crate::project_slice) fn stage_surface_invocations() -> usize {
    STAGE_SURFACE_INVOCATIONS.with(std::cell::Cell::get)
}

pub(in crate::project_slice) fn finish(
    prepared: PreparedPostClassicMedialGap,
) -> Result<PreparedPostClassicGapExtrusion, SliceError> {
    let scale = prepared.predecessor.scale;
    let validated = match preflight::validate(&prepared, scale) {
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
    let PreparedPostClassicMedialGap {
        predecessor,
        objects,
    } = prepared;
    let objects = objects
        .into_iter()
        .zip(staged)
        .map(|(source, staged)| move_object(source, staged))
        .collect();
    Ok(PreparedPostClassicGapExtrusion {
        predecessor,
        objects,
    })
}

fn stage(
    prepared: &PreparedPostClassicMedialGap,
    validated: &[ValidatedObject],
) -> Result<Vec<StagedObject>, SliceError> {
    let scale = prepared.predecessor.scale;
    prepared
        .objects
        .iter()
        .zip(&prepared.predecessor.objects)
        .zip(validated)
        .map(|((source, traversal), validated)| {
            let onion = &traversal.predecessor.predecessor;
            assert_eq!(source.records.len(), onion.records.len());
            assert_eq!(source.records.len(), validated.records.len());
            let records = source
                .records
                .iter()
                .zip(&onion.records)
                .zip(&validated.records)
                .map(
                    |((source, onion), validated)| match (source, onion, validated) {
                        (None, None, None) => Ok(None),
                        (Some(source), Some(onion), Some(validated)) => {
                            stage_record(source, onion, *validated, scale).map(Some)
                        }
                        _ => panic!("O14 predecessor record alignment is invariant"),
                    },
                )
                .collect::<Result<Vec<_>, _>>()?;
            Ok(StagedObject { records })
        })
        .collect()
}

fn stage_record(
    source: &PreparedMedialGapRecord,
    onion: &super::onion::ClassicOnionRecord,
    validated: ValidatedRecord,
    scale: crate::geometry::CoordinateScale,
) -> Result<StagedRecord, SliceError> {
    assert_eq!(source.surfaces.len(), onion.surfaces.len());
    let surfaces = source
        .surfaces
        .iter()
        .zip(&onion.surfaces)
        .map(|(source, onion)| {
            assert_eq!(source.source_index, onion.source_index);
            stage_surface(source, &onion.last, validated, scale)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(StagedRecord { surfaces })
}

fn stage_surface(
    source: &PreparedMedialGapSurface,
    last: &[ExPolygon],
    validated: ValidatedRecord,
    scale: crate::geometry::CoordinateScale,
) -> Result<StagedSurface, SliceError> {
    #[cfg(test)]
    STAGE_SURFACE_INVOCATIONS.with(|count| count.set(count.get() + 1));
    let keep = source
        .medial
        .as_ref()
        .map(|medial| retention_mask(&medial.polylines, validated.threshold));
    let retained = source
        .medial
        .as_ref()
        .zip(keep.as_ref())
        .map(|(medial, keep)| {
            medial
                .polylines
                .iter()
                .zip(keep)
                .filter(|(_, keep)| **keep)
                .map(|(polyline, _)| polyline.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let gap_fill = variable_width::convert(&retained, validated.flow, scale)
        .map_err(|_| SliceError::InvalidInput(FLOW_ERROR.to_owned()))?;
    let covered = coverage::covered_polygons(&gap_fill, scale)
        .map_err(|_| SliceError::InvalidInput(GEOMETRY_ERROR.to_owned()))?;
    let remaining = if retained.is_empty() {
        last.to_vec()
    } else {
        difference_ex_polygons(last, &covered)
            .map_err(|_| SliceError::InvalidInput(GEOMETRY_ERROR.to_owned()))?
    };
    Ok(StagedSurface {
        keep,
        gap_fill,
        remaining,
    })
}

pub(super) fn retention_mask(polylines: &[ThickPolyline], threshold: f64) -> Vec<bool> {
    polylines
        .iter()
        .map(|polyline| polyline.length() >= threshold)
        .collect()
}

fn move_object(
    source: PreparedMedialGapObject,
    staged: StagedObject,
) -> PreparedGapExtrusionObject {
    let records = source
        .records
        .into_iter()
        .zip(staged.records)
        .map(|(source, staged)| match (source, staged) {
            (None, None) => None,
            (Some(source), Some(staged)) => Some(move_record(source, staged)),
            _ => panic!("O14 staged record alignment is invariant"),
        })
        .collect();
    PreparedGapExtrusionObject { records }
}

fn move_record(
    source: PreparedMedialGapRecord,
    staged: StagedRecord,
) -> PreparedGapExtrusionRecord {
    let surfaces = source
        .surfaces
        .into_iter()
        .zip(staged.surfaces)
        .map(|(source, staged)| move_surface(source, staged))
        .collect();
    PreparedGapExtrusionRecord { surfaces }
}

fn move_surface(
    source: PreparedMedialGapSurface,
    staged: StagedSurface,
) -> PreparedGapExtrusionSurface {
    let medial = match (source.medial, staged.keep) {
        (None, None) => None,
        (Some(mut medial), Some(keep)) => {
            medial.polylines = medial
                .polylines
                .into_iter()
                .zip(keep)
                .filter_map(|(polyline, keep)| keep.then_some(polyline))
                .collect();
            Some(medial)
        }
        _ => panic!("O14 medial alignment is invariant"),
    };
    PreparedGapExtrusionSurface {
        source_index: source.source_index,
        inactive: source.inactive,
        appended: source.appended,
        medial,
        gap_fill: staged.gap_fill,
        remaining: staged.remaining,
    }
}

fn consume_predecessor(prepared: PreparedPostClassicMedialGap) {
    let PreparedPostClassicMedialGap {
        predecessor,
        objects,
    } = prepared;
    for object in objects {
        incomplete_sink::consume_medial_gap_object(object);
    }
    incomplete_sink::consume_boxed_post_classic_traversal(predecessor);
}

struct StagedObject {
    records: Vec<Option<StagedRecord>>,
}
struct StagedRecord {
    surfaces: Vec<StagedSurface>,
}
struct StagedSurface {
    keep: Option<Vec<bool>>,
    gap_fill: GapFillCollection,
    remaining: Vec<ExPolygon>,
}

type VariableWidthFn = fn(
    &[ThickPolyline],
    crate::project_slice::perimeters::types::Flow,
    crate::geometry::CoordinateScale,
) -> Result<GapFillCollection, SliceError>;
const _: VariableWidthFn = variable_width::convert;
