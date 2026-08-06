use crate::project_slice::{
    prepare_infill::horizontal_shell_propagation::{self, GeometryStep, PropagationEvent},
    tests::support::KsrArchive,
};

#[test]
fn task22o26_aligned_none_neighbor_keeps_its_array_position_and_runs_safety_intersection() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\t\"ensure_vertical_shell_thickness\": \"ensure_all\",",
        "\t\"ensure_vertical_shell_thickness\": \"ensure_moderate\",",
    );
    let mut input = super::fixture::prepare_o25(archive.bytes());
    let index = 1;
    input.objects[0].records[index] = None;
    input.caches[0].records[index] = None;
    input.projections[0].records[index] = None;
    input.trims[0].records[index] = None;
    input.regularizations[0].records[index] = None;
    input.filters[0].records[index] = None;
    input.predecessor.objects[0].records[index] = None;
    let prelude = &mut input.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    prelude.records[index] = None;
    prelude.object.records[index] = None;

    horizontal_shell_propagation::reset_hooks();
    let output = horizontal_shell_propagation::prepare(input).unwrap();
    let events = horizontal_shell_propagation::events();
    let neighbor_visits = events
        .iter()
        .filter(|event| matches!(event, PropagationEvent::NeighborVisit { .. }))
        .count();
    assert!(events.iter().any(|event| matches!(
        event,
        PropagationEvent::NeighborVisit { neighbor, .. } if *neighbor == index
    )));
    assert!(!events.iter().any(|event| matches!(
        event,
        PropagationEvent::RecordVisit { layer, .. } if *layer == index
    )));
    assert_eq!(
        horizontal_shell_propagation::geometry_events()
            .iter()
            .filter(|step| **step == GeometryStep::SafetyIntersection)
            .count(),
        neighbor_visits
    );
    horizontal_shell_propagation::dispose(output);
}
