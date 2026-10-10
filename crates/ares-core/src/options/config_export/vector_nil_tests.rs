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
    check_nullable::<OrcaFloat>(OrcaFloat(0.0));
    check_nullable::<Percent>(Percent(0.0));
    check_nullable::<OrcaBool>(OrcaBool(false));
    check_nullable::<OrcaInt>(OrcaInt(0));
    check_nullable::<FloatOrPercent>(FloatOrPercent::Float(0.0));
}

fn check_nullable<T: Serialize>(value: T) {
    use crate::options::config_types::semantic::NullableVectorRef;
    assert!(
        serialize_config_value(&NullableVectorRef::<T>::new(&[]))
            .unwrap()
            .is_nil
    );
    assert!(
        serialize_config_value(&NullableVectorRef::<T>::new(&[Nullable::Nil]))
            .unwrap()
            .is_nil
    );
    assert!(
        !serialize_config_value(&NullableVectorRef::new(&[
            Nullable::Nil,
            Nullable::Value(value)
        ]))
        .unwrap()
        .is_nil
    );
}

#[test]
fn nil_vector_canonical_entries_are_not_resurrected_by_raw_footer_fallback() {
    use super::{collector::collect_config_entries, writer::write_canonical_entries};
    use crate::project::raw_settings::ProjectSettingsRaw;
    let mut settings = crate::ProjectSettings::default();
    settings.project.print.other_layers_print_sequence = OrcaInts(Vec::new());
    settings.project.print.first_layer_print_sequence = OrcaInts(vec![OrcaInt(0)]);
    let raw: ProjectSettingsRaw =
        serde_json::from_str(r#"{"other_layers_print_sequence":[],"post_process":[]}"#).unwrap();
    let entries = collect_config_entries(&settings).unwrap();
    let mut out = Vec::new();
    write_canonical_entries(&settings, 0, &entries, &raw, &mut out).unwrap();
    let text = String::from_utf8(out).unwrap();
    assert!(!text.contains("; other_layers_print_sequence = "));
    assert!(text.contains("; first_layer_print_sequence = 0\n"));
    assert!(text.contains("; post_process = \n"));
}
