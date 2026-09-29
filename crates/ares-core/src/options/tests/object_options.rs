use serde::Deserialize;

use super::super::{ObjectOptionOverrides, ObjectOptions, ProcessObjectSourceOptions};

mod base;
mod clamps;
mod fixture;
mod inventory;
mod normalization;
mod projection;
mod types;

#[derive(Debug, Deserialize)]
struct InventoryRow {
    key: String,
    raw_scope: String,
    static_owner: String,
    option_type: String,
    nullable: bool,
    default_serialized: String,
    wire_shape: String,
}

fn inventory() -> Vec<InventoryRow> {
    serde_json::from_str(crate::test_fixtures::options_inventory())
    .unwrap()
}

fn object_rows(rows: &[InventoryRow]) -> Vec<&InventoryRow> {
    rows.iter()
        .filter(|row| row.raw_scope == "process" && row.static_owner == "print_object_config")
        .collect()
}
