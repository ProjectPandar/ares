mod polyline;
// Reached raw-path seam from OrcaSlicer v2.4.2 `PerimeterGenerator.cpp:153-207,
// 213-222`, `ExtrusionEntity.hpp:153-188,551-580`, and `Polyline.hpp:291-302`.
// Thin walls, entity traversal, and final orientation are deferred.

pub(in crate::project_slice) mod path;

pub(in crate::project_slice) mod tree;
mod types;

pub(in crate::project_slice) use types::{
    ExtrusionPath, ExtrusionRole, FittedArc, FittedMove, Point3, Polyline3,
    PreparedPostClassicRawPaths, PreparedRawPathObject, PreparedRawPathRecord,
    PreparedRawPathSurface, RawPathNode,
};

use crate::SliceError;

use super::traversal::PreparedPostClassicTraversal;

/// Layer-region traversal identity for `PerimeterGenerator.cpp:153-222`.
/// The index retains the Ares record slot around that upstream materialization boundary.
pub(super) struct RecordContext<'a> {
    pub(super) record_index: usize,
    pub(super) record: &'a super::traversal::ClassicTraversalRecord,
}

pub(in crate::project_slice) fn finish(
    prepared: Box<PreparedPostClassicTraversal>,
    context: &mut crate::SlicerContext,
) -> Result<PreparedPostClassicRawPaths, SliceError> {
    let objects = prepare_sidecars(&prepared, context)?;
    Ok(PreparedPostClassicRawPaths {
        predecessor: prepared,
        objects,
    })
}

fn prepare_sidecars(
    prepared: &PreparedPostClassicTraversal,
    context: &mut crate::SlicerContext,
) -> Result<Vec<PreparedRawPathObject>, SliceError> {
    let mut objects = Vec::with_capacity(prepared.objects.len());
    for object in &prepared.objects {
        objects.push(prepare_object(object, prepared.scale, context)?);
    }
    Ok(objects)
}

fn prepare_object(
    object: &super::traversal::PostClassicTraversalPrintObject,
    scale: crate::geometry::CoordinateScale,
    context: &mut crate::SlicerContext,
) -> Result<PreparedRawPathObject, SliceError> {
    let input = &object
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object;
    let mut records = object
        .records
        .iter()
        .map(|record| {
            record.as_ref().map(|_| PreparedRawPathRecord {
                surfaces: Vec::new(),
            })
        })
        .collect::<Vec<_>>();
    let has_fuzzy = object
        .records
        .iter()
        .flatten()
        .any(|record| record.fuzzy_skin.has_contour || record.fuzzy_skin.has_hole);
    if has_fuzzy {
        for group in &input.groups {
            context.check_canceled()?;
            let record_index = input.region_layer_record_index(group.owner);
            let record = object.records[record_index].as_ref().unwrap();
            records[record_index] = Some(prepare_record(
                object,
                record_index,
                record,
                scale,
                Some(context),
            )?);
        }
    } else {
        let built =
            crate::parallel::par_range_map_try(input.groups.len(), context.cancel_ref(), |g| {
                let record_index = input.region_layer_record_index(input.groups[g].owner);
                let record = object.records[record_index].as_ref().unwrap();
                Ok((
                    record_index,
                    prepare_record(object, record_index, record, scale, None)?,
                ))
            })?;
        for (record_index, built_record) in built {
            records[record_index] = Some(built_record);
        }
    }
    Ok(PreparedRawPathObject { records })
}

fn prepare_record(
    object: &super::traversal::PostClassicTraversalPrintObject,
    record_index: usize,
    record: &super::traversal::ClassicTraversalRecord,
    scale: crate::geometry::CoordinateScale,
    mut context: Option<&mut crate::SlicerContext>,
) -> Result<PreparedRawPathRecord, SliceError> {
    let mut surfaces = Vec::with_capacity(record.surfaces.len());
    for surface in &record.surfaces {
        surfaces.push(tree::materialize_surface(
            object,
            crate::project_slice::perimeters::classic::materialize::RecordContext {
                record_index,
                record,
            },
            surface,
            scale,
            context.as_deref_mut(),
        )?);
    }
    Ok(PreparedRawPathRecord { surfaces })
}
