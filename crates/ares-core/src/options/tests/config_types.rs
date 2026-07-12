use serde::{Deserialize, Serialize};

use crate::options::{
    config_types::{
        AmsCounts, CsvTable, FlatMatrix, FloatOrPercent, Millimeters, Nullable, OrcaBool,
        OrcaBools, OrcaFloat, OrcaFloatOrPercents, OrcaFloats, OrcaInt, OrcaInts, OrcaPercents,
        NullablePrinterTechnologies, OrcaString, OrcaStrings, OrcaUInt, Percent, Point2d,
        Point2dGroups, Point2dList, PrinterTechnologies, PrinterTechnology, RammingParameters,
        SpaceTuple, VariantStride,
    },
    option_group::declare_option_group,
};

fn round_trip<T>(json: &str, expected: T, canonical_json: &str)
where
    T: std::fmt::Debug + PartialEq + Serialize + for<'de> Deserialize<'de>,
{
    let parsed: T = serde_json::from_str(json).unwrap();
    assert_eq!(parsed, expected);
    assert_eq!(serde_json::to_string(&parsed).unwrap(), canonical_json);
}

#[test]
fn config_types_parse_scalar_project_wire_forms() {
    round_trip(r#""1""#, OrcaBool(true), r#""1""#);
    round_trip("true", OrcaBool(true), r#""1""#);
    round_trip(r#""-17""#, OrcaInt(-17), r#""-17""#);
    round_trip("23", OrcaUInt(23), r#""23""#);
    round_trip(r#""1.25""#, OrcaFloat(1.25), r#""1.25""#);
    round_trip("1.25", OrcaFloat(1.25), r#""1.25""#);
    round_trip("0.4", Millimeters(0.4), r#""0.4""#);
    round_trip("125", Percent(125.0), r#""125%""#);
    round_trip("1.5", FloatOrPercent::Float(1.5), r#""1.5""#);
    round_trip(
        r#""1.23456789""#,
        OrcaFloat(1.23456789),
        r#""1.23457""#,
    );
    round_trip(r#""0.4""#, Millimeters(0.4), r#""0.4""#);
    round_trip(r#""50%""#, Percent(50.0), r#""50%""#);
    round_trip(r#""125""#, Percent(125.0), r#""125%""#);
    round_trip(r#""50 %""#, Percent(50.0), r#""50%""#);
    round_trip(
        r#""1.5""#,
        FloatOrPercent::Float(1.5),
        r#""1.5""#,
    );
    round_trip(
        r#""25%""#,
        FloatOrPercent::Percent(Percent(25.0)),
        r#""25%""#,
    );
    round_trip(
        r#""25 %""#,
        FloatOrPercent::Percent(Percent(25.0)),
        r#""25%""#,
    );
    round_trip(
        r#""nil""#,
        Nullable::<OrcaFloat>::Nil,
        r#""nil""#,
    );
    round_trip(
        r#""2.5""#,
        Nullable::Value(OrcaFloat(2.5)),
        r#""2.5""#,
    );
    round_trip(r#""1,0""#, OrcaBool(true), r#""1""#);
    round_trip("23", OrcaInt(23), r#""23""#);
    round_trip("23", OrcaUInt(23), r#""23""#);
    round_trip("-2147483648", OrcaInt(i32::MIN), r#""-2147483648""#);
    round_trip("2147483647", OrcaInt(i32::MAX), r#""2147483647""#);
    round_trip("4294967295", OrcaUInt(u32::MAX), r#""4294967295""#);
    round_trip(r#""-0""#, OrcaFloat(-0.0), r#""-0""#);
    round_trip(r#""0.00001""#, OrcaFloat(0.00001), r#""1e-05""#);
    round_trip(r#""1000000""#, OrcaFloat(1_000_000.0), r#""1e+06""#);
    round_trip(r#""999999.9""#, OrcaFloat(999_999.9), r#""1e+06""#);
    round_trip(r#""0.00009999996""#, OrcaFloat(0.000_099_999_96), r#""0.0001""#);
}

#[test]
fn config_types_reject_invalid_scalar_lexemes() {
    for invalid in [r#""true""#, r#""2""#, r#""""#] {
        assert!(serde_json::from_str::<OrcaBool>(invalid).is_err());
    }
    for invalid in [r#""NaN""#, r#""inf""#, r#""1.2.3""#] {
        assert!(serde_json::from_str::<OrcaFloat>(invalid).is_err());
    }
    for invalid in [r#""-1""#, r#""1.5""#] {
        assert!(serde_json::from_str::<OrcaUInt>(invalid).is_err());
    }
    assert!(serde_json::from_str::<OrcaInt>("18446744073709551615").is_err());
    assert!(serde_json::from_str::<OrcaInt>("2147483648").is_err());
    assert!(serde_json::from_str::<OrcaInt>("-2147483649").is_err());
    assert!(serde_json::from_str::<OrcaUInt>("4294967296").is_err());
    assert!(serde_json::from_str::<OrcaUInt>("-1").is_err());
    for invalid in [r#""%""#, r#""50%%""#] {
        assert!(serde_json::from_str::<Percent>(invalid).is_err());
    }
    assert!(serde_json::from_str::<Point2dList>(r#"["1,2"]"#).is_err());
}

#[test]
fn config_types_reject_non_finite_values_during_serialization() {
    assert!(serde_json::to_string(&OrcaFloat(f64::NAN)).is_err());
    assert!(serde_json::to_string(&Millimeters(f64::INFINITY)).is_err());
    assert!(serde_json::to_string(&Percent(f64::NEG_INFINITY)).is_err());
    assert!(serde_json::to_string(&FloatOrPercent::Float(f64::NAN)).is_err());
    assert!(serde_json::to_string(&Point2d::new(f64::NAN, 0.0)).is_err());
    assert!(serde_json::to_string(&FlatMatrix(vec![0.0, f64::INFINITY])).is_err());
}

#[test]
fn config_types_cover_typed_vector_and_exact_enum_forms() {
    round_trip(
        r#"["1","0"]"#,
        OrcaBools(vec![OrcaBool(true), OrcaBool(false)]),
        r#"["1","0"]"#,
    );
    round_trip(
        r#"["-1","2"]"#,
        OrcaInts(vec![OrcaInt(-1), OrcaInt(2)]),
        r#"["-1","2"]"#,
    );
    round_trip(
        r#"["1.25","2"]"#,
        OrcaFloats(vec![OrcaFloat(1.25), OrcaFloat(2.0)]),
        r#"["1.25","2"]"#,
    );
    round_trip(
        r#"["10%","125"]"#,
        OrcaPercents(vec![Percent(10.0), Percent(125.0)]),
        r#"["10%","125%"]"#,
    );
    round_trip(
        r#"["10 %"]"#,
        OrcaPercents(vec![Percent(10.0)]),
        r#"["10%"]"#,
    );
    round_trip(
        r#"["1.5","10%"]"#,
        OrcaFloatOrPercents(vec![
            FloatOrPercent::Float(1.5),
            FloatOrPercent::Percent(Percent(10.0)),
        ]),
        r#"["1.5","10%"]"#,
    );
    round_trip(r#""FFF""#, PrinterTechnology::Fff, r#""FFF""#);
    round_trip(
        r#"["FFF","SLA"]"#,
        PrinterTechnologies(vec![PrinterTechnology::Fff, PrinterTechnology::Sla]),
        r#"["FFF","SLA"]"#,
    );
    round_trip(
        r#"["nil","FFF"]"#,
        NullablePrinterTechnologies(vec![
            Nullable::Nil,
            Nullable::Value(PrinterTechnology::Fff),
        ]),
        r#"["nil","FFF"]"#,
    );
    assert!(serde_json::from_str::<NullablePrinterTechnologies>(r#"[null,"FFF"]"#).is_err());
    assert!(serde_json::from_str::<PrinterTechnology>(r#""fff""#).is_err());
    assert!(serde_json::from_str::<PrinterTechnology>(r#""future""#).is_err());
}

#[test]
fn config_types_preserve_strings_vectors_points_and_special_encodings() {
    round_trip(r#""""#, OrcaString(String::new()), r#""""#);
    round_trip(
        r#""line 1\nline 2""#,
        OrcaString("line 1\nline 2".to_owned()),
        r#""line 1\nline 2""#,
    );
    round_trip("[]", OrcaStrings(Vec::new()), "[]");
    round_trip(
        r#"["","two"]"#,
        OrcaStrings(vec![String::new(), "two".to_owned()]),
        r#"["","two"]"#,
    );
    round_trip(r#""3.5,-2""#, Point2d::new(3.5, -2.0), r#""3.5,-2""#);
    round_trip(
        r#"["0x0","2.5x3"]"#,
        Point2dList(vec![Point2d::new(0.0, 0.0), Point2d::new(2.5, 3.0)]),
        r#"["0x0","2.5x3"]"#,
    );
    round_trip(
        r#"["0x0,10x0,10x10","2x3,4x5"]"#,
        Point2dGroups(vec![
            vec![
                Point2d::new(0.0, 0.0),
                Point2d::new(10.0, 0.0),
                Point2d::new(10.0, 10.0),
            ],
            vec![Point2d::new(2.0, 3.0), Point2d::new(4.0, 5.0)],
        ]),
        r#"["0x0,10x0,10x10","2x3,4x5"]"#,
    );

    round_trip(
        r#"["Direct Drive Standard","Direct Drive High Flow","Bowden Standard","Bowden High Flow","Direct Drive Standard","Direct Drive High Flow","Bowden Standard","Bowden High Flow"]"#,
        VariantStride(vec![
            "Direct Drive Standard".to_owned(),
            "Direct Drive High Flow".to_owned(),
            "Bowden Standard".to_owned(),
            "Bowden High Flow".to_owned(),
            "Direct Drive Standard".to_owned(),
            "Direct Drive High Flow".to_owned(),
            "Bowden Standard".to_owned(),
            "Bowden High Flow".to_owned(),
        ]),
        r#"["Direct Drive Standard","Direct Drive High Flow","Bowden Standard","Bowden High Flow","Direct Drive Standard","Direct Drive High Flow","Bowden Standard","Bowden High Flow"]"#,
    );
    round_trip(
        r#"["0","280","280","0","0","280","280","0"]"#,
        FlatMatrix(vec![0.0, 280.0, 280.0, 0.0, 0.0, 280.0, 280.0, 0.0]),
        r#"["0","280","280","0","0","280","280","0"]"#,
    );
    round_trip(
        r#"["1#0|4#0","1#0|4#0"]"#,
        AmsCounts(vec!["1#0|4#0".to_owned(), "1#0|4#0".to_owned()]),
        r#"["1#0|4#0","1#0|4#0"]"#,
    );
    round_trip(
        r#"["120 100 6.6| 0.05 6.6","120 100 6.6| 0.05 6.6"]"#,
        RammingParameters(vec![
            "120 100 6.6| 0.05 6.6".to_owned(),
            "120 100 6.6| 0.05 6.6".to_owned(),
        ]),
        r#"["120 100 6.6| 0.05 6.6","120 100 6.6| 0.05 6.6"]"#,
    );
    round_trip(
        r#"["0,0,0\n1,2,3"]"#,
        CsvTable(vec!["0,0,0\n1,2,3".to_owned()]),
        r#"["0,0,0\n1,2,3"]"#,
    );
    round_trip(
        r#"["0 0 0 0 0 0"]"#,
        SpaceTuple(vec!["0 0 0 0 0 0".to_owned()]),
        r#"["0 0 0 0 0 0"]"#,
    );
}

declare_option_group! {
    pub(crate) struct CodecGroup, CodecGroupBuilder {
        enabled => "enabled": OrcaBool = OrcaBool(false),
        distance => "distance": Millimeters = Millimeters(0.4),
        overrides => "overrides": Vec<Nullable<OrcaFloat>> = vec![Nullable::Value(OrcaFloat(1.0))],
    }
}

declare_option_group! {
    pub(crate) struct SecondaryGroup, SecondaryGroupBuilder {
        name => "name": OrcaString = OrcaString("default".to_owned()),
    }
}

#[derive(Debug)]
struct DispatchGroups {
    primary: CodecGroup,
    secondary: SecondaryGroup,
}

impl<'de> Deserialize<'de> for DispatchGroups {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = DispatchGroups;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("an Orca option group")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                deserialize_dispatch_map(&mut map)
            }
        }

        deserializer.deserialize_map(Visitor)
    }
}

fn deserialize_dispatch_map<'de, A>(map: &mut A) -> Result<DispatchGroups, A::Error>
where
    A: serde::de::MapAccess<'de>,
{
    let mut builder = CodecGroupBuilder::default();
    let mut secondary = SecondaryGroupBuilder::default();
    while let Some(key) = map.next_key::<String>()? {
        if builder.deserialize_known_field(&key, map)?
            || secondary.deserialize_known_field(&key, map)?
        {
            continue;
        }
        return Err(serde::de::Error::custom(format!(
            "unknown Orca option {key}"
        )));
    }
    Ok(DispatchGroups {
        primary: builder.resolve(),
        secondary: secondary.resolve(),
    })
}

#[test]
fn option_group_dispatches_typed_values_and_resolves_defaults() {
    let group = serde_json::from_str::<DispatchGroups>(r#"{"enabled":"1"}"#)
        .unwrap()
        .primary;
    assert_eq!(group.enabled, OrcaBool(true));
    assert_eq!(group.distance, Millimeters(0.4));
    assert_eq!(
        group.overrides,
        vec![Nullable::Value(OrcaFloat(1.0))]
    );

    let error = serde_json::from_str::<DispatchGroups>(r#"{"future":{"not":"consumed"}}"#)
        .unwrap_err()
        .to_string();
    assert!(error.contains("unknown Orca option future"));
    assert!(!error.contains("invalid type"));
}

#[test]
fn option_group_distinguishes_missing_nil_and_value_and_rejects_null() {
    let missing = serde_json::from_str::<DispatchGroups>("{}").unwrap().primary;
    assert_eq!(
        missing.overrides,
        vec![Nullable::Value(OrcaFloat(1.0))]
    );

    let explicit = serde_json::from_str::<DispatchGroups>(
        r#"{"overrides":["nil","2.5"]}"#,
    )
    .unwrap()
    .primary;
    assert_eq!(
        explicit.overrides,
        vec![Nullable::Nil, Nullable::Value(OrcaFloat(2.5))]
    );
    assert!(serde_json::from_str::<DispatchGroups>(r#"{"overrides":[null]}"#).is_err());
    let spaced = serde_json::from_str::<DispatchGroups>(r#"{"overrides":[" nil "]}"#)
        .unwrap()
        .primary;
    assert_eq!(spaced.overrides, vec![Nullable::Nil]);
}

#[test]
fn nonmatching_group_leaves_the_value_for_the_next_typed_group() {
    let parsed: DispatchGroups = serde_json::from_str(r#"{"name":"picked"}"#).unwrap();
    assert_eq!(parsed.primary.enabled, OrcaBool(false));
    assert_eq!(parsed.secondary.name.0, "picked");

    let error = serde_json::from_str::<DispatchGroups>(r#"{"name":42}"#)
        .unwrap_err()
        .to_string();
    assert!(error.contains("string"));
    assert!(!error.contains("unknown Orca option"));
}

#[test]
fn option_group_rejects_duplicate_known_keys() {
    let error = serde_json::from_str::<DispatchGroups>(r#"{"enabled":"1","enabled":"0"}"#)
        .unwrap_err()
        .to_string();
    assert!(error.contains("duplicate Orca option enabled"));
}
