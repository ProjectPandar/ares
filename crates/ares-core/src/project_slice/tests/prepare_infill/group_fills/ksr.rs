use crate::{
    ExtrusionRole, OrcaBool, ProcessInfillPattern,
    project_slice::{
        group_fills::{self, SurfaceFillPattern},
        prepare_infill::combine_infill,
        region_slices::RegionSurfaceKind,
        tests::{prepare_infill::bridge_over_infill::transaction::snapshot, support::KsrArchive},
    },
};

#[test]
fn task22o73_real_ksr_layer_255_matches_owned_pre_narrow_group_without_mutation() {
    let (grouped, header) = {
        let input = super::super::combine_infill::prepare_o71(KsrArchive::new());
        let mut graph = combine_infill::prepare(input).unwrap();
        graph
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .resolved
            .objects[0]
            .object
            .detect_narrow_internal_solid_infill = OrcaBool(false);
        let external = &graph.predecessor.predecessor;
        let traversal = &external.predecessor.predecessor;
        let traversal_object = &traversal.objects[0];
        let prelude = &traversal_object
            .predecessor
            .predecessor
            .predecessor
            .predecessor;
        let (compensated, _) = prelude.object.as_parts();
        let (post_regions, _) = compensated.as_parts();
        let (plan, _, _) = post_regions.as_parts();
        let planned = &plan.layers[255];
        let header = super::oracle::LayerHeader {
            id: planned.id,
            height: planned.height,
            print_z: planned.print_z,
        };
        let before = snapshot(&graph.predecessor);

        let grouped = group_fills::group_fills(external, 0, 255).unwrap();

        let after = snapshot(&graph.predecessor);
        assert_eq!(after.bytes, before.bytes);
        assert_eq!(after.bridge_layers, before.bridge_layers);
        assert_eq!(after.bridge_surfaces, before.bridge_surfaces);
        assert_eq!(
            after.bridge_expolygon_points,
            before.bridge_expolygon_points
        );
        combine_infill::dispose(graph);
        (grouped, header)
    };

    assert_eq!(header.id, 255);
    assert!(grouped.lock_region_param.skin_density_params.is_empty());
    assert!(grouped.lock_region_param.skeleton_density_params.is_empty());
    assert!(grouped.lock_region_param.skin_flow_params.is_empty());
    assert!(grouped.lock_region_param.skeleton_flow_params.is_empty());
    assert_eq!(grouped.surface_fills.len(), 1);

    let fill = &grouped.surface_fills[0];
    assert_eq!(fill.region_id, 0);
    assert_eq!(fill.representative.kind, RegionSurfaceKind::InternalBridge);
    assert_eq!(
        fill.params.pattern,
        SurfaceFillPattern::Configured(ProcessInfillPattern::Monotonic)
    );
    assert_eq!(
        fill.params.extrusion_role,
        ExtrusionRole::InternalBridgeInfill
    );
    assert_eq!(fill.region_id_group, [0]);
    assert_eq!(fill.expolygons.len(), 1);
    assert!(fill.expolygons[0].holes().is_empty());
    assert_eq!(fill.no_overlap_expolygons.len(), 1);
    assert!(fill.no_overlap_expolygons[0].holes().is_empty());
}
