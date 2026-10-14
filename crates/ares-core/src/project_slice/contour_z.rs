//! PrintObject.cpp:748-804 / ContourZ.cpp:235-243. production per-region contour stage.
mod entities;
mod mesh;
mod path;
mod ray;
#[cfg(test)]
mod tests;
mod tree;

use crate::project_slice::fill_entities::PreparedPostFillEntities;

pub(super) fn prepare(prepared: &mut PreparedPostFillEntities) {
    let combined = &prepared.predecessor;
    let external = &combined.predecessor.predecessor;
    let traversal = &external.predecessor.predecessor;
    for (object_index, layers) in prepared.objects.iter_mut().enumerate() {
        let (resolved, group) = traversal.print_object_config(object_index);
        if !resolved
            .all_regions
            .iter()
            .any(|region| region.zaa_enabled.0)
        {
            continue;
        }
        let mesh = mesh::IndexedMesh::from_object(
            &traversal.project.objects()[resolved.source_object_index],
            group,
            traversal.scale,
        );
        let prelude = &traversal.objects[object_index]
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .object;
        let (compensated, _) = prelude.as_parts();
        let (post_regions, _) = compensated.as_parts();
        let (plan, _, regions) = post_regions.as_parts();
        for (layer_index, layer) in layers.iter_mut().enumerate().skip(1) {
            entities::prepare_layer(
                layer,
                &plan.layers[layer_index],
                regions,
                &mesh,
                traversal.scale,
            );
        }
    }
}
