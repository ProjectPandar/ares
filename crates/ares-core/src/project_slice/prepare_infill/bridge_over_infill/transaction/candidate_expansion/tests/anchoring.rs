use super::*;

#[test]
fn unchanged_lower_geometry_in_later_region_preserves_expanded_bridges() {
    let baseline = expanded_bridges(false);
    assert!(!baseline.is_empty());
    assert_eq!(expanded_bridges(true), baseline);
}

fn expanded_bridges(move_lower_region: bool) -> Vec<(f64, Vec<crate::geometry::Polygon>)> {
    let mut raw = prepare(KsrArchive::new());
    if move_lower_region {
        let horizontal = &mut raw.predecessor.predecessor;
        let prelude = &mut horizontal.predecessor.objects[0]
            .predecessor
            .predecessor
            .predecessor
            .predecessor;
        let layer_count = prelude.object.object.as_parts().0.plan.layers.len();
        let regions = &mut prelude.object.object.as_parts_mut().0.regions;
        let region_id = regions[0].id + 1;
        regions.push(PostRegion {
            id: region_id,
            options: regions[0].options.clone(),
            layers: (0..layer_count)
                .map(|_| RegionLayer {
                    surfaces: Vec::new(),
                })
                .collect(),
        });
        prelude.object.records.resize_with(2 * layer_count, || None);
        horizontal.objects[0]
            .records
            .resize_with(2 * layer_count, || None);
        for layer in 0..15 {
            let mut input = prelude.object.records[layer].take().unwrap();
            input.current.region_index = 1;
            input.region_id = region_id;
            prelude.object.records[layer_count + layer] = Some(input);
            horizontal.objects[0].records[layer_count + layer] =
                horizontal.objects[0].records[layer].take();
        }
    }
    let candidates = raw.objects[0].surfaces_by_layer.remove(&15).unwrap();
    raw.objects[0].surfaces_by_layer.clear();
    raw.objects[0].surfaces_by_layer.insert(15, candidates);

    super::super::prepare(&raw.predecessor, &mut raw.objects).unwrap();

    let snapshot = raw.objects[0].surfaces_by_layer[&15]
        .iter()
        .map(|candidate| (candidate.bridge_angle, candidate.new_polygons.clone()))
        .collect();
    bridge_over_infill::dispose(raw);
    snapshot
}
