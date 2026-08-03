// Reached raw-path seam from OrcaSlicer v2.4.2 `PerimeterGenerator.cpp:153-207,
// 213-222`, `ExtrusionEntity.hpp:153-188,551-580`, and `Polyline.hpp:291-302`.
// Thin walls, entity traversal, and final orientation are deferred.

pub(in crate::project_slice) mod path;
#[cfg(test)]
mod tests;
pub(in crate::project_slice) mod tree;
mod types;

pub(in crate::project_slice) use types::{
    ExtrusionPath, ExtrusionRole, Point3, Polyline3, PreparedPostClassicRawPaths,
    PreparedRawPathObject, PreparedRawPathRecord, PreparedRawPathSurface, RawPathNode,
};

use crate::{SliceError, project_slice::incomplete_sink};

use super::traversal::PreparedPostClassicTraversal;

pub(in crate::project_slice) fn finish(
    prepared: Box<PreparedPostClassicTraversal>,
) -> Result<PreparedPostClassicRawPaths, SliceError> {
    let objects = match prepare_sidecars(&prepared) {
        Ok(objects) => objects,
        Err(error) => {
            incomplete_sink::consume_boxed_post_classic_traversal(prepared);
            return Err(error);
        }
    };
    Ok(PreparedPostClassicRawPaths {
        predecessor: prepared,
        objects,
    })
}

fn prepare_sidecars(
    prepared: &PreparedPostClassicTraversal,
) -> Result<Vec<PreparedRawPathObject>, SliceError> {
    let mut objects = Vec::with_capacity(prepared.objects.len());
    for object in &prepared.objects {
        match prepare_object(object, prepared.scale) {
            Ok(object) => objects.push(object),
            Err(error) => {
                consume_objects(objects);
                return Err(error);
            }
        }
    }
    Ok(objects)
}

fn prepare_object(
    object: &super::traversal::PostClassicTraversalPrintObject,
    scale: crate::geometry::CoordinateScale,
) -> Result<PreparedRawPathObject, SliceError> {
    let mut records = Vec::with_capacity(object.records.len());
    for (record_index, record) in object.records.iter().enumerate() {
        let Some(record) = record else {
            records.push(None);
            continue;
        };
        match prepare_record(object, record_index, record, scale) {
            Ok(record) => records.push(Some(record)),
            Err(error) => {
                consume_records(records);
                return Err(error);
            }
        }
    }
    Ok(PreparedRawPathObject { records })
}

fn prepare_record(
    object: &super::traversal::PostClassicTraversalPrintObject,
    record_index: usize,
    record: &super::traversal::ClassicTraversalRecord,
    scale: crate::geometry::CoordinateScale,
) -> Result<PreparedRawPathRecord, SliceError> {
    let mut surfaces = Vec::with_capacity(record.surfaces.len());
    for surface in &record.surfaces {
        match tree::materialize_surface(object, record_index, record, surface, scale) {
            Ok(surface) => surfaces.push(surface),
            Err(error) => {
                consume_surfaces(surfaces);
                return Err(error);
            }
        }
    }
    Ok(PreparedRawPathRecord { surfaces })
}

fn consume_surfaces(surfaces: Vec<PreparedRawPathSurface>) {
    for surface in surfaces {
        tree::consume_nodes(surface.roots);
    }
}

fn consume_records(records: Vec<Option<PreparedRawPathRecord>>) {
    for record in records.into_iter().flatten() {
        consume_surfaces(record.surfaces);
    }
}

fn consume_objects(objects: Vec<PreparedRawPathObject>) {
    for object in objects {
        consume_records(object.records);
    }
}
