use crate::{
    ExtrusionRole, ProcessInfillPattern,
    project_slice::{
        fill_entities::generate_layer,
        prepare_infill::combine_infill,
        region_slices::RegionSurfaceKind,
        tests::prepare_infill::group_fills::focused::fixture::{
            external, graph, graph_snapshot, options_mut, record_mut, rectangle, surface,
        },
    },
};

const LAYER: usize = 1;

#[test]
fn task22o90_monotonic_internal_solid_becomes_ordered_flow_entities() {
    let mut graph = graph();
    record_mut(&mut graph, LAYER)
        .fill_no_overlap_expolygons
        .clear();
    record_mut(&mut graph, LAYER).fill_surfaces = vec![surface(
        RegionSurfaceKind::InternalSolid,
        rectangle(0, 0, 12_000_000, 8_000_000),
        0,
    )];
    options_mut(&mut graph, LAYER).internal_solid_infill_pattern = ProcessInfillPattern::Monotonic;
    let before = graph_snapshot(&graph);

    let first = generate_layer(external(&graph), 0, LAYER).unwrap();
    let second = generate_layer(external(&graph), 0, LAYER).unwrap();

    assert_eq!(first, second);
    assert_eq!(graph_snapshot(&graph).bytes, before.bytes);
    assert_eq!(first.collections.len(), 1);
    assert!(!first.collections[0].paths.is_empty());
    assert!(
        first.collections[0]
            .paths
            .iter()
            .all(|path| path.role == ExtrusionRole::SolidInfill && path.polyline.is_valid())
    );
    assert_eq!(
        first.collections[0]
            .paths
            .iter()
            .map(|path| (
                path.mm3_per_mm.to_bits(),
                path.width.to_bits(),
                path.height.to_bits(),
                path.polyline.points().len(),
            ))
            .collect::<Vec<_>>(),
        vec![(4_590_098_710_712_549_376, 1_054_280_253, 1_045_220_557, 70)]
    );
    combine_infill::dispose(graph);
}

#[test]
fn task22o90_monotonicline_top_surface_keeps_lines_disconnected() {
    let mut graph = graph();
    record_mut(&mut graph, LAYER)
        .fill_no_overlap_expolygons
        .clear();
    record_mut(&mut graph, LAYER).fill_surfaces = vec![surface(
        RegionSurfaceKind::Top,
        rectangle(0, 0, 12_000_000, 8_000_000),
        0,
    )];
    options_mut(&mut graph, LAYER).top_surface_pattern = ProcessInfillPattern::MonotonicLine;

    let entities = generate_layer(external(&graph), 0, LAYER).unwrap();

    assert_eq!(entities.collections.len(), 1);
    assert!(entities.collections[0].paths.len() > 1);
    assert!(
        entities.collections[0]
            .paths
            .iter()
            .all(|path| path.role == ExtrusionRole::TopSolidInfill)
    );
    combine_infill::dispose(graph);
}
