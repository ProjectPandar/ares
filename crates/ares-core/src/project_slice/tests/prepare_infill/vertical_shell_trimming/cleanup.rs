use crate::{
    SliceError,
    project_slice::{
        consume_post_vertical_shell_trim,
        prepare_infill::vertical_shell_trimming::{self, GeometryStep},
        tests::{
            deep_cleanup_support::{deepen_both_tree_families, run_on_constrained_stack},
            support::{KsrArchive, metadata},
        },
    },
};

use super::fixture;

#[test]
fn task22o21_direct_success_disposal_fits_both_deep_trees() {
    let mut prepared = fixture::prepare_o20(KsrArchive::new().bytes());
    deepen_both_tree_families(&mut prepared.predecessor);
    let (probe, dropped) = prepared.predecessor.drop_probe_observer();
    run_on_constrained_stack(move || {
        let output = vertical_shell_trimming::prepare(prepared).unwrap();
        vertical_shell_trimming::dispose(output);
    });
    assert!(probe.upgrade().is_none());
    assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
}

#[test]
fn task22o21_public_incomplete_disposal_fits_both_deep_trees() {
    let mut prepared = fixture::prepare_o20(KsrArchive::new().bytes());
    deepen_both_tree_families(&mut prepared.predecessor);
    let (probe, dropped) = prepared.predecessor.drop_probe_observer();
    run_on_constrained_stack(move || {
        let output = vertical_shell_trimming::prepare(prepared).unwrap();
        assert_eq!(
            consume_post_vertical_shell_trim(output, metadata()).unwrap_err(),
            SliceError::ProjectSlicingIncomplete
        );
    });
    assert!(probe.upgrade().is_none());
    assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
}

#[test]
fn task22o21_each_geometry_failure_disposes_both_deep_trees() {
    for step in [
        GeometryStep::SafetyOffset,
        GeometryStep::SafetyIntersection,
        GeometryStep::Difference,
    ] {
        let mut prepared = fixture::prepare_o20(KsrArchive::new().bytes());
        deepen_both_tree_families(&mut prepared.predecessor);
        let (probe, dropped) = prepared.predecessor.drop_probe_observer();
        run_on_constrained_stack(move || {
            vertical_shell_trimming::reset_geometry_hooks();
            vertical_shell_trimming::fail_geometry_at(step);
            assert!(matches!(
                vertical_shell_trimming::prepare(prepared),
                Err(SliceError::InvalidInput(message))
                    if message == "vertical-shell internal trimming geometry is outside the supported Clipper range"
            ));
            vertical_shell_trimming::reset_geometry_hooks();
        });
        assert!(probe.upgrade().is_none());
        assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
    }
}
