use std::collections::BTreeSet;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct OptionInventoryRow {
    key: String,
    default_serialized: String,
    upstream_consumers: Vec<SourceCitation>,
}

#[derive(Debug, Deserialize)]
struct SourceCitation {
    path: String,
}

fn committed_inventory() -> Vec<OptionInventoryRow> {
    serde_json::from_str(include_str!(
        "../../../tests/ksr_fdmtest_v4/options-v242.json"
    ))
    .unwrap()
}

#[test]
fn committed_inventory_is_available_without_an_orca_checkout() {
    let rows = committed_inventory();
    assert_eq!(rows.len(), 653);
    assert_eq!(
        rows.iter()
            .map(|row| &row.key)
            .collect::<BTreeSet<_>>()
            .len(),
        653
    );
    for row in &rows {
        assert!(!row.upstream_consumers.is_empty(), "{}", row.key);
        for citation in &row.upstream_consumers {
            assert!(
                !matches!(
                    citation.path.as_str(),
                    "src/libslic3r/PrintConfig.hpp"
                        | "src/libslic3r/PrintConfig.cpp"
                        | "src/libslic3r/Preset.cpp"
                ),
                "{} has a declaration/static-list consumer",
                row.key
            );
        }
    }
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
