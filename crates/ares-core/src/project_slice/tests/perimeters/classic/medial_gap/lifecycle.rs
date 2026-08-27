use crate::project_slice::tests::deep_cleanup_support::{
    deepen_both_tree_families, run_on_constrained_stack,
};

use crate::{
    SliceError,
    geometry::{ExPolygon, Polygon},
    project_slice::{
        incomplete_sink,
        perimeters::{classic::medial_gap, prepare_post_classic_gap_domain},
    },
};

use super::super::super::super::support::ksr_project;
#[test]
fn task22o13_repeated_source_point_maps_exact_transactional_error() {
    let mut prepared = prepare_post_classic_gap_domain(ksr_project()).unwrap();
    let (probe, dropped) = prepared.predecessor.drop_probe_observer();
    assert!(probe.upgrade().is_some());
    assert!(!dropped.load(std::sync::atomic::Ordering::SeqCst));
    inject_repeated_adjacent_point(&mut prepared);
    let result = medial_gap::finish(prepared);
    assert!(matches!(
        result,
        Err(SliceError::InvalidInput(message))
            if message == "Classic medial-axis Voronoi construction failed"
    ));
    assert_eq!(medial_gap::error_cleanup_probe_alive(), Some(true));
    assert!(probe.upgrade().is_none());
    assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
}

#[test]
fn task22o13_success_and_error_cleanup_fit_constrained_stack() {
    let success = prepare_post_classic_gap_domain(ksr_project()).unwrap();
    let mut success = medial_gap::finish(success).unwrap();
    deepen_both_tree_families(&mut success.predecessor);
    let mut failure = prepare_post_classic_gap_domain(ksr_project()).unwrap();
    deepen_both_tree_families(&mut failure.predecessor);
    inject_repeated_adjacent_point(&mut failure);

    run_on_constrained_stack(move || {
        for object in success.objects {
            incomplete_sink::consume_medial_gap_object(object);
        }
        incomplete_sink::consume_boxed_post_classic_traversal(success.predecessor);
    });
    run_on_constrained_stack(move || {
        assert!(medial_gap::finish(failure).is_err());
    });
}

fn inject_repeated_adjacent_point(
    prepared: &mut crate::project_slice::perimeters::classic::gap_domain::PreparedPostClassicGapDomain,
) {
    let domain = prepared
        .objects
        .iter_mut()
        .flat_map(|object| object.records.iter_mut().flatten())
        .flat_map(|record| &mut record.surfaces)
        .find_map(|surface| surface.pre_medial.as_mut())
        .unwrap();
    let expolygon = domain.expolygons.first_mut().unwrap();
    let mut points = expolygon.contour().points().to_vec();
    points.insert(1, points[0]);
    let holes = expolygon.holes().to_vec();
    *expolygon = ExPolygon::new(Polygon::new(points), holes);
}
