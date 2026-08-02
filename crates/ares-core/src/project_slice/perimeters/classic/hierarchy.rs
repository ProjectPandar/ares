mod materialize;
mod nest;
mod types;

pub(in crate::project_slice) use types::{
    PerimeterGeneratorLoop, PostClassicHierarchyPrintObject, PreparedPostClassicHierarchy,
};

use super::onion::{PostClassicOnionPrintObject, PreparedPostClassicOnion};
use types::{ClassicHierarchyRecord, PreparedHierarchySurface};

pub(super) fn finish(prepared: PreparedPostClassicOnion) -> PreparedPostClassicHierarchy {
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
            let records = prepare_object(&predecessor);
            PostClassicHierarchyPrintObject {
                predecessor,
                records,
            }
        })
        .collect();
    PreparedPostClassicHierarchy {
        project,
        resolved,
        config_block,
        scale,
        objects,
    }
}

fn prepare_object(
    predecessor: &PostClassicOnionPrintObject,
) -> Vec<Option<ClassicHierarchyRecord>> {
    predecessor
        .records
        .iter()
        .map(|record| {
            record.as_ref().map(|record| ClassicHierarchyRecord {
                surfaces: record
                    .surfaces
                    .iter()
                    .map(|surface| {
                        let nested = nest::nest(materialize::materialize(
                            surface.effective_loop_number,
                            &surface.shells,
                        ));
                        PreparedHierarchySurface {
                            source_index: surface.source_index,
                            roots: nested.roots,
                            remaining_contours: nested.contours,
                            remaining_holes: nested.holes,
                        }
                    })
                    .collect(),
            })
        })
        .collect()
}
