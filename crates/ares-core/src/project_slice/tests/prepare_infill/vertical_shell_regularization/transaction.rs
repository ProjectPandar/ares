use crate::{
    SliceError,
    project_slice::{
        prepare_infill::vertical_shell_regularization::{
            self, GeometryStep, fail_geometry_at_occurrence,
        },
        tests::support::KsrArchive,
    },
};

#[test]
fn task22o22_later_slot_failure_is_whole_project_transactional() {
    let prepared = super::fixture::prepare_o21(KsrArchive::new().bytes());
    assert!(
        prepared.trims[0]
            .records
            .iter()
            .flatten()
            .filter(|trim| !trim.shell.is_empty())
            .count()
            > 1
    );
    let (probe, dropped) = prepared.predecessor.drop_probe_observer();
    vertical_shell_regularization::reset_geometry_hooks();
    fail_geometry_at_occurrence(GeometryStep::Union, 2);
    let error = match vertical_shell_regularization::prepare(prepared) {
        Err(error) => error,
        Ok(_) => panic!("later-slot failure must expose no O22 successor"),
    };
    assert_eq!(
        error,
        SliceError::InvalidInput(
            "vertical-shell regularization geometry is outside the supported Clipper range"
                .to_owned()
        )
    );
    assert_eq!(
        vertical_shell_regularization::geometry_events()
            .iter()
            .filter(|event| **event == GeometryStep::Union)
            .count(),
        2
    );
    assert!(probe.upgrade().is_none());
    assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
    vertical_shell_regularization::reset_geometry_hooks();
}

#[test]
fn task22o22_later_object_failure_is_whole_project_transactional() {
    let mut first = super::fixture::prepare_o21(KsrArchive::new().bytes());
    let mut second = super::fixture::prepare_o21(KsrArchive::new().bytes());
    let first_calls = first
        .trims
        .iter()
        .flat_map(|object| &object.records)
        .flatten()
        .filter(|trim| !trim.shell.is_empty())
        .count();
    assert!(first_calls > 0);
    first.objects.push(second.objects.pop().unwrap());
    first.caches.push(second.caches.pop().unwrap());
    first.projections.push(second.projections.pop().unwrap());
    first.trims.push(second.trims.pop().unwrap());
    first
        .predecessor
        .objects
        .push(second.predecessor.objects.pop().unwrap());
    let (probe, dropped) = first.predecessor.drop_probe_observer();

    vertical_shell_regularization::reset_geometry_hooks();
    fail_geometry_at_occurrence(GeometryStep::Union, first_calls + 1);
    let error = match vertical_shell_regularization::prepare(first) {
        Err(error) => error,
        Ok(_) => panic!("later-object failure must expose no O22 successor"),
    };
    assert_eq!(
        error,
        SliceError::InvalidInput(
            "vertical-shell regularization geometry is outside the supported Clipper range"
                .to_owned()
        )
    );
    let events = vertical_shell_regularization::geometry_events();
    assert_eq!(
        events
            .iter()
            .filter(|event| **event == GeometryStep::Union)
            .count(),
        first_calls + 1
    );
    assert!(probe.upgrade().is_none());
    assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
    vertical_shell_regularization::reset_geometry_hooks();
}
