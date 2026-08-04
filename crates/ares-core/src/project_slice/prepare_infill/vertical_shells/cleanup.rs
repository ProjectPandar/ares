use crate::project_slice::{
    incomplete_sink,
    prepare_infill::{
        fill_surfaces::PreparedPostFillSurfacePreparation,
        vertical_shells::types::PreparedPostVerticalShellCache,
    },
};

pub(super) fn predecessor(prepared: PreparedPostFillSurfacePreparation) {
    let PreparedPostFillSurfacePreparation {
        predecessor,
        objects,
    } = prepared;
    for object in objects {
        incomplete_sink::surface_type_detection::consume_object(object);
    }
    incomplete_sink::consume_boxed_post_classic_traversal(predecessor);
}

pub(super) fn successor(prepared: PreparedPostVerticalShellCache) {
    let PreparedPostVerticalShellCache {
        predecessor,
        objects,
        caches,
    } = prepared;
    for object in caches {
        for cache in object.records.into_iter().flatten() {
            drop(cache.top_surfaces);
            drop(cache.bottom_surfaces);
            drop(cache.holes);
        }
    }
    for object in objects {
        incomplete_sink::surface_type_detection::consume_object(object);
    }
    incomplete_sink::consume_boxed_post_classic_traversal(predecessor);
}
