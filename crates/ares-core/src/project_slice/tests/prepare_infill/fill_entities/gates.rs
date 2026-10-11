use crate::{
    ProcessInfillPattern, SliceError,
    project_slice::{
        fill_entities::generate_layer,
        prepare_infill::combine_infill,
        region_slices::RegionSurfaceKind,
        tests::prepare_infill::group_fills::focused::fixture::{
            external, graph, graph_snapshot, options_mut, outside_clipper_range, record_mut,
            rectangle, surface,
        },
    },
};

const LAYER: usize = 1;

#[test]
fn task22o76_line_pattern_group_emits_entities() {
    // `Line` infill is implemented (`FillBase.cpp::Fill::new_from_type` →
    // `generate.rs` `Line` dispatch arm): an internal surface produces
    // fill entities, not an empty fallback.
    let mut graph = graph();
    record_mut(&mut graph, LAYER).fill_surfaces = vec![surface(
        RegionSurfaceKind::Internal,
        rectangle(0, 0, 12_000_000, 8_000_000),
        0,
    )];
    options_mut(&mut graph, LAYER).sparse_infill_pattern = ProcessInfillPattern::Line;

    let generated = generate_layer(external(&graph), 0, LAYER, None, None).unwrap();
    assert!(!generated.collections.is_empty());
    combine_infill::dispose(graph);
}

#[test]
fn implemented_sparse_groups_emit_fill_paths() {
    for pattern in [
        ProcessInfillPattern::Rectilinear,
        ProcessInfillPattern::Triangles,
        ProcessInfillPattern::Cubic,
        ProcessInfillPattern::Gyroid,
    ] {
        let mut graph = graph();
        record_mut(&mut graph, LAYER).fill_surfaces = vec![surface(
            RegionSurfaceKind::Internal,
            rectangle(0, 0, 12_000_000, 8_000_000),
            0,
        )];
        options_mut(&mut graph, LAYER).sparse_infill_pattern = pattern;

        let generated = generate_layer(external(&graph), 0, LAYER, None, None).unwrap();

        assert!(!generated.collections.is_empty(), "{pattern:?}");
        assert!(
            generated
                .collections
                .iter()
                .all(|collection| !collection.entities.is_empty()),
            "{pattern:?}"
        );
        combine_infill::dispose(graph);
    }
}

#[test]
fn optimized_gyroid_emits_via_marching_squares_field() {
    // `gyroid_optimized` is ported (`FillGyroid.cpp:314-333` marching-squares
    // implicit-field path): an internal surface produces fill entities.
    let mut graph = graph();
    record_mut(&mut graph, LAYER).fill_surfaces = vec![surface(
        RegionSurfaceKind::Internal,
        rectangle(0, 0, 12_000_000, 8_000_000),
        0,
    )];
    let options = options_mut(&mut graph, LAYER);
    options.sparse_infill_pattern = ProcessInfillPattern::Gyroid;
    options.gyroid_optimized = crate::OrcaBool(true);

    let generated = generate_layer(external(&graph), 0, LAYER, None, None).unwrap();
    assert!(!generated.collections.is_empty());
    combine_infill::dispose(graph);
}

#[test]
fn task22o76_grouping_range_error_is_atomic() {
    let mut graph = graph();
    record_mut(&mut graph, LAYER).fill_surfaces = vec![
        surface(RegionSurfaceKind::Internal, outside_clipper_range(), 0),
        surface(RegionSurfaceKind::Internal, outside_clipper_range(), 0),
    ];
    options_mut(&mut graph, LAYER).sparse_infill_pattern = ProcessInfillPattern::CrossHatch;
    let before = graph_snapshot(&graph);

    assert_eq!(
        generate_layer(external(&graph), 0, LAYER, None, None),
        Err(SliceError::InvalidInput(
            "fill-grouping polygon coordinate is outside the supported Clipper range".to_owned()
        ))
    );
    assert_eq!(graph_snapshot(&graph).bytes, before.bytes);
    combine_infill::dispose(graph);
}
