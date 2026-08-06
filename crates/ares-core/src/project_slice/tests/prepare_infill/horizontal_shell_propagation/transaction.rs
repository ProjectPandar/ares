mod failures;
mod mismatches;

use crate::{
    geometry::CoordinateScale,
    project_slice::{
        prepare_infill::{horizontal_shell_promotion, horizontal_shell_propagation},
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

pub(super) fn prepared() -> horizontal_shell_promotion::PreparedPostHorizontalShellPromotion {
    super::fixture::prepare_o25(KsrArchive::new().bytes())
}

pub(super) fn rejects_alignment(
    prepared: horizontal_shell_promotion::PreparedPostHorizontalShellPromotion,
) {
    horizontal_shell_propagation::reset_hooks();
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = horizontal_shell_propagation::prepare(prepared);
        }))
        .is_err()
    );
    assert!(horizontal_shell_propagation::events().is_empty());
    assert!(horizontal_shell_propagation::geometry_events().is_empty());
    assert_eq!(horizontal_shell_propagation::commits(), 0);
}

#[test]
fn task22o26_retained_scale_mismatch_precedes_clones_and_geometry() {
    let mut input = prepared();
    input.predecessor.scale = match input.predecessor.scale {
        CoordinateScale::Normal => CoordinateScale::LargeBed,
        CoordinateScale::LargeBed => CoordinateScale::Normal,
    };
    rejects_alignment(input);
}

#[test]
fn task22o26_outer_and_record_count_mismatches_precede_clones() {
    let mut outer = prepared();
    outer.filters.pop();
    rejects_alignment(outer);

    let mut records = prepared();
    records.regularizations[0].records.pop();
    rejects_alignment(records);
}

#[test]
fn task22o26_slot_presence_mismatch_precedes_clones() {
    let mut input = prepared();
    input.objects[0].records[1] = None;
    rejects_alignment(input);
}

#[test]
fn task22o26_input_identity_mismatch_precedes_clones() {
    let mut input = prepared();
    prelude!(&mut input).object.records[0]
        .as_mut()
        .unwrap()
        .planned_layer_index += 1;
    rejects_alignment(input);
}
