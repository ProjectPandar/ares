use crate::options::{
    Nullable, OrcaBool,
    project_variants::materialize_project_variants,
    registry::filament_options_with_variant,
};

use super::support::{
    assert_changed_keys, assert_invalid_key, assert_outside_variant_families_unchanged,
    assert_selected_indices, filament_sentinel_source, ints,
};

#[test]
fn filament_selects_all_fields_at_raw_index_six_and_preserves_source() {
    let source = filament_sentinel_source();
    let original = source.clone();

    let materialized = materialize_project_variants(&source, &ints(&[1, 2])).unwrap();

    assert_eq!(source, original);
    assert_eq!(filament_options_with_variant().len(), 37);
    assert_selected_indices(
        &source,
        &materialized,
        filament_options_with_variant(),
        &[0, 6],
    );
    assert_eq!(
        materialized
            .filament
            .retract_overrides
            .filament_retract_when_changing_layer,
        vec![Nullable::Value(OrcaBool(true)), Nullable::Nil]
    );
    assert_outside_variant_families_unchanged(&source, &materialized);
}

#[test]
fn rematerializing_same_raw_source_is_deterministic_and_map_changes_exact_family() {
    let source = filament_sentinel_source();

    let first = materialize_project_variants(&source, &ints(&[1, 2])).unwrap();
    let repeated = materialize_project_variants(&source, &ints(&[1, 2])).unwrap();
    let remapped = materialize_project_variants(&source, &ints(&[2, 1])).unwrap();

    assert_eq!(first, repeated);
    assert_changed_keys(
        &first,
        &remapped,
        filament_options_with_variant()
            .iter()
            .copied()
            .chain(["filament_map"]),
    );
}

#[test]
fn selected_filament_payload_out_of_range_names_key() {
    let mut source = filament_sentinel_source();
    source
        .filament
        .gcode
        .filament_max_volumetric_speed
        .0
        .truncate(6);

    assert_invalid_key(
        materialize_project_variants(&source, &ints(&[1, 2])),
        "filament_max_volumetric_speed",
    );
}
