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
    for group in &input.groups {
        let record_index = input.region_layer_record_index(group.owner);
        let record = object.records[record_index].as_ref().unwrap();
        records[record_index] = Some(prepare_record(
            object,
            record_index,
            record,
            scale,
            context,
        )?);
    }
    Ok(PreparedRawPathObject { records })
}

fn prepare_record(
    object: &super::traversal::PostClassicTraversalPrintObject,
    record_index: usize,
    record: &super::traversal::ClassicTraversalRecord,
    scale: crate::geometry::CoordinateScale,
    context: &mut crate::SlicerContext,
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
            context,
        )?);
    }
    Ok(PreparedRawPathRecord { surfaces })
}
