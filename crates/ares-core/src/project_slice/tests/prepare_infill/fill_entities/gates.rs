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
fn task22o76_non_line_pattern_group_emits_no_fallback() {
    let mut graph = graph();
    record_mut(&mut graph, LAYER).fill_surfaces = vec![surface(
        RegionSurfaceKind::Internal,
        rectangle(0, 0, 12_000_000, 8_000_000),
        0,
    )];
    options_mut(&mut graph, LAYER).sparse_infill_pattern = ProcessInfillPattern::Gyroid;

    assert_eq!(
        generate_layer(external(&graph), 0, LAYER).unwrap(),
        crate::project_slice::fill_entities::LayerFillEntities::default()
    );
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
        generate_layer(external(&graph), 0, LAYER),
        Err(SliceError::InvalidInput(
            "fill-grouping polygon coordinate is outside the supported Clipper range".to_owned()
        ))
    );
    assert_eq!(graph_snapshot(&graph).bytes, before.bytes);
    combine_infill::dispose(graph);
}
