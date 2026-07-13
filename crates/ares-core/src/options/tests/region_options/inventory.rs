use std::collections::{BTreeMap, BTreeSet};

use super::{ProcessRegionSourceOptions, RegionOptions, inventory, region_rows};

const SELECTED_IRONING_KEYS: [&str; 4] = [
    "filament_ironing_flow",
    "filament_ironing_spacing",
    "filament_ironing_inset",
    "filament_ironing_speed",
];

#[test]
fn region_options_inventories_and_histograms_are_exact() {
    let rows = inventory();
    let region = region_rows(&rows);
    let source_histogram = region.iter().fold(BTreeMap::new(), |mut counts, row| {
        *counts.entry(row.option_type.as_str()).or_insert(0) += 1;
        counts
    });
    assert_eq!(region.len(), 149);
    assert_eq!(
        source_histogram,
        BTreeMap::from([
            ("coBool", 31),
            ("coEnum", 14),
            ("coFloat", 49),
            ("coFloatOrPercent", 24),
            ("coInt", 15),
            ("coInts", 1),
            ("coPercent", 11),
            ("coString", 3),
            ("coStrings", 1),
        ])
    );
    let mut effective_histogram = source_histogram;
    *effective_histogram.entry("coFloat").or_insert(0) += 3;
    *effective_histogram.entry("coPercent").or_insert(0) += 1;
    assert_eq!(
        effective_histogram,
        BTreeMap::from([
            ("coBool", 31),
            ("coEnum", 14),
            ("coFloat", 52),
            ("coFloatOrPercent", 24),
            ("coInt", 15),
            ("coInts", 1),
            ("coPercent", 12),
            ("coString", 3),
            ("coStrings", 1),
        ])
    );
}

#[test]
fn region_options_orders_have_exact_unique_keys() {
    let rows = inventory();
    let inventory_keys = region_rows(&rows)
        .into_iter()
        .map(|row| row.key.as_str())
        .collect::<BTreeSet<_>>();
    let source_keys = RegionOptions::PROCESS_DECLARATION_ORDER
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let effective_keys = RegionOptions::DECLARATION_ORDER
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();

    assert_eq!(RegionOptions::PROCESS_DECLARATION_ORDER.len(), 149);
    assert_eq!(RegionOptions::DECLARATION_ORDER.len(), 153);
    assert_eq!(source_keys.len(), 149);
    assert_eq!(effective_keys.len(), 153);
    assert_eq!(source_keys, inventory_keys);
    assert_eq!(
        RegionOptions::PROCESS_DECLARATION_ORDER,
        ProcessRegionSourceOptions::DECLARATION_ORDER
    );
    assert_eq!(
        &RegionOptions::DECLARATION_ORDER[..149],
        RegionOptions::PROCESS_DECLARATION_ORDER.as_slice()
    );
    assert_eq!(&RegionOptions::DECLARATION_ORDER[149..], &SELECTED_IRONING_KEYS);
}
