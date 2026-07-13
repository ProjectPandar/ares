use serde::Deserialize;

use super::super::{
    FilamentRegionSourceOptions, Nullable, ProcessRegionSourceOptions, RegionOptions,
    region_options::RegionOverrideSources,
};

mod base;
mod codecs;
mod filament;
mod fixture;
mod inventory;
mod modifier;
mod normalization;
mod overrides;
mod precedence;
mod types;

#[derive(Debug, Deserialize)]
struct InventoryRow {
    key: String,
    raw_scope: String,
    static_owner: String,
    option_type: String,
    default_serialized: String,
}

fn inventory() -> Vec<InventoryRow> {
    serde_json::from_str(include_str!(
        "../../../../../tests/ksr_fdmtest_v4/options-v242.json"
    ))
    .unwrap()
}

fn region_rows(rows: &[InventoryRow]) -> Vec<&InventoryRow> {
    rows.iter()
        .filter(|row| row.raw_scope == "process" && row.static_owner == "print_region_config")
        .collect()
}

fn resolve_region(sources: RegionOverrideSources<'_>, num_extruders: usize) -> RegionOptions {
    let filament = FilamentRegionSourceOptions {
        filament_ironing_flow: vec![Nullable::Nil; num_extruders],
        filament_ironing_spacing: vec![Nullable::Nil; num_extruders],
        filament_ironing_inset: vec![Nullable::Nil; num_extruders],
        filament_ironing_speed: vec![Nullable::Nil; num_extruders],
    };
    RegionOptions::resolve(&filament, sources, num_extruders)
}
