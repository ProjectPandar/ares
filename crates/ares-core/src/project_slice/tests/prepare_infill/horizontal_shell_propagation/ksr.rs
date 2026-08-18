pub(in crate::project_slice::tests::prepare_infill::horizontal_shell_propagation) mod digest;

use crate::project_slice::{
    prepare_infill::{
        horizontal_shell_promotion::{self, PreparedPostHorizontalShellPromotion},
        horizontal_shell_propagation::{
            self, PreparedPostHorizontalShellPropagation, PropagationEvent,
        },
        surface_type_detection::PreparedSurfaceTypeObject,
    },
    tests::support::KsrArchive,
};
use digest::{event_sequence_digest, propagation_event_counts, surface_sequence_digest};

#[test]
fn task22o26_ksr_visits_and_ensure_all_skips_every_aligned_record_without_commits() {
    horizontal_shell_propagation::reset_hooks();
    let input = super::fixture::prepare_o25(KsrArchive::new().bytes());
    let before_pointers = fill_pointers(&input.objects);
    let before_envelope = o25_envelope(&input);
    let before_nonfill = nonfill_pointers(&input.objects);
    let before_digest = surface_sequence_digest(&input.objects);
    let output = horizontal_shell_propagation::prepare(input).unwrap();
    let events = horizontal_shell_propagation::events();
    let after_digest = surface_sequence_digest(&output.objects);
    let first_event_digest = event_sequence_digest(&events);

    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, PropagationEvent::RecordVisit { .. }))
            .count(),
        460
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, PropagationEvent::EnsureAllSkip { .. }))
            .count(),
        460
    );
    assert_eq!(events.len(), 920);
    assert_eq!(after_digest, before_digest);
    assert_eq!(horizontal_shell_propagation::geometry_events(), Vec::new());
    assert_eq!(horizontal_shell_propagation::commits(), 0);
    assert_eq!(horizontal_shell_propagation::invocations(), 1);
    assert_eq!(horizontal_shell_propagation::disposals(), 0);
    assert_eq!(fill_pointers(&output.objects), before_pointers);
    assert_eq!(o26_envelope(&output), before_envelope);
    assert_eq!(nonfill_pointers(&output.objects), before_nonfill);

    horizontal_shell_propagation::dispose(output);
    assert_eq!(horizontal_shell_propagation::disposals(), 1);

    horizontal_shell_propagation::reset_hooks();
    let repeated = horizontal_shell_propagation::prepare(super::fixture::prepare_o25(
        KsrArchive::new().bytes(),
    ))
    .unwrap();
    assert_eq!(surface_sequence_digest(&repeated.objects), after_digest);
    assert_eq!(
        event_sequence_digest(&horizontal_shell_propagation::events()),
        first_event_digest
    );
    assert_eq!(horizontal_shell_propagation::commits(), 0);
    horizontal_shell_propagation::dispose(repeated);
}

#[test]
fn task22o26_ensure_all_gate_observes_active_o25_promotion_before_skipping() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\t\"extra_solid_infills\": \"\",",
        "\t\"extra_solid_infills\": \"1#\",",
    );
    horizontal_shell_promotion::reset_hooks();
    horizontal_shell_propagation::reset_hooks();
    let output = super::fixture::prepare(archive.bytes());

    assert_eq!(horizontal_shell_promotion::commits(), 460);
    assert_eq!(
        horizontal_shell_propagation::events()
            .iter()
            .filter(|event| matches!(event, PropagationEvent::EnsureAllSkip { .. }))
            .count(),
        460
    );
    assert_eq!(horizontal_shell_propagation::commits(), 0);
    horizontal_shell_propagation::dispose(output);
}

fn o25_envelope(prepared: &PreparedPostHorizontalShellPromotion) -> [usize; 8] {
    [
        (&*prepared.predecessor) as *const _ as usize,
        prepared.objects.as_ptr() as usize,
        prepared.caches.as_ptr() as usize,
        prepared.projections.as_ptr() as usize,
        prepared.trims.as_ptr() as usize,
        prepared.regularizations.as_ptr() as usize,
        prepared.filters.as_ptr() as usize,
        prepared.predecessor.objects.as_ptr() as usize,
    ]
}

