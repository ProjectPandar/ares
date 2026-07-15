use serde::Serialize;

use crate::options::{
    AmsCounts, CsvTable, ExtruderVariantLists, FloatOrPercent, NozzleType, Nullable,
    NullableFloats, NullableInts, NullableNozzleTypes, OrcaBool, OrcaBools, OrcaFloat,
    OrcaFloats, OrcaInt, OrcaInts, OrcaString, OrcaStrings, OrcaUInt, Percent, Point2d,
    Point2dGroups, Point2dList, PrinterTechnologies, PrinterTechnology, RammingParameters,
    SpaceTuple, VariantStride, ZHopType,
    config_export::value::{SerializedConfigValue, serialize_config_value},
    config_types::semantic::NullableVectorRef,
};

fn assert_config_value<T>(value: &T, token: &str, is_nil: bool)
where
    T: Serialize + ?Sized,
{
    assert_eq!(
        serialize_config_value(value).unwrap(),
        SerializedConfigValue {
            token: token.to_owned(),
            is_nil,
        }
    );
}

fn assert_json<T>(value: &T, expected: &str)
where
    T: Serialize + ?Sized,
{
    assert_eq!(serde_json::to_string(value).unwrap(), expected);
}

#[test]
fn config_export_value_serializes_typed_scalars_and_enums() {
    assert_config_value(&OrcaBool(true), "1", false);
    assert_config_value(&OrcaBool(false), "0", false);
    assert_config_value(&OrcaInt(-17), "-17", false);
    assert_config_value(&OrcaUInt(u32::MAX), "4294967295", false);
    assert_config_value(&OrcaFloat(-0.0), "-0", false);
    assert_config_value(&OrcaFloat(1.23456789), "1.23457", false);
    assert_config_value(&OrcaFloat(0.00001), "1e-05", false);
    assert_config_value(&OrcaFloat(1_000_000.0), "1e+06", false);
    assert_config_value(&Percent(50.0), "50%", false);
    assert_config_value(&FloatOrPercent::Float(1.5), "1.5", false);
    assert_config_value(
        &FloatOrPercent::Percent(Percent(25.0)),
        "25%",
        false,
    );
    assert_config_value(&PrinterTechnology::Fff, "FFF", false);
    assert_config_value(&ZHopType::Auto, "Auto Lift", false);
}

#[test]
fn config_export_value_escapes_scalar_strings_once_without_outer_quotes() {
    assert_config_value(
        &OrcaString("line\r\n\\\"tail".to_owned()),
        r#"line\r\n\\\"tail"#,
        false,
    );
}

