use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct OptionInventoryRow {
    key: String,
    default_serialized: String,
}

fn committed_inventory() -> Vec<OptionInventoryRow> {
    serde_json::from_str(include_str!(
        "../../../tests/ksr_fdmtest_v4/options-v242.json"
    ))
    .unwrap()
}

#[test]
fn committed_inventory_keeps_qualified_enum_defaults() {
    let rows = committed_inventory();
    let input_shaping = rows
        .iter()
        .find(|row| row.key == "input_shaping_type")
        .unwrap();
    assert_eq!(input_shaping.default_serialized, "Default");
    let nozzle_type = rows.iter().find(|row| row.key == "nozzle_type").unwrap();
    assert_eq!(nozzle_type.default_serialized, "undefine");
}
