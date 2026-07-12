use super::super::*;
use crate::{
    SliceError,
    options::{support_style::SupportStyle, support_type::SupportType},
};
use serde_json::{Value, json};

fn options(value: Value) -> SliceOptions {
    serde_json::from_value(value).unwrap()
}

fn parsed(value: Value) -> Result<SupportStyle, SliceError> {
    options(json!({ "support_style": value })).support_style()
}

#[test]
fn support_style_defaults_to_default() {
    assert_eq!(
        SliceOptions::default().support_style().unwrap(),
        SupportStyle::Default
    );
}

#[test]
fn support_style_parses_canonical_values() {
    for (value, expected) in [
        ("default", SupportStyle::Default),
        ("grid", SupportStyle::Grid),
        ("snug", SupportStyle::Snug),
        ("organic", SupportStyle::TreeOrganic),
        ("tree_slim", SupportStyle::TreeSlim),
        ("tree_strong", SupportStyle::TreeStrong),
        ("tree_hybrid", SupportStyle::TreeHybrid),
    ] {
        assert_eq!(parsed(json!(value)).unwrap(), expected);
    }
}

#[test]
fn support_style_rejects_invalid_strings() {
    for value in ["", "tree", "normal", "organic_tree", "tree-organic"] {
        let err = parsed(json!(value)).unwrap_err();
        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("support_style"));
    }
}

#[test]
fn support_style_rejects_non_strings() {
    for value in [
        json!(true),
        json!(1),
        json!(1.0),
        json!([]),
        json!({}),
        Value::Null,
    ] {
        let err = parsed(value).unwrap_err();
        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("support_style"));
    }
}

#[test]
fn support_style_tree_style_truth_table_matches_upstream_grouping() {
    for (style, is_tree_style) in [
        (SupportStyle::Default, false),
        (SupportStyle::Grid, false),
        (SupportStyle::Snug, false),
        (SupportStyle::TreeOrganic, true),
        (SupportStyle::TreeSlim, true),
        (SupportStyle::TreeStrong, true),
        (SupportStyle::TreeHybrid, true),
    ] {
        assert_eq!(style.is_tree_style(), is_tree_style);
    }
}

#[test]
fn support_style_resolves_for_support_type_like_support_parameters() {
    for (style, support_type, expected) in [
        (
            SupportStyle::Default,
            SupportType::NormalAuto,
            SupportStyle::Grid,
        ),
        (
            SupportStyle::Default,
            SupportType::NormalManual,
            SupportStyle::Grid,
        ),
        (
            SupportStyle::Default,
            SupportType::TreeAuto,
            SupportStyle::TreeOrganic,
        ),
        (
            SupportStyle::Default,
            SupportType::TreeManual,
            SupportStyle::TreeOrganic,
        ),
        (
            SupportStyle::Grid,
            SupportType::TreeAuto,
            SupportStyle::TreeOrganic,
        ),
        (
            SupportStyle::Snug,
            SupportType::TreeManual,
            SupportStyle::TreeOrganic,
        ),
        (
            SupportStyle::TreeOrganic,
            SupportType::NormalAuto,
            SupportStyle::Grid,
        ),
        (
            SupportStyle::TreeSlim,
            SupportType::NormalManual,
            SupportStyle::Grid,
        ),
        (
            SupportStyle::TreeStrong,
            SupportType::NormalAuto,
            SupportStyle::Grid,
        ),
        (
            SupportStyle::TreeHybrid,
            SupportType::NormalManual,
            SupportStyle::Grid,
        ),
        (
            SupportStyle::TreeSlim,
            SupportType::TreeAuto,
            SupportStyle::TreeSlim,
        ),
        (
            SupportStyle::TreeSlim,
            SupportType::TreeManual,
            SupportStyle::TreeSlim,
        ),
        (
            SupportStyle::TreeHybrid,
            SupportType::TreeManual,
            SupportStyle::TreeHybrid,
        ),
    ] {
        assert_eq!(style.resolve_for_support_type(support_type), expected);
    }
}

#[test]
fn support_type_is_tree_slim_matches_upstream_helper() {
    for (support_type, style, expected) in [
        (SupportType::TreeAuto, SupportStyle::TreeSlim, true),
        (SupportType::TreeManual, SupportStyle::TreeSlim, true),
        (SupportType::NormalAuto, SupportStyle::TreeSlim, false),
        (SupportType::NormalManual, SupportStyle::TreeSlim, false),
        (SupportType::TreeAuto, SupportStyle::TreeOrganic, false),
        (SupportType::TreeAuto, SupportStyle::Grid, false),
    ] {
        assert_eq!(support_type.is_tree_slim(style), expected);
    }
}
