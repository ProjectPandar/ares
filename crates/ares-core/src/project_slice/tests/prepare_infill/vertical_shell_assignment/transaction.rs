mod mismatches;

use crate::{
    SliceError,
    project_slice::{
        prepare_infill::vertical_shell_assignment::{
            self, GeometryStep, fail_geometry_at, fail_geometry_at_occurrence,
        },
        tests::support::KsrArchive,
    },
};

#[test]
fn task22o24_each_geometry_failure_exposes_no_successor_and_disposes_o23() {
    for step in [
        GeometryStep::SolidIntersection,
        GeometryStep::InternalDifference,
        GeometryStep::InternalVoidDifference,
    ] {
        let input = super::fixture::prepare_o23(KsrArchive::new().bytes());
        let (probe, dropped) = input.predecessor.drop_probe_observer();
        vertical_shell_assignment::reset_geometry_hooks();
        fail_geometry_at(step);
        assert_eq!(reject(input), range_error());
        assert!(probe.upgrade().is_none());
        assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
    }
    vertical_shell_assignment::reset_geometry_hooks();
}

#[test]
fn task22o24_later_record_failure_stages_before_any_commit() {
    let input = super::fixture::prepare_o23(KsrArchive::new().bytes());
    let (probe, dropped) = input.predecessor.drop_probe_observer();
    vertical_shell_assignment::reset_geometry_hooks();
    fail_geometry_at_occurrence(GeometryStep::SolidIntersection, 2);
    assert_eq!(reject(input), range_error());
    assert_eq!(vertical_shell_assignment::commits(), 0);
    assert_eq!(
        vertical_shell_assignment::geometry_events()
            .iter()
            .filter(|step| **step == GeometryStep::SolidIntersection)
            .count(),
        2
    );
    assert!(probe.upgrade().is_none());
    assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
    vertical_shell_assignment::reset_geometry_hooks();
}

#[test]
fn task22o24_later_object_failure_is_whole_project_transactional() {
    let mut first = prepared();
    let mut second = prepared();
    let first_calls = first
        .filters
        .iter()
        .flat_map(|object| &object.records)
        .flatten()
        .filter(|filter| !filter.filtered_shell.is_empty())
        .count();
    first.objects.push(second.objects.pop().unwrap());
    first.caches.push(second.caches.pop().unwrap());
    first.projections.push(second.projections.pop().unwrap());
    first.trims.push(second.trims.pop().unwrap());
    first
        .regularizations
        .push(second.regularizations.pop().unwrap());
    first.filters.push(second.filters.pop().unwrap());
    first
        .predecessor
        .objects
        .push(second.predecessor.objects.pop().unwrap());
    let (probe, dropped) = first.predecessor.drop_probe_observer();
    vertical_shell_assignment::reset_geometry_hooks();
    fail_geometry_at_occurrence(GeometryStep::SolidIntersection, first_calls + 1);
    assert_eq!(reject(first), range_error());
    assert_eq!(vertical_shell_assignment::commits(), 0);
    assert_eq!(
        vertical_shell_assignment::geometry_events()
            .iter()
            .filter(|step| **step == GeometryStep::SolidIntersection)
            .count(),
        first_calls + 1
    );
    assert!(probe.upgrade().is_none());
    assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
    vertical_shell_assignment::reset_geometry_hooks();
}

#[test]
fn task22o24_alignment_mismatch_panics_before_first_geometry_event() {
    let mut input = super::fixture::prepare_o23(KsrArchive::new().bytes());
    input.filters[0].records.pop();
    vertical_shell_assignment::reset_geometry_hooks();
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = vertical_shell_assignment::prepare(input);
        }))
        .is_err()
    );
    assert!(vertical_shell_assignment::geometry_events().is_empty());
}

pub(super) fn prepared(
) -> crate::project_slice::prepare_infill::vertical_shell_filtering::PreparedPostVerticalShellFiltering{
    super::fixture::prepare_o23(KsrArchive::new().bytes())
}

pub(super) fn rejects_alignment(
    input: crate::project_slice::prepare_infill::vertical_shell_filtering::PreparedPostVerticalShellFiltering,
) {
    vertical_shell_assignment::reset_geometry_hooks();
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = vertical_shell_assignment::prepare(input);
        }))
        .is_err()
    );
    assert!(vertical_shell_assignment::geometry_events().is_empty());
}

fn reject(
    input: crate::project_slice::prepare_infill::vertical_shell_filtering::PreparedPostVerticalShellFiltering,
) -> SliceError {
    match vertical_shell_assignment::prepare(input) {
        Err(error) => error,
        Ok(output) => {
            vertical_shell_assignment::dispose(output);
            panic!("failed O24 assignment must expose no successor")
        }
    }
}

fn range_error() -> SliceError {
    SliceError::InvalidInput(
        "vertical-shell fill-surface assignment geometry is outside the supported Clipper range"
            .to_owned(),
    )
}
