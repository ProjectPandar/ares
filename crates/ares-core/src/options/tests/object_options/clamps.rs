use crate::OrcaInt;

use super::{ObjectOptionOverrides, ObjectOptions, ProcessObjectSourceOptions};

#[derive(Clone, Copy)]
enum SupportField {
    Support,
    Interface,
}

fn resolve(
    base: &ProcessObjectSourceOptions,
    overrides: &ObjectOptionOverrides,
    num_extruders: usize,
) -> ObjectOptions {
    ObjectOptions::resolve(base, overrides, num_extruders)
}

fn base_pair(support: i32, interface: i32) -> ProcessObjectSourceOptions {
    ProcessObjectSourceOptions {
        support_filament: OrcaInt(support),
        support_interface_filament: OrcaInt(interface),
        ..Default::default()
    }
}

fn override_pair(support: Option<i32>, interface: Option<i32>) -> ObjectOptionOverrides {
    ObjectOptionOverrides {
        support_filament: support.map(OrcaInt),
        support_interface_filament: interface.map(OrcaInt),
        ..Default::default()
    }
}

fn field_case(
    field: SupportField,
    input: i32,
    expected: i32,
    other: i32,
) -> (ObjectOptionOverrides, (i32, i32)) {
    match field {
        SupportField::Support => (override_pair(Some(input), None), (expected, other)),
        SupportField::Interface => (override_pair(None, Some(input)), (other, expected)),
    }
}

fn assert_resolved(
    base: &ProcessObjectSourceOptions,
    overrides: &ObjectOptionOverrides,
    num_extruders: usize,
    expected_pair: (i32, i32),
) {
    let mut expected = ObjectOptions::overlay(base, overrides);
    expected.support_filament = OrcaInt(expected_pair.0);
    expected.support_interface_filament = OrcaInt(expected_pair.1);

    let actual = resolve(base, overrides, num_extruders);
    assert_eq!(
        (
            actual.support_filament,
            actual.support_interface_filament,
        ),
        (OrcaInt(expected_pair.0), OrcaInt(expected_pair.1))
    );
    assert_eq!(actual, expected);
}

#[test]
fn object_options_clamps_each_field_with_count_three() {
    const CASES: [(i32, i32); 5] = [(-1, -1), (0, 0), (1, 1), (3, 3), (4, 1)];
    let base = base_pair(2, 2);

    for field in [SupportField::Support, SupportField::Interface] {
        for (input, expected) in CASES {
            let (overrides, expected_pair) = field_case(field, input, expected, 2);
            assert_resolved(&base, &overrides, 3, expected_pair);
        }
    }
}

#[test]
fn object_options_clamps_each_field_with_zero_count() {
    const CASES: [(i32, i32); 4] = [(-1, -1), (0, 0), (1, 1), (2, 1)];
    let base = base_pair(0, 0);

    for field in [SupportField::Support, SupportField::Interface] {
        for (input, expected) in CASES {
            let (overrides, expected_pair) = field_case(field, input, expected, 0);
            assert_resolved(&base, &overrides, 0, expected_pair);
        }
    }
}

#[test]
fn object_options_clamps_both_fields_after_overlay() {
    let base = base_pair(2, 2);
    for (support, interface, expected) in [
        (Some(4), None, (1, 2)),
        (None, Some(4), (2, 1)),
    ] {
        let overrides = override_pair(support, interface);
        assert_resolved(&base, &overrides, 3, expected);
    }
}

#[test]
fn object_options_clamps_recomputes_the_same_overrides_for_the_current_count() {
    let base = base_pair(2, 2);
    let overrides = override_pair(Some(3), Some(3));

    assert_resolved(&base, &overrides, 2, (1, 1));
    assert_resolved(&base, &overrides, 3, (3, 3));
}
