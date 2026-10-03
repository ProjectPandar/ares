use crate::{
    OrcaFloat,
    project_slice::{
        prepare_infill::bridge_over_infill,
        region_slices::{PostRegion, RegionLayer, RegionSurfaceKind},
        tests::{prepare_infill::bridge_over_infill::prepare, support::KsrArchive},
    },
};

mod anchoring;
mod expansion;
mod foreign_candidates;
mod lightning;

#[test]
fn later_lower_region_uses_its_density_with_empty_front_lower_region() {
    for (density, keeps_bridge) in [(15.0, true), (100.0, false)] {
        let mut raw = prepare(KsrArchive::new());
        let horizontal = &mut raw.predecessor.predecessor;
        let prelude = &mut horizontal.predecessor.objects[0]
            .predecessor
            .predecessor
            .predecessor
            .predecessor;
        let layer_count = prelude.object.object.as_parts().0.plan.layers.len();
        let regions = &mut prelude.object.object.as_parts_mut().0.regions;
        let region_id = regions[0].id + 1;
        let mut options = regions[0].options.clone();
        options.sparse_infill_density.0 = density;
        regions.push(PostRegion {
            id: region_id,
            options,
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
            horizontal.objects[0].records[layer_count + layer]
                .as_mut()
                .unwrap()
                .fill_surfaces
                .iter_mut()
                .filter(|surface| surface.as_parts().0 == RegionSurfaceKind::InternalVoid)
                .for_each(|surface| surface.retag(RegionSurfaceKind::Internal));
        }
        let candidates = raw.objects[0].surfaces_by_layer.remove(&15).unwrap();
        raw.objects[0].surfaces_by_layer.clear();
        raw.objects[0].surfaces_by_layer.insert(15, candidates);

        super::prepare(&raw.predecessor, &mut raw.objects).unwrap();

        let expanded = &raw.objects[0].surfaces_by_layer[&15];
        assert_eq!(
            !expanded.is_empty(),
            keeps_bridge,
            "lower region density {density}"
        );
        assert!(
            expanded
                .iter()
                .all(|candidate| candidate.source.region_index == 0)
        );
        assert!(
            expanded
                .iter()
                .all(|candidate| !candidate.new_polygons.is_empty())
        );
        bridge_over_infill::dispose(raw);
    }
}

#[test]
fn absent_front_region_retains_later_region_bridge_and_its_configured_angle() {
    let mut raw = prepare(KsrArchive::new());
    let horizontal = &mut raw.predecessor.predecessor;
    let prelude = &mut horizontal.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let layer_count = prelude.object.object.as_parts().0.plan.layers.len();
    let regions = &mut prelude.object.object.as_parts_mut().0.regions;
    let region_id = regions[0].id + 1;
    let mut options = regions[0].options.clone();
    options.internal_bridge_angle = OrcaFloat(90.0);
    regions.push(PostRegion {
        id: region_id,
        options,
        layers: (0..layer_count)
            .map(|_| RegionLayer {
                surfaces: Vec::new(),
            })
            .collect(),
    });
    prelude.object.records.resize_with(2 * layer_count, || None);
    let mut input = prelude.object.records[15].take().unwrap();
    input.current.region_index = 1;
    input.region_id = region_id;
    prelude.object.records[layer_count + 15] = Some(input);
    horizontal.objects[0]
        .records
        .resize_with(2 * layer_count, || None);
    horizontal.objects[0].records[layer_count + 15] = horizontal.objects[0].records[15].take();
    let mut candidates = raw.objects[0].surfaces_by_layer.remove(&15).unwrap();
    for candidate in &mut candidates {
        candidate.source.region_index = 1;
    }
    raw.objects[0].surfaces_by_layer.clear();
    raw.objects[0].surfaces_by_layer.insert(15, candidates);

    super::prepare(&raw.predecessor, &mut raw.objects).unwrap();

    let expanded = &raw.objects[0].surfaces_by_layer[&15];
    assert!(!expanded.is_empty());
    for candidate in expanded {
        assert_eq!(candidate.source.layer_index, 15);
        assert_eq!(candidate.source.region_index, 1);
        assert_eq!(candidate.bridge_angle, std::f64::consts::FRAC_PI_2);
        assert!(!candidate.new_polygons.is_empty());
    }
    bridge_over_infill::dispose(raw);
}
