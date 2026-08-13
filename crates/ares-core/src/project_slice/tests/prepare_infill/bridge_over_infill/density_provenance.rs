use crate::project_slice::{
    prepare_infill::{bridge_over_infill, external_surfaces},
    tests::support::KsrArchive,
};

#[test]
fn task22o43_stage_reads_composed_sparse_density_after_o42_geometry() {
    let horizontal =
        super::super::horizontal_shell_propagation::fixture::prepare(KsrArchive::new().bytes());
    let mut external = external_surfaces::prepare(horizontal).unwrap();
    {
        let traversal = &mut external.predecessor.predecessor;
        let prelude = &mut traversal.objects[0]
            .predecessor
            .predecessor
            .predecessor
            .predecessor;
        let (post_regions, _) = prelude.object.object.as_parts_mut();
        assert_eq!(
            post_regions.regions[0].options.sparse_infill_density.0,
            15.0
        );
        post_regions.regions[0].options.sparse_infill_density.0 = 100.0;
    }

    let prepared = bridge_over_infill::prepare(external).unwrap();
    let inventory = super::inventory_counts(&prepared);
    assert_ne!(inventory, (18, 43, 53));
    assert_eq!(inventory, (0, 0, 0));

    let traversal = &prepared.predecessor.predecessor.predecessor;
    let prelude = &traversal.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    assert_eq!(
        prelude.object.object.as_parts().0.regions[0]
            .options
            .sparse_infill_density
            .0,
        100.0
    );

    bridge_over_infill::dispose(prepared);
}
