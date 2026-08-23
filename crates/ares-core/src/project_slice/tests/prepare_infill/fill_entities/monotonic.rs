use crate::{
    ExtrusionRole, ProcessInfillPattern,
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
    assert!(first.collections[0].no_sort);
    assert!(!first.collections[0].entities.is_empty());
    assert!(first.collections[0].entities.iter().all(|entity| {
        matches!(
            entity,
            FillExtrusionEntity::Path(path)
                if path.role == ExtrusionRole::SolidInfill && path.polyline.is_valid()
        )
    }));
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
    assert!(entities.collections[0].no_sort);
    assert!(entities.collections[0].entities.len() > 1);
    assert!(entities.collections[0].entities.iter().all(|entity| {
        matches!(
            entity,
            FillExtrusionEntity::Path(path) if path.role == ExtrusionRole::TopSolidInfill
        )
    }));
    combine_infill::dispose(graph);
}
