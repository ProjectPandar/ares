use super::region_slices::PostRegionPrintObject;

pub(super) fn remove_project_top_empty_layers(objects: &mut [PostRegionPrintObject]) {
    for object in objects {
        let retained = (0..object.plan.layers.len())
            .rfind(|&layer_index| {
                object
                    .regions
                    .iter()
                    .any(|region| !region.layers[layer_index].surfaces.is_empty())
            })
            .map_or(0, |layer_index| layer_index + 1);

        object.plan.layers.truncate(retained);
        for region in &mut object.regions {
            region.layers.truncate(retained);
        }
    }
}

const _: fn(&mut [PostRegionPrintObject]) = remove_project_top_empty_layers;
