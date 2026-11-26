mod materialize;
mod nest;
mod types;

pub(in crate::project_slice) use types::{
    PerimeterGeneratorLoop, PostClassicHierarchyPrintObject, PreparedPostClassicHierarchy,
};

use super::onion::{PostClassicOnionPrintObject, PreparedPostClassicOnion};
use types::{ClassicHierarchyRecord, PreparedHierarchySurface};

pub(super) fn finish(
    prepared: PreparedPostClassicOnion,
    cancel: &crate::CancelToken,
) -> Result<PreparedPostClassicHierarchy, crate::SliceError> {
    let PreparedPostClassicOnion {
        project,
        resolved,
        config_block,
        scale,
        objects,
    } = prepared;
    let objects = objects
        .into_iter()
        .map(|predecessor| {
            let records = prepare_object(&predecessor, cancel)?;
            Ok(PostClassicHierarchyPrintObject {
                predecessor,
                records,
            })
        })
        .collect::<Result<Vec<_>, crate::SliceError>>()?;
    Ok(PreparedPostClassicHierarchy {
        project,
        resolved,
        config_block,
        scale,
        objects,
    })
}

fn prepare_object(
    predecessor: &PostClassicOnionPrintObject,
    cancel: &crate::CancelToken,
) -> Result<Vec<Option<ClassicHierarchyRecord>>, crate::SliceError> {
    crate::parallel::par_range_map_try(predecessor.records.len(), cancel, |index| {
        Ok(predecessor.records[index]
            .as_ref()
            .map(|record| ClassicHierarchyRecord {
                surfaces: record
                    .surfaces
                    .iter()
                    .map(|surface| {
                        let nested = nest::nest(materialize::materialize(
                            surface.effective_loop_number,
                            &surface.shells,
                            &surface.thin_wall_holes,
                            &surface.narrow_wall_holes,
                        ));
                        PreparedHierarchySurface {
                            source_index: surface.source_index,
                            roots: nested.roots,
                            remaining_contours: nested.contours,
                            remaining_holes: nested.holes,
                        }
                    })
                    .collect(),
            }))
    })
}
