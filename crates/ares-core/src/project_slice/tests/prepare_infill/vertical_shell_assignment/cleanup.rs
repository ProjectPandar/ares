use crate::{
    SliceError,
    project_slice::{
        consume_post_vertical_shell_assignment,
        prepare_infill::vertical_shell_assignment::{self, GeometryStep},
        tests::{
            deep_cleanup_support::{deepen_both_tree_families, run_on_constrained_stack},
            support::{KsrArchive, metadata},
        },
    },
};

#[test]
fn task22o24_direct_success_and_public_incomplete_dispose_both_deep_tree_families() {
    for public in [false, true] {
        let mut input = super::fixture::prepare_o23(KsrArchive::new().bytes());
        deepen_both_tree_families(&mut input.predecessor);
        let (probe, dropped) = input.predecessor.drop_probe_observer();
        run_on_constrained_stack(move || {
            let output = vertical_shell_assignment::prepare(input).unwrap();
            if public {
                assert_eq!(
                    consume_post_vertical_shell_assignment(output, metadata()).unwrap_err(),
                    SliceError::ProjectSlicingIncomplete
                );
            } else {
                vertical_shell_assignment::dispose(output);
            }
        });
        assert!(probe.upgrade().is_none());
        assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
    }
}

#[test]
fn task22o24_each_failure_disposes_both_deep_tree_families() {
    for step in [
        GeometryStep::SolidIntersection,
        GeometryStep::InternalDifference,
        GeometryStep::InternalVoidDifference,
    ] {
        let mut input = super::fixture::prepare_o23(KsrArchive::new().bytes());
        deepen_both_tree_families(&mut input.predecessor);
        let (probe, dropped) = input.predecessor.drop_probe_observer();
        run_on_constrained_stack(move || {
            vertical_shell_assignment::reset_geometry_hooks();
            vertical_shell_assignment::fail_geometry_at(step);
            assert!(matches!(
                vertical_shell_assignment::prepare(input),
                Err(SliceError::InvalidInput(message))
                    if message == "vertical-shell fill-surface assignment geometry is outside the supported Clipper range"
            ));
            vertical_shell_assignment::reset_geometry_hooks();
        });
        assert!(probe.upgrade().is_none());
        assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
    }
}
