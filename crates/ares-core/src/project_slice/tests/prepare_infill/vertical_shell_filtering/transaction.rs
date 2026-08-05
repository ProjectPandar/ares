mod failures;

use crate::{
    SliceError,
    project_slice::{
        prepare_infill::vertical_shell_filtering::{self, GeometryStep},
        tests::support::KsrArchive,
    },
};

#[test]
fn task22o23_later_slot_failure_rolls_back_the_whole_predecessor() {
    let input = super::fixture::prepare_o22(KsrArchive::new().bytes());
    let (probe, dropped) = input.predecessor.drop_probe_observer();
    vertical_shell_filtering::reset_geometry_hooks();
    vertical_shell_filtering::fail_geometry_at_occurrence(GeometryStep::NeighborIntersection, 2);
    let error = match vertical_shell_filtering::prepare(input) {
        Err(error) => error,
        Ok(_) => panic!("later filtering failure must expose no successor"),
    };
    assert_eq!(error, range_error());
    assert_eq!(
        vertical_shell_filtering::geometry_events()
            .iter()
            .filter(|event| **event == GeometryStep::NeighborIntersection)
            .count(),
        2
    );
    assert!(probe.upgrade().is_none());
    assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
    vertical_shell_filtering::reset_geometry_hooks();
}

#[test]
fn task22o23_later_object_failure_is_whole_project_transactional() {
    let mut first = super::fixture::prepare_o22(KsrArchive::new().bytes());
    let mut second = super::fixture::prepare_o22(KsrArchive::new().bytes());
    let first_calls = first
        .trims
        .iter()
        .flat_map(|object| &object.records)
        .flatten()
        .filter(|trim| !trim.shell.is_empty())
        .count();
    first.objects.push(second.objects.pop().unwrap());
    first.caches.push(second.caches.pop().unwrap());
    first.projections.push(second.projections.pop().unwrap());
    first.trims.push(second.trims.pop().unwrap());
    first
        .regularizations
        .push(second.regularizations.pop().unwrap());
    first
        .predecessor
        .objects
        .push(second.predecessor.objects.pop().unwrap());
    let (probe, dropped) = first.predecessor.drop_probe_observer();
    vertical_shell_filtering::reset_geometry_hooks();
    vertical_shell_filtering::fail_geometry_at_occurrence(
        GeometryStep::NeighborIntersection,
        first_calls + 1,
    );
    let error = match vertical_shell_filtering::prepare(first) {
        Err(error) => error,
        Ok(_) => panic!("later-object failure must expose no successor"),
    };
    assert_eq!(error, range_error());
    assert!(probe.upgrade().is_none());
    assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
    vertical_shell_filtering::reset_geometry_hooks();
}

pub(super) fn range_error() -> SliceError {
    SliceError::InvalidInput(
        "vertical-shell tiny-island filtering geometry is outside the supported Clipper range"
            .to_owned(),
    )
}
