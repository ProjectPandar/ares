use std::collections::{BTreeMap, BTreeSet};

use super::{
    ObjectOptionOverrides, ObjectOptions, ProcessObjectSourceOptions, inventory, object_rows,
    types,
};
use super::super::process_object_source::expected::DECLARATION_ORDER;

#[test]
fn object_options_inventory_matches_fixed_126_rows() {
    let rows = inventory();
    let object = object_rows(&rows);

    assert_eq!(object.len(), 126);
    assert!(
        object
            .iter()
            .all(|row| !row.nullable && row.wire_shape == "scalar_string")
    );
    assert_eq!(
        object.iter().fold(BTreeMap::new(), |mut counts, row| {
            *counts.entry(row.option_type.as_str()).or_insert(0) += 1;
            counts
        }),
        BTreeMap::from([
            ("coBool", 22),
            ("coEnum", 12),
            ("coFloat", 63),
            ("coFloatOrPercent", 6),
            ("coInt", 13),
            ("coPercent", 10),
        ])
    );

    let inventory_keys = object
        .iter()
        .map(|row| row.key.as_str())
        .collect::<BTreeSet<_>>();
    let declaration_keys = ObjectOptions::DECLARATION_ORDER
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    assert_eq!(ObjectOptions::DECLARATION_ORDER.len(), 126);
    assert_eq!(
        ObjectOptions::DECLARATION_ORDER,
        DECLARATION_ORDER
    );
    assert_eq!(ProcessObjectSourceOptions::DECLARATION_ORDER, DECLARATION_ORDER);
    assert_eq!(declaration_keys, inventory_keys);
}

#[test]
fn object_options_inventory_expands_concrete_effective_and_sparse_structs() {
    let base = ProcessObjectSourceOptions::default();
    let effective = ObjectOptions::from_base(&base);
    let sparse = ObjectOptionOverrides::default();

    types::assert_base_and_sparse(&effective, &base, &sparse);
}
