use crate::{
    ProcessInfillPattern,
    project_slice::{
        fill_entities::{FillExtrusionEntity, generate_layer},
        prepare_infill::combine_infill,
        region_slices::RegionSurfaceKind,
        tests::prepare_infill::group_fills::focused::fixture::{
            external, graph, graph_snapshot, options_mut, record_mut, rectangle, surface,
        },
    },
};

const LAYER: usize = 1;

#[test]
fn task22o76_crosshatch_group_becomes_owned_flow_annotated_entities() {
    let mut graph = graph();
    record_mut(&mut graph, LAYER)
        .fill_no_overlap_expolygons
        .clear();
    record_mut(&mut graph, LAYER).fill_surfaces = vec![surface(
        RegionSurfaceKind::Internal,
        rectangle(0, 0, 12_000_000, 8_000_000),
        0,
    )];
    options_mut(&mut graph, LAYER).sparse_infill_pattern = ProcessInfillPattern::CrossHatch;
    let before = graph_snapshot(&graph);

    let first = generate_layer(external(&graph), 0, LAYER).unwrap();
    let second = generate_layer(external(&graph), 0, LAYER).unwrap();

    assert_eq!(first, second);
    assert_eq!(graph_snapshot(&graph).bytes, before.bytes);
    assert_eq!(first.collections.len(), 1);
    assert!(!first.collections[0].no_sort);
    assert!(!first.collections[0].entities.is_empty());
    assert!(
        first.collections[0]
            .entities
            .iter()
            .all(|entity| match entity {
                FillExtrusionEntity::Path(path) => {
                    path.role == crate::ExtrusionRole::InternalInfill
                        && path.mm3_per_mm.to_bits() == 0x3fb4_d7ac_a000_0000
                        && path.width.to_bits() == 0x3ee6_6666
                        && path.height.to_bits() == 0x3e4c_cccd
                        && path.polyline.points().len() >= 2
                }
                FillExtrusionEntity::VariableWidth(_) => false,
            })
    );
    combine_infill::dispose(graph);
}
