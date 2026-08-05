mod mismatches;

use crate::{
    SliceError,
    geometry::CoordinateScale,
    project_slice::{
        prepare_infill::{horizontal_shell_promotion, vertical_shell_assignment},
        tests::support::KsrArchive,
    },
};

macro_rules! prelude {
    ($prepared:expr) => {
        &mut ($prepared).predecessor.objects[0]
            .predecessor
            .predecessor
            .predecessor
            .predecessor
    };
}

#[test]
fn task22o25_later_object_parse_failure_rolls_back_all_earlier_matches() {
    let mut active_archive = KsrArchive::new();
    active_archive.replace_unique(
        "Metadata/project_settings.config",
        "\"extra_solid_infills\": \"\"",
        "\"extra_solid_infills\": \"1#\"",
    );
    let mut first = super::fixture::prepare_o24(active_archive.bytes());
    let mut second = super::fixture::prepare_o24(KsrArchive::new().bytes());
    prelude!(&mut second).object.object.as_parts_mut().0.regions[0]
        .options
        .extra_solid_infills
        .0 = "2147483648".to_owned();

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

    horizontal_shell_promotion::reset_hooks();
    assert!(matches!(
        horizontal_shell_promotion::prepare(first),
        Err(SliceError::InvalidInput(message))
            if message == "invalid extra_solid_infills pattern"
    ));
    assert_eq!(horizontal_shell_promotion::commits(), 0);
    assert_eq!(horizontal_shell_promotion::disposals(), 1);
    assert!(horizontal_shell_promotion::events().len() > 460);
    assert!(probe.upgrade().is_none());
    assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
}

#[test]
fn task22o25_alignment_preflight_precedes_schedule_visits_and_commits() {
    let mut scale = prepared();
    scale.predecessor.scale = match scale.predecessor.scale {
        CoordinateScale::Normal => CoordinateScale::LargeBed,
        CoordinateScale::LargeBed => CoordinateScale::Normal,
    };
    rejects_alignment(scale);

    let mut count = prepared();
    count.filters[0].records.pop();
    rejects_alignment(count);

    let mut identity = prepared();
    prelude!(&mut identity).object.records[0]
        .as_mut()
        .unwrap()
        .planned_layer_index += 1;
    rejects_alignment(identity);
}

pub(super) fn prepared() -> vertical_shell_assignment::PreparedPostVerticalShellAssignment {
    super::fixture::prepare_o24(KsrArchive::new().bytes())
}

pub(super) fn rejects_alignment(
    prepared: vertical_shell_assignment::PreparedPostVerticalShellAssignment,
) {
    horizontal_shell_promotion::reset_hooks();
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = horizontal_shell_promotion::prepare(prepared);
        }))
        .is_err()
    );
    assert!(horizontal_shell_promotion::events().is_empty());
    assert_eq!(horizontal_shell_promotion::commits(), 0);
}
