use crate::{
    SliceError,
    project_slice::{
        consume_post_vertical_shell_filtering,
        prepare_infill::vertical_shell_filtering::{self, GeometryStep},
        tests::{
            deep_cleanup_support::{deepen_both_tree_families, run_on_constrained_stack},
            support::{KsrArchive, metadata},
        },
    },
};

#[test]
fn task22o23_success_disposal_delegates_iterative_predecessor_cleanup() {
    let output = super::fixture::prepare(KsrArchive::new().bytes());
    let (probe, dropped) = output.predecessor.drop_probe_observer();
    vertical_shell_filtering::dispose(output);
    assert!(probe.upgrade().is_none());
    assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
}

#[test]
fn task22o23_direct_success_and_public_incomplete_dispose_both_10000_node_trees() {
    for public in [false, true] {
        let mut prepared = super::fixture::prepare_o22(KsrArchive::new().bytes());
        deepen_both_tree_families(&mut prepared.predecessor);
        let (probe, dropped) = prepared.predecessor.drop_probe_observer();
        run_on_constrained_stack(move || {
            let output = vertical_shell_filtering::prepare(prepared).unwrap();
            if public {
                assert_eq!(
                    consume_post_vertical_shell_filtering(output, metadata()).unwrap_err(),
                    SliceError::ProjectSlicingIncomplete
                );
            } else {
                vertical_shell_filtering::dispose(output);
            }
        });
        assert!(probe.upgrade().is_none());
        assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
    }
}

#[test]
fn task22o23_each_failure_site_disposes_both_10000_node_trees() {
    for step in [
        GeometryStep::NeighborIntersection,
        GeometryStep::ClosingGrow,
        GeometryStep::ClosingShrink,
        GeometryStep::VisibilityDifference,
        GeometryStep::CandidateExpansion,
        GeometryStep::ProtectionDifference,
    ] {
        let mut prepared = super::fixture::prepare_o22(KsrArchive::new().bytes());
        deepen_both_tree_families(&mut prepared.predecessor);
        let (probe, dropped) = prepared.predecessor.drop_probe_observer();
        run_on_constrained_stack(move || {
            vertical_shell_filtering::reset_geometry_hooks();
            vertical_shell_filtering::fail_geometry_at(step);
            assert!(matches!(
                vertical_shell_filtering::prepare(prepared),
                Err(SliceError::InvalidInput(message))
                    if message == "vertical-shell tiny-island filtering geometry is outside the supported Clipper range"
            ));
            vertical_shell_filtering::reset_geometry_hooks();
        });
        assert!(probe.upgrade().is_none());
        assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
    }
}