fn o26_envelope(prepared: &PreparedPostHorizontalShellPropagation) -> [usize; 8] {
    [
        (&*prepared.predecessor) as *const _ as usize,
        prepared.objects.as_ptr() as usize,
        prepared.caches.as_ptr() as usize,
        prepared.projections.as_ptr() as usize,
        prepared.trims.as_ptr() as usize,
        prepared.regularizations.as_ptr() as usize,
        prepared.filters.as_ptr() as usize,
        prepared.predecessor.objects.as_ptr() as usize,
    ]
}

fn nonfill_pointers(objects: &[PreparedSurfaceTypeObject]) -> Vec<[usize; 5]> {
    objects
        .iter()
        .flat_map(|object| &object.records)
        .filter_map(Option::as_ref)
        .map(|record| {
            [
                record.perimeters.as_ptr() as usize,
                record.thin_fills.as_ptr() as usize,
                record.slices.as_ptr() as usize,
                record.fill_expolygons.as_ptr() as usize,
                record.fill_no_overlap_expolygons.as_ptr() as usize,
            ]
        })
        .collect()
}

fn record_fill_vector_pointers(
    objects: &[PreparedSurfaceTypeObject],
) -> Vec<*const crate::project_slice::region_slices::RegionSurface> {
    objects
        .iter()
        .flat_map(|object| &object.records)
        .filter_map(Option::as_ref)
        .map(|record| record.fill_surfaces.as_ptr())
        .collect()
}

fn fill_pointers(
    objects: &[PreparedSurfaceTypeObject],
) -> Vec<(
    *const crate::project_slice::region_slices::RegionSurface,
    usize,
    Vec<Vec<*const crate::geometry::Point>>,
)> {
    objects
        .iter()
        .flat_map(|object| &object.records)
        .filter_map(Option::as_ref)
        .map(|record| {
            (
                record.fill_surfaces.as_ptr(),
                record.fill_surfaces.capacity(),
                record
                    .fill_surfaces
                    .iter()
                    .map(|surface| {
                        let expolygon = surface.as_parts().1;
                        std::iter::once(expolygon.contour())
                            .chain(expolygon.holes())
                            .map(|path| path.points().as_ptr())
                            .collect()
                    })
                    .collect(),
            )
        })
        .collect()
}

#[test]
fn task22o26_typed_moderate_archive_executes_horizontal_propagation() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\t\"ensure_vertical_shell_thickness\": \"ensure_all\",",
        "\t\"ensure_vertical_shell_thickness\": \"ensure_moderate\",",
    );
    horizontal_shell_propagation::reset_hooks();
    let input = super::fixture::prepare_o25(archive.bytes());
    let before_record_pointers = record_fill_vector_pointers(&input.objects);
    let before_envelope = o25_envelope(&input);
    let before_nonfill = nonfill_pointers(&input.objects);
    let output = horizontal_shell_propagation::prepare(input).unwrap();
    let events = horizontal_shell_propagation::events();

    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, PropagationEvent::RecordVisit { .. }))
            .count(),
        460
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, PropagationEvent::EnsureAllSkip { .. }))
            .count(),
        0
    );
    assert_eq!(
        (
            propagation_event_counts(&events),
            horizontal_shell_propagation::geometry_events().len(),
            horizontal_shell_propagation::commits(),
        ),
        ([460, 460, 0, 1_380, 1_010, 547, 143], 5_469, 143)
    );
    assert!(
        horizontal_shell_propagation::gather_observations()
            .iter()
            .any(|observation| observation.dirty_before_gather && observation.path_count > 0)
    );
    let after_record_pointers = record_fill_vector_pointers(&output.objects);
    assert_eq!(o26_envelope(&output), before_envelope);
    assert_eq!(nonfill_pointers(&output.objects), before_nonfill);
    let mut dirty = vec![false; after_record_pointers.len()];
    for event in &events {
        if let PropagationEvent::DirtyCommit { object, layer } = *event {
            assert_eq!(object, 0);
            dirty[layer] = true;
        }
    }
    for (index, (&before, &after)) in before_record_pointers
        .iter()
        .zip(&after_record_pointers)
        .enumerate()
    {
        assert_eq!(before == after, !dirty[index]);
    }
    horizontal_shell_propagation::dispose(output);
}
