use serde::Deserialize;

mod inventory;
mod fixture;
mod projection;
mod templates;
mod types;

#[derive(Debug, Deserialize)]
struct InventoryRow {
    key: String,
    raw_scope: String,
    static_owner: String,
    option_type: String,
    nullable: bool,
    wire_shape: String,
    effective_projections: Vec<String>,
}

fn inventory() -> Vec<InventoryRow> {
    serde_json::from_str(include_str!(
        "../../../../../tests/ksr_fdmtest_v4/options-v242.json"
    ))
    .unwrap()
}

fn gcode_rows(rows: &[InventoryRow]) -> Vec<&InventoryRow> {
    rows.iter()
        .filter(|row| {
            row.effective_projections
                .iter()
                .any(|projection| projection == "g_code")
        })
        .collect()
}
