//! Failure inventory: empty numeric vectors must be nil; zero/false and mixed
//! nullable values must not vanish; empty strings/points remain nonnil; JSON is unchanged.
use super::value::serialize_config_value;
use crate::options::config_types::*;
use serde::Serialize;

fn check(value: &impl Serialize, expected_nil: bool, json: &str) {
    assert_eq!(serde_json::to_string(value).unwrap(), json);
    assert_eq!(serialize_config_value(value).unwrap().is_nil, expected_nil);
}

#[test]
fn vector_nil_is_owned_by_upstream_option_family() {
    check(&OrcaInts::default(), true, "[]");
    check(&OrcaFloats::default(), true, "[]");
    check(&OrcaPercents::default(), true, "[]");
    check(&OrcaFloatOrPercents::default(), true, "[]");
    check(&OrcaBools::default(), true, "[]");
    check(&FlatMatrix::default(), true, "[]");
    check(&OrcaInts(vec![OrcaInt(0)]), false, r#"["0"]"#);
    check(&OrcaBools(vec![OrcaBool(false)]), false, r#"["0"]"#);
    let nil = OrcaInts(vec![OrcaInt(i32::MAX)]);
    assert!(serialize_config_value(&nil).unwrap().is_nil);
    let mixed = OrcaInts(vec![OrcaInt(i32::MAX), OrcaInt(0)]);
    assert!(!serialize_config_value(&mixed).unwrap().is_nil);
    check(&OrcaStrings::default(), false, "[]");
    check(&Point2dList(Vec::new()), false, "[]");
    check(&Point2dGroups(Vec::new()), false, "[]");
}
