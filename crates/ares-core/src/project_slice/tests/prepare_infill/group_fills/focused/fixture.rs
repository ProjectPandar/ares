use crate::{
    ObjectOptions, OrcaBool, OrcaFloats, RegionOptions, Transform3d,
    geometry::{ExPolygon, Point, Polygon},
    project_slice::{
        prepare_infill::{
            combine_infill::{self, PreparedPostInfillCombination},
            external_surfaces::PreparedPostExternalSurfaces,
            surface_type_detection::types::PreparedSurfaceTypeRecord,
        },
        region_slices::{RegionSurface, RegionSurfaceKind},
        tests::{
            prepare_infill::bridge_over_infill::transaction::{SurfaceSnapshot, snapshot},
            support::KsrArchive,
        },
    },
};

pub(in crate::project_slice::tests::prepare_infill) fn graph() -> PreparedPostInfillCombination {
    let mut graph = combine_infill::prepare(super::super::super::combine_infill::prepare_o71(
        KsrArchive::new(),
    ))
    .unwrap();
    object_mut(&mut graph).detect_narrow_internal_solid_infill = OrcaBool(false);
    graph
}

pub(in crate::project_slice::tests::prepare_infill) fn external(
    graph: &PreparedPostInfillCombination,
) -> &PreparedPostExternalSurfaces {
    &graph.predecessor.predecessor
}

pub(in crate::project_slice::tests::prepare_infill) fn external_mut(
    graph: &mut PreparedPostInfillCombination,
) -> &mut PreparedPostExternalSurfaces {
    &mut graph.predecessor.predecessor
}

pub(in crate::project_slice::tests::prepare_infill) fn record_mut(
    graph: &mut PreparedPostInfillCombination,
    layer: usize,
) -> &mut PreparedSurfaceTypeRecord {
    external_mut(graph).predecessor.objects[0].records[layer]
        .as_mut()
        .unwrap()
}

pub(in super::super) fn clear_aligned_layer(
    graph: &mut PreparedPostInfillCombination,
    layer: usize,
) {
    let external = external_mut(graph);
    external.predecessor.objects[0].records[layer] = None;
    external.predecessor.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object
        .records[layer] = None;
}

pub(in super::super) fn options(
    graph: &PreparedPostInfillCombination,
    layer: usize,
) -> &RegionOptions {
    let traversal = &external(graph).predecessor.predecessor;
    let prelude = &traversal.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let input = prelude.object.records[layer].as_ref().unwrap();
    prelude.object.region_options(input)
}

pub(in crate::project_slice::tests::prepare_infill) fn options_mut(
    graph: &mut PreparedPostInfillCombination,
    layer: usize,
) -> &mut RegionOptions {
    let traversal = &mut external_mut(graph).predecessor.predecessor;
    let prelude = &mut traversal.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let region_index = prelude.object.records[layer]
        .as_ref()
        .unwrap()
        .current
        .region_index;
    let (post_regions, _) = prelude.object.object.as_parts_mut();
    &mut post_regions.regions[region_index].options
}

pub(in super::super) fn object_mut(
    graph: &mut PreparedPostInfillCombination,
) -> &mut ObjectOptions {
    &mut external_mut(graph).predecessor.predecessor.resolved.objects[0].object
}

pub(in super::super) fn planned_layer_mut(
    graph: &mut PreparedPostInfillCombination,
    layer: usize,
) -> &mut crate::project_slice::layers::PlannedLayer {
    let traversal = &mut external_mut(graph).predecessor.predecessor;
    let prelude = &mut traversal.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let (post_regions, _) = prelude.object.object.as_parts_mut();
    &mut post_regions.plan.layers[layer]
}

pub(in super::super) fn set_nozzles(
    graph: &mut PreparedPostInfillCombination,
    nozzles: OrcaFloats,
) {
    external_mut(graph)
        .predecessor
        .predecessor
        .resolved
        .views
        .full
        .project
        .print
        .nozzle_diameter = nozzles;
}

pub(in super::super) fn set_transform(
    graph: &mut PreparedPostInfillCombination,
    transform: Transform3d,
) {
    external_mut(graph).predecessor.predecessor.resolved.objects[0].print_objects[0].transform =
        transform;
}

pub(in crate::project_slice::tests::prepare_infill) fn surface(
    kind: RegionSurfaceKind,
    expolygon: ExPolygon,
    extra_perimeters: u16,
) -> RegionSurface {
    let mut surface =
        RegionSurface::internal_with_metadata(expolygon, -1.0, 1, -1.0, extra_perimeters);
    surface.retag(kind);
    surface
}

pub(in crate::project_slice::tests::prepare_infill) fn surface_with_height(
    expolygon: ExPolygon,
    height: f64,
) -> RegionSurface {
    RegionSurface::internal_with_metadata(expolygon, height, 1, -1.0, 0)
}

pub(in crate::project_slice::tests::prepare_infill) fn rectangle(
    min_x: i64,
    min_y: i64,
    max_x: i64,
    max_y: i64,
) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(min_x, min_y),
            Point::new(max_x, min_y),
            Point::new(max_x, max_y),
            Point::new(min_x, max_y),
        ]),
        Vec::new(),
    )
}

pub(in crate::project_slice::tests::prepare_infill) fn outside_clipper_range() -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(0x4000_0000_0000_0000, 0),
            Point::new(0x4000_0000_0000_0000, 10),
            Point::new(0x3fff_ffff_ffff_ffff, 10),
        ]),
        Vec::new(),
    )
}

pub(in crate::project_slice::tests::prepare_infill) fn graph_snapshot(
    graph: &PreparedPostInfillCombination,
) -> SurfaceSnapshot {
    snapshot(&graph.predecessor)
}

pub(in super::super) fn assert_snapshot_eq(actual: SurfaceSnapshot, expected: SurfaceSnapshot) {
    assert_eq!(actual.bytes, expected.bytes);
    assert_eq!(actual.bridge_layers, expected.bridge_layers);
    assert_eq!(actual.bridge_surfaces, expected.bridge_surfaces);
    assert_eq!(
        actual.bridge_expolygon_points,
        expected.bridge_expolygon_points
    );
}
