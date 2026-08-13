use crate::{
    ProcessInfillPattern, ProcessInternalBridgeFilter, SliceError,
    geometry::{ExPolygon, Point, Polygon},
    project_slice::{
        prepare_infill::{bridge_over_infill, external_surfaces},
        region_slices::RegionSurfaceKind,
        tests::support::{KsrArchive, metadata},
    },
    slice_project,
};

fn prepare(archive: KsrArchive) -> bridge_over_infill::PreparedPostBridgeCandidates {
    let horizontal = super::horizontal_shell_propagation::fixture::prepare(archive.bytes());
    let external = external_surfaces::prepare(horizontal).unwrap();
    bridge_over_infill::prepare(external).unwrap()
}

fn inventory_counts(
    prepared: &bridge_over_infill::PreparedPostBridgeCandidates,
) -> (usize, usize, usize) {
    (
        prepared.objects[0].surfaces_by_layer.len(),
        prepared.objects[0]
            .surfaces_by_layer
            .values()
            .map(Vec::len)
            .sum(),
        prepared.objects[0]
            .surfaces_by_layer
            .values()
            .flatten()
            .map(|candidate| candidate.new_polygons.len())
            .sum(),
    )
}

#[tokio::test]
async fn task22o43_task22o44_public_lifecycle_still_disposes_o43_once() {
    bridge_over_infill::reset_hooks();

    assert_eq!(
        slice_project(KsrArchive::new().bytes(), metadata())
            .await
            .unwrap_err(),
        SliceError::ProjectSlicingIncomplete
    );
    assert_eq!(bridge_over_infill::invocations(), 1);
    assert_eq!(bridge_over_infill::disposals(), 1);

    bridge_over_infill::reset_hooks();
}

#[test]
fn task22o43_real_ksr_retains_source_valid_crosshatch_candidates() {
    let prepared = prepare(KsrArchive::new());
    assert_eq!(prepared.objects.len(), 1);
    assert!(!prepared.objects[0].has_lightning_infill);
    assert_eq!(inventory_counts(&prepared), (18, 43, 53));
    assert_eq!(
        prepared.objects[0]
            .surfaces_by_layer
            .keys()
            .copied()
            .collect::<Vec<_>>(),
        vec![
            15, 30, 31, 32, 41, 45, 60, 65, 70, 75, 82, 85, 90, 105, 116, 125, 136, 255,
        ]
    );

    let horizontal = &prepared.predecessor.predecessor;
    let traversal = &horizontal.predecessor;
    let traversal_object = &traversal.objects[0];
    let prelude = &traversal_object
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let source_index = prelude.object.identity().0;
    let resolved = traversal
        .resolved
        .objects
        .iter()
        .find(|object| object.source_object_index == source_index)
        .unwrap();
    assert_eq!(
        resolved.object.dont_filter_internal_bridges,
        ProcessInternalBridgeFilter::Disabled
    );

    let (_, inputs) = prelude.object.as_parts();
    assert_eq!(inputs.len(), 460);
    assert!(inputs.iter().flatten().all(|input| {
        prelude.object.region_options(input).sparse_infill_pattern
            == ProcessInfillPattern::CrossHatch
    }));

    for (&layer_index, candidates) in &prepared.objects[0].surfaces_by_layer {
        let input = inputs[layer_index].as_ref().unwrap();
        let record = horizontal.objects[0].records[layer_index].as_ref().unwrap();
        for candidate in candidates {
            assert_eq!(candidate.source.layer_index, layer_index);
            assert_eq!(candidate.source.region_index, input.current.region_index);
            assert_eq!(
                record.fill_surfaces[candidate.source.surface_index]
                    .as_parts()
                    .0,
                RegionSurfaceKind::InternalSolid
            );
            assert!(!candidate.new_polygons.is_empty());
            assert_eq!(candidate.bridge_angle.to_bits(), 0.0_f64.to_bits());
        }
    }

    bridge_over_infill::dispose(prepared);
}

#[test]
fn task22o43_real_ksr_limited_filter_changes_candidate_and_polygon_counts() {
    let baseline = prepare(KsrArchive::new());
    let mut limited_archive = KsrArchive::new();
    limited_archive.replace_unique(
        "Metadata/project_settings.config",
        "\"dont_filter_internal_bridges\": \"disabled\"",
        "\"dont_filter_internal_bridges\": \"limited\"",
    );
    let limited = prepare(limited_archive);

    assert_eq!(inventory_counts(&baseline), (18, 43, 53));
    assert_eq!(inventory_counts(&limited), (58, 100, 166));

    let traversal = &limited.predecessor.predecessor.predecessor;
    let traversal_object = &traversal.objects[0];
    let prelude = &traversal_object
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let source_index = prelude.object.identity().0;
    assert_eq!(
        traversal
            .resolved
            .objects
            .iter()
            .find(|object| object.source_object_index == source_index)
            .unwrap()
            .object
            .dont_filter_internal_bridges,
        ProcessInternalBridgeFilter::Limited
    );

    bridge_over_infill::dispose(baseline);
    bridge_over_infill::dispose(limited);
}

#[test]
fn task22o43_range_error_maps_and_disposes_owned_o42_predecessor() {
    external_surfaces::reset_hooks();
    bridge_over_infill::reset_hooks();
    let horizontal =
        super::horizontal_shell_propagation::fixture::prepare(KsrArchive::new().bytes());
    let mut external = external_surfaces::prepare(horizontal).unwrap();
    assert_eq!(external_surfaces::invocations(), 1);
    assert_eq!(external_surfaces::disposals(), 0);

    external.predecessor.objects[0].records[0]
        .as_mut()
        .unwrap()
        .fill_expolygons = vec![outside_clipper_range()];

    let result = bridge_over_infill::prepare(external);
    let Err(SliceError::InvalidInput(message)) = result else {
        panic!("O43 range failure must map to invalid project input")
    };
    assert_eq!(
        message,
        "internal-bridge candidate coordinate is outside the supported Clipper range"
    );
    assert_eq!(bridge_over_infill::invocations(), 1);
    assert_eq!(bridge_over_infill::disposals(), 0);
    assert_eq!(external_surfaces::disposals(), 1);

    external_surfaces::reset_hooks();
    bridge_over_infill::reset_hooks();
}

fn outside_clipper_range() -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(0x4000_0000_0000_0000, 0),
            Point::new(0x4000_0000_0000_0000, 10),
            Point::new(0x3fff_ffff_ffff_ffff, 10),
        ]),
        Vec::new(),
    )
}

mod density_provenance;
mod multi_object;
mod object_scope;