#[test]
fn config_export_value_serializes_string_vectors_with_orca_rules() {
    assert_config_value(&OrcaStrings(Vec::new()), "", false);
    assert_config_value(&OrcaStrings(vec![String::new()]), r#""""#, false);
    assert_config_value(
        &OrcaStrings(vec![String::new(), "two".to_owned(), String::new()]),
        ";two;",
        false,
    );
    assert_config_value(
        &OrcaStrings(vec![
            "plain".to_owned(),
            "two words".to_owned(),
            "tab\tvalue".to_owned(),
            "line\r\n".to_owned(),
            "slash\\\"quote".to_owned(),
        ]),
        "plain;\"two words\";\"tab\tvalue\";\"line\\r\\n\";\"slash\\\\\\\"quote\"",
        false,
    );
}

#[test]
fn config_export_value_tags_every_special_string_vector_wrapper() {
    let values = vec!["two words".to_owned(), "plain".to_owned()];
    let expected = "\"two words\";plain";
    assert_config_value(&AmsCounts(values.clone()), expected, false);
    assert_config_value(&RammingParameters(values.clone()), expected, false);
    assert_config_value(&CsvTable(values.clone()), expected, false);
    assert_config_value(&SpaceTuple(values.clone()), expected, false);
    assert_config_value(&VariantStride(values.clone()), expected, false);
    assert_config_value(&ExtruderVariantLists(values), expected, false);
}

#[test]
fn config_export_value_serializes_sequences_points_and_point_groups() {
    assert_config_value(
        &OrcaBools(vec![OrcaBool(true), OrcaBool(false)]),
        "1,0",
        false,
    );
    assert_config_value(
        &OrcaInts(vec![OrcaInt(-1), OrcaInt(2)]),
        "-1,2",
        false,
    );
    assert_config_value(
        &PrinterTechnologies(vec![
            PrinterTechnology::Fff,
            PrinterTechnology::Sla,
        ]),
        "FFF,SLA",
        false,
    );
    assert_config_value(&Point2d::new(1.5, -2.0), "1.5,-2", false);
    assert_config_value(
        &Point2dList(vec![Point2d::new(1.0, 2.0), Point2d::new(3.0, 4.0)]),
        "1x2,3x4",
        false,
    );
    assert_config_value(
        &Point2dGroups(vec![
            vec![Point2d::new(1.0, 2.0), Point2d::new(3.0, 4.0)],
            vec![Point2d::new(5.0, 6.0)],
        ]),
        "1x2,3x4#5x6",
        false,
    );
}

#[test]
fn config_export_value_preserves_explicit_nullable_state() {
    let empty = Vec::<Nullable<OrcaFloat>>::new();
    assert_config_value(&NullableVectorRef::new(&empty), "", true);

    let all_nil = vec![Nullable::<OrcaFloat>::Nil, Nullable::Nil];
    assert_config_value(&NullableVectorRef::new(&all_nil), "nil,nil", true);

    let mixed = vec![Nullable::Nil, Nullable::Value(OrcaFloat(2.5))];
    assert_config_value(&NullableVectorRef::new(&mixed), "nil,2.5", false);

    let all_values = vec![
        Nullable::Value(OrcaFloat(1.0)),
        Nullable::Value(OrcaFloat(2.0)),
    ];
    assert_config_value(&NullableVectorRef::new(&all_values), "1,2", false);
    assert_config_value(&OrcaFloats(Vec::new()), "", false);
    assert_config_value(&Nullable::<OrcaFloat>::Nil, "nil", true);
    assert_config_value(&NullableFloats(Vec::new()), "", true);
    assert_config_value(
        &NullableInts(vec![Nullable::Nil, Nullable::Nil]),
        "nil,nil",
        true,
    );
    assert_config_value(
        &NullableNozzleTypes(vec![
            Nullable::Nil,
            Nullable::Value(NozzleType::Brass),
        ]),
        "nil,brass",
        false,
    );
}

#[test]
fn config_export_value_rejects_unsupported_maps_and_structs() {
    #[derive(Serialize)]
    struct UnsupportedStruct {
        value: OrcaInt,
    }

    let error = serialize_config_value(&UnsupportedStruct {
        value: OrcaInt(1),
    })
    .unwrap_err();
    assert!(error.to_string().contains("struct"));

    let map = std::collections::BTreeMap::from([("value", OrcaInt(1))]);
    let error = serialize_config_value(&map).unwrap_err();
    assert!(error.to_string().contains("map"));
}

#[test]
fn config_export_value_semantic_tags_preserve_json_wire_shape() {
    assert_json(&OrcaStrings(vec!["orca".to_owned()]), r#"["orca"]"#);
    assert_json(&AmsCounts(vec!["1#0|4#0".to_owned()]), r#"["1#0|4#0"]"#);
    assert_json(
        &RammingParameters(vec!["120 100 6.6| 0.05 6.6".to_owned()]),
        r#"["120 100 6.6| 0.05 6.6"]"#,
    );
    assert_json(&CsvTable(vec!["0,0,0\n1,2,3".to_owned()]), r#"["0,0,0\n1,2,3"]"#);
    assert_json(&SpaceTuple(vec!["0 0 0".to_owned()]), r#"["0 0 0"]"#);
    assert_json(&VariantStride(vec!["Direct Drive".to_owned()]), r#"["Direct Drive"]"#);
    assert_json(
        &ExtruderVariantLists(vec!["Direct Drive Standard".to_owned()]),
        r#"["Direct Drive Standard"]"#,
    );
    assert_json(
        &Point2dGroups(vec![vec![Point2d::new(1.0, 2.0)]]),
        r#"["1x2"]"#,
    );

    let nullable = vec![Nullable::Nil, Nullable::Value(OrcaFloat(1.5))];
    assert_json(&NullableVectorRef::new(&nullable), r#"["nil","1.5"]"#);
    assert_json(&Nullable::<OrcaFloat>::Nil, r#""nil""#);
    assert_json(&Nullable::Value(OrcaFloat(1.5)), r#""1.5""#);
    assert_json(
        &NullableFloats(nullable.clone()),
        r#"["nil","1.5"]"#,
    );
    assert_json(
        &NullableInts(vec![Nullable::Nil, Nullable::Value(OrcaInt(2))]),
        r#"["nil","2"]"#,
    );
    assert_json(
        &NullableNozzleTypes(vec![
            Nullable::Nil,
            Nullable::Value(NozzleType::Brass),
        ]),
        r#"["nil","brass"]"#,
    );
}
