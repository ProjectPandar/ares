use std::collections::BTreeSet;

use crate::OrcaInt;

use super::{ObjectOptionOverrides, ObjectOptions, ProcessObjectSourceOptions, types};
use super::super::process_object_source::expected::DECLARATION_ORDER as FIXED_PRINT_OBJECT_CONFIG_KEYS;

const NORMALIZE_FDM_WRITE_KEYS: [&str; 16] = [
    "extruder",
    "sparse_infill_filament_id",
    "outer_wall_filament_id",
    "inner_wall_filament_id",
    "internal_solid_filament_id",
    "top_surface_filament_id",
    "bottom_surface_filament_id",
    "retract_when_changing_layer",
    "filament_retract_when_changing_layer",
    "wall_loops",
    "alternate_extra_wall",
    "top_shell_layers",
    "sparse_infill_density",
    "resolution",
    "enable_prime_tower",
    "independent_support_layer_height",
];

const NORMALIZE_FDM_1_WRITE_KEYS: [&str; 14] = [
    "extruder",
    "sparse_infill_filament_id",
    "outer_wall_filament_id",
    "inner_wall_filament_id",
    "internal_solid_filament_id",
    "top_surface_filament_id",
    "bottom_surface_filament_id",
    "retract_when_changing_layer",
    "filament_retract_when_changing_layer",
    "wall_loops",
    "alternate_extra_wall",
    "top_shell_layers",
    "sparse_infill_density",
    "resolution",
];

const NORMALIZE_FDM_2_WRITE_KEYS: [&str; 2] = [
    "enable_prime_tower",
    "independent_support_layer_height",
];

#[test]
fn object_options_normalization_fixed_write_sets_are_exact_and_disjoint() {
    let monolithic = unique_set(&NORMALIZE_FDM_WRITE_KEYS);
    let split_1 = unique_set(&NORMALIZE_FDM_1_WRITE_KEYS);
    let split_2 = unique_set(&NORMALIZE_FDM_2_WRITE_KEYS);
    let object = unique_set(&FIXED_PRINT_OBJECT_CONFIG_KEYS);

    assert_eq!(monolithic.len(), 16);
    assert_eq!(split_1.len(), 14);
    assert_eq!(split_2.len(), 2);
    assert_eq!(object.len(), 126);
    assert!(split_1.is_disjoint(&split_2));

    let split_order = NORMALIZE_FDM_1_WRITE_KEYS
        .into_iter()
        .chain(NORMALIZE_FDM_2_WRITE_KEYS)
        .collect::<Vec<_>>();
    assert_eq!(split_order, NORMALIZE_FDM_WRITE_KEYS);

    let split_union = split_1.union(&split_2).copied().collect::<BTreeSet<_>>();
    assert_eq!(split_union, monolithic);
    assert!(monolithic.is_disjoint(&object));
    assert!(split_1.is_disjoint(&object));
    assert!(split_2.is_disjoint(&object));
}

#[test]
fn object_options_normalization_empty_overrides_cannot_change_effective_options() {
    let base = ProcessObjectSourceOptions {
        support_filament: OrcaInt(2),
        support_interface_filament: OrcaInt(1),
        ..Default::default()
    };
    let overrides = ObjectOptionOverrides::default();
    let expected = ObjectOptions::from_base(&base);
    let actual = ObjectOptions::resolve(&base, &overrides, 2);

    types::assert_base_and_sparse(&actual, &base, &overrides);
    assert_eq!(actual, expected);
}

fn unique_set<'a, const N: usize>(values: &'a [&'a str; N]) -> BTreeSet<&'a str> {
    let set = values.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(set.len(), N);
    set
}
