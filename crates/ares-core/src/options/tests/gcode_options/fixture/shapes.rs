use std::collections::{BTreeMap, BTreeSet};

use super::super::{gcode_rows, inventory};
use super::load::Fixture;

const PRINTER_VARIANT_KEYS: [&str; 19] = [
    "deretraction_speed",
    "nozzle_flush_dataset",
    "retract_before_wipe",
    "retraction_distances_when_cut",
    "long_retractions_when_cut",
    "retraction_length",
    "retract_length_toolchange",
    "z_hop",
    "z_hop_types",
    "travel_slope",
    "retract_lift_above",
    "retract_lift_below",
    "retract_lift_enforce",
    "retract_restart_extra",
    "retract_restart_extra_toolchange",
    "retraction_speed",
    "nozzle_type",
    "printer_extruder_id",
    "printer_extruder_variant",
];

const FILAMENT_VARIANT_KEYS: [&str; 10] = [
    "filament_adaptive_volumetric_speed",
    "filament_cooling_before_tower",
    "filament_extruder_variant",
    "filament_flow_ratio",
    "filament_flush_temp",
    "filament_flush_volumetric_speed",
    "filament_max_volumetric_speed",
    "long_retractions_when_ec",
    "retraction_distances_when_ec",
    "volumetric_speed_coefficients",
];

pub(super) fn assert_raw_shapes(fixture: &Fixture) {
    let inventory = inventory();
    let rows = gcode_rows(&inventory);
    let arrays = rows
        .into_iter()
        .filter(|row| row.wire_shape == "array")
        .collect::<Vec<_>>();
    assert_eq!(arrays.len(), 80);

    let histogram = arrays.iter().fold(BTreeMap::new(), |mut counts, row| {
        let length = fixture.raw[&row.key].as_array().unwrap().len();
        *counts.entry(length).or_insert(0) += 1;
        counts
    });
    assert_eq!(
        histogram,
        BTreeMap::from([(0, 1), (2, 49), (4, 19), (8, 10), (10, 1)])
    );

    let length_four = keys_with_length(&arrays, fixture, 4);
    let printer_variant = BTreeSet::from(PRINTER_VARIANT_KEYS);
    assert_eq!(length_four, printer_variant);

    let filament_keys = arrays
        .iter()
        .filter(|row| row.raw_scope == "filament")
        .map(|row| row.key.as_str())
        .collect::<BTreeSet<_>>();
    let filament_variant = BTreeSet::from(FILAMENT_VARIANT_KEYS);
    assert!(filament_variant.is_subset(&filament_keys));
    assert_eq!(keys_with_length(&arrays, fixture, 8), filament_variant);

    let ordinary_filament = filament_keys
        .difference(&filament_variant)
        .copied()
        .collect::<BTreeSet<_>>();
    assert_eq!(ordinary_filament.len(), 43);
    assert!(ordinary_filament.iter().all(|key| {
        fixture.raw[*key].as_array().unwrap().len() == 2
    }));
}

fn keys_with_length<'a>(
    rows: &[&'a super::super::InventoryRow],
    fixture: &Fixture,
    length: usize,
) -> BTreeSet<&'a str> {
    rows.iter()
        .filter(|row| fixture.raw[&row.key].as_array().unwrap().len() == length)
        .map(|row| row.key.as_str())
        .collect()
}
