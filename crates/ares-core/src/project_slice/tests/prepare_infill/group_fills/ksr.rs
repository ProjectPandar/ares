use crate::{
    ExtrusionRole, ProcessInfillPattern,
    project_slice::{
        group_fills::{self, SurfaceFillPattern},
        prepare_infill::combine_infill,
        region_slices::RegionSurfaceKind,
        tests::{
            prepare_infill::bridge_over_infill::transaction::{sha256, snapshot},
            support::KsrArchive,
        },
    },
};

const O71_SURFACE_SHA256: &str = "c547cb34b8d5d27d572a166f13a16741f75f7f9d34f15db59ddac8575b5a33b9";
const LAYER_255_METADATA_SHA256: &str =
    "4e07d77b1730480a2d2a0a69a082c1062c532747b42f19ad4a4bd92f4c094f2b";
const LAYER_255_AUTHORITATIVE_GEOMETRY_SHA256: &str =
    "46ce8d7f622341235996928aeb0e86f8cc585568b712324f80b429e2411b4d56";

#[test]
fn task22o73_real_ksr_layer_255_matches_owned_pre_narrow_group_without_mutation() {
    let (grouped, header) = {
        let input = super::super::combine_infill::prepare_o71(KsrArchive::new());
        let graph = combine_infill::prepare(input).unwrap();
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
        assert_eq!(sha256(&before.bytes), O71_SURFACE_SHA256);

        let grouped = group_fills::group_fills_base(external, 0, 255).unwrap();

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
    assert_eq!(header.height.to_bits(), 4_596_373_779_694_328_320);
    assert_eq!(header.print_z.to_bits(), 4_632_402_576_713_292_212);
    assert!(!grouped.has_internal_voids);
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
    assert_eq!(fill.expolygons[0].contour().points().len(), 11);
    assert!(fill.expolygons[0].holes().is_empty());
    assert_eq!(fill.no_overlap_expolygons.len(), 1);
    assert_eq!(fill.no_overlap_expolygons[0].contour().points().len(), 11);
    assert!(fill.no_overlap_expolygons[0].holes().is_empty());

    assert_eq!(
        sha256(&super::oracle::metadata(header, &grouped)),
        LAYER_255_METADATA_SHA256
    );
    assert_eq!(
        sha256(&super::oracle::authoritative_geometry(&grouped)),
        LAYER_255_AUTHORITATIVE_GEOMETRY_SHA256
    );
}
