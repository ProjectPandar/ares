use std::collections::BTreeSet;

use serde::Deserialize;

use super::source_names;

#[derive(Deserialize)]
struct InventoryRow {
    legacy_inputs: Vec<LegacyInput>,
}

#[derive(Deserialize)]
struct LegacyInput {
    key: String,
}

#[test]
fn typed_legacy_fixture_oracle_has_exact_rows_names_and_source_delta() {
    let rows: Vec<InventoryRow> = serde_json::from_str(include_str!(
        "../../../../../../tests/ksr_fdmtest_v4/options-v242.json"
    ))
    .unwrap();
    let ledger_rows = rows
        .iter()
        .flat_map(|row| &row.legacy_inputs)
        .collect::<Vec<_>>();
    let ledger_names = ledger_rows
        .iter()
        .map(|input| input.key.as_str())
        .collect::<BTreeSet<_>>();

    assert_eq!(ledger_rows.len(), 88);
    assert_eq!(ledger_names.len(), 73);
    assert_eq!(
        source_names()
            .difference(&ledger_names)
            .copied()
            .collect::<Vec<_>>(),
        [
            "compatible_printers_condition_cummulative",
            "compatible_prints_condition_cummulative",
            "different_settings_to_system",
            "inherits_cummulative",
        ]
    );
    assert_eq!(
        ledger_names
            .difference(&source_names())
            .copied()
            .collect::<Vec<_>>(),
        ["perimeter_feed_rate"]
    );
}
