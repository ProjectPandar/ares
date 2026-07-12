use super::super::*;
use crate::{SliceError, options::support_type::SupportType};
use serde_json::{Value, json};

fn options(value: Value) -> SliceOptions {
    serde_json::from_value(value).unwrap()
}

fn parsed(value: Value) -> Result<SupportType, SliceError> {
    options(json!({ "support_type": value })).support_type()
}

fn raw_parsed(value: Value) -> Result<SupportType, SliceError> {
    SliceOptions {
        values: std::collections::BTreeMap::from([("support_type".to_owned(), value)]),
    }
    .support_type()
}

#[test]
fn support_type_defaults_to_normal_auto() {
    assert_eq!(
        SliceOptions::default().support_type().unwrap(),
        SupportType::NormalAuto
    );
}

#[test]
fn support_type_parses_canonical_values() {
    for (value, expected) in [
        ("normal(auto)", SupportType::NormalAuto),
        ("tree(auto)", SupportType::TreeAuto),
        ("normal(manual)", SupportType::NormalManual),
        ("tree(manual)", SupportType::TreeManual),
    ] {
        assert_eq!(parsed(json!(value)).unwrap(), expected);
    }
}

#[test]
fn support_type_helpers_match_upstream_truth_table() {
    for (value, is_tree, is_auto) in [
        (SupportType::NormalAuto, false, true),
        (SupportType::TreeAuto, true, true),
        (SupportType::NormalManual, false, false),
        (SupportType::TreeManual, true, false),
    ] {
        assert_eq!(value.is_tree(), is_tree);
        assert_eq!(value.is_auto(), is_auto);
    }
}

#[test]
fn support_type_legacy_values_resolve_to_canonical_variants() {
    for (value, expected) in [
        ("normal", SupportType::NormalManual),
        ("tree", SupportType::TreeManual),
        ("hybrid(auto)", SupportType::TreeAuto),
    ] {
        assert_eq!(parsed(json!(value)).unwrap(), expected);
    }
}

#[test]
fn support_type_rejects_invalid_strings() {
    for value in ["", "normal", "tree", "hybrid(auto)", "tree(slim)", "unknown"] {
        let err = raw_parsed(json!(value)).unwrap_err();
        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("support_type"));
    }
}

#[test]
fn support_type_rejects_non_strings() {
    for value in [json!(true), json!(1), json!(1.0), json!([]), json!({}), Value::Null] {
        let err = parsed(value).unwrap_err();
        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("support_type"));
    }
}
