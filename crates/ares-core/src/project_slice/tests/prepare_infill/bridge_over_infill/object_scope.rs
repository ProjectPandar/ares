use crate::{
    ProcessInfillPattern,
    project_slice::{
        prepare_infill::{bridge_over_infill, external_surfaces},
        region_slices::{PostRegion, RegionLayer},
        tests::support::KsrArchive,
    },
};

#[test]
fn task22o43_object_lightning_config_survives_absent_layer_records() {
    let archive = KsrArchive::new();
    let horizontal = super::super::horizontal_shell_propagation::fixture::prepare(archive.bytes());
    let mut external = external_surfaces::prepare(horizontal).unwrap();
    let (horizontal_records, prelude_records, inputs) = {
        let horizontal = &mut external.predecessor;
        let horizontal_records = horizontal.objects[0]
            .records
            .iter_mut()
            .map(Option::take)
            .collect::<Vec<_>>();
        let traversal = &mut horizontal.predecessor.objects[0];
        let prelude = &mut traversal.predecessor.predecessor.predecessor.predecessor;
        let prelude_records = prelude
            .records
            .iter_mut()
            .map(Option::take)
            .collect::<Vec<_>>();
        let inputs = prelude
            .object
            .records
            .iter_mut()
            .map(Option::take)
            .collect::<Vec<_>>();
        let post_regions = prelude.object.object.as_parts_mut().0;
        let first = &post_regions.regions[0];
        let second_id = first.id + 1;
        let third_id = second_id + 1;
        let mut second_options = first.options.clone();
        let third_options = first.options.clone();
        let layer_count = first.layers.len();
        second_options.sparse_infill_pattern = ProcessInfillPattern::Lightning;
        post_regions.regions.push(PostRegion {
            id: second_id,
            options: second_options,
            layers: (0..layer_count)
                .map(|_| RegionLayer {
                    surfaces: Vec::new(),
                })
                .collect(),
        });
        post_regions.regions.push(PostRegion {
            id: third_id,
            options: third_options,
            layers: (0..layer_count)
                .map(|_| RegionLayer {
                    surfaces: Vec::new(),
                })
                .collect(),
        });
        (horizontal_records, prelude_records, inputs)
    };

    let mut prepared = bridge_over_infill::prepare(external).unwrap();

    assert!(prepared.objects[0].has_lightning_infill);
    assert!(prepared.objects[0].surfaces_by_layer.is_empty());
    let traversal = &prepared.predecessor.predecessor.predecessor;
    assert_eq!(
        traversal
            .resolved
            .views
            .full
            .process
            .region
            .sparse_infill_pattern,
        ProcessInfillPattern::CrossHatch
    );
    let prelude = &traversal.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    assert_eq!(
        prelude
            .object
            .object
            .as_parts()
            .0
            .regions
            .iter()
            .map(|region| region.options.sparse_infill_pattern)
            .collect::<Vec<_>>(),
        vec![
            ProcessInfillPattern::CrossHatch,
            ProcessInfillPattern::Lightning,
            ProcessInfillPattern::CrossHatch,
        ]
    );

    {
        let horizontal = &mut prepared.predecessor.predecessor;
        for (slot, record) in horizontal.objects[0]
            .records
            .iter_mut()
            .zip(horizontal_records)
        {
            *slot = record;
        }
        let traversal = &mut horizontal.predecessor.objects[0];
        let prelude = &mut traversal.predecessor.predecessor.predecessor.predecessor;
        for (slot, record) in prelude.records.iter_mut().zip(prelude_records) {
            *slot = record;
        }
        for (slot, input) in prelude.object.records.iter_mut().zip(inputs) {
            *slot = input;
        }
    }
    bridge_over_infill::dispose(prepared);
}
