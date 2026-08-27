use crate::{
    SliceError,
    project_slice::{
        consume_post_vertical_shell_cache, perimeters,
        prepare_infill::{fill_surfaces, surface_type_detection, vertical_shells},
        tests::{
            deep_cleanup_support::{deepen_both_tree_families, run_on_constrained_stack},
            support::{KsrArchive, metadata},
        },
    },
};

fn prepared() -> fill_surfaces::PreparedPostFillSurfacePreparation {
    let detected = surface_type_detection::prepare(
        perimeters::prepare_post_layer_region_perimeters(KsrArchive::new().bytes()).unwrap(),
    )
    .unwrap();
    fill_surfaces::prepare(detected)
}

#[test]
fn task22o19_public_incomplete_cleanup_fits_constrained_stack_with_both_deep_trees() {
    let mut prepared = prepared();
    deepen_both_tree_families(&mut prepared.predecessor);
    let (probe, dropped) = prepared.predecessor.drop_probe_observer();
    run_on_constrained_stack(move || {
        let output = vertical_shells::prepare(prepared).unwrap();
        assert_eq!(
            consume_post_vertical_shell_cache(output, metadata()).unwrap_err(),
            SliceError::ProjectSlicingIncomplete
        );
    });
    assert!(probe.upgrade().is_none());
    assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
}

#[test]
fn task22o19_direct_success_disposal_fits_constrained_stack_with_both_deep_trees() {
    let mut prepared = prepared();
    deepen_both_tree_families(&mut prepared.predecessor);
    let (probe, dropped) = prepared.predecessor.drop_probe_observer();
    run_on_constrained_stack(move || {
        let output = vertical_shells::prepare(prepared).unwrap();
        vertical_shells::dispose(output);
    });
    assert!(probe.upgrade().is_none());
    assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
}

#[test]
fn task22o19_top_failure_cleanup_fits_constrained_stack_with_both_deep_trees() {
    failure_cleanup(
        vertical_shells::GeometryStep::Top,
        vec![vertical_shells::GeometryStep::Top],
    );
}

#[test]
fn task22o19_bottom_failure_cleanup_fits_constrained_stack_with_both_deep_trees() {
    failure_cleanup(
        vertical_shells::GeometryStep::Bottom,
        vec![
            vertical_shells::GeometryStep::Top,
            vertical_shells::GeometryStep::Bottom,
        ],
    );
}

fn failure_cleanup(
    step: vertical_shells::GeometryStep,
    expected: Vec<vertical_shells::GeometryStep>,
) {
    let mut prepared = prepared();
    deepen_both_tree_families(&mut prepared.predecessor);
    let (probe, dropped) = prepared.predecessor.drop_probe_observer();
    run_on_constrained_stack(move || {
        vertical_shells::reset_geometry_hooks();
        vertical_shells::fail_geometry_at(step);
        let error = match vertical_shells::prepare(prepared) {
            Err(error) => error,
            Ok(_) => panic!("failed O19 stage must not expose a successor"),
        };
        assert_eq!(
            error,
            SliceError::InvalidInput(
                "vertical-shell cache geometry is outside the supported Clipper range".to_owned()
            )
        );
        assert_eq!(vertical_shells::geometry_events(), expected);
        vertical_shells::reset_geometry_hooks();
    });
    assert!(probe.upgrade().is_none());
    assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
}
