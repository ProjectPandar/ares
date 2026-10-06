use std::collections::BTreeMap;

use serde::de::DeserializeOwned;

use super::deserialize_preset_config;
use crate::{
    FloatOrPercent, GCodeFlavor, OrcaBool, OrcaFloat, OrcaInt, Percent,
    project::raw_settings::RawConfigValue,
};

fn decode<T: DeserializeOwned>(key: &str, value: &str) -> Result<T, String> {
    let input = RawConfigValue::Object(BTreeMap::from([(
        key.to_owned(),
        RawConfigValue::Text(value.to_owned()),
    )]));
    let values: BTreeMap<String, T> = deserialize_preset_config(input)?;
    Ok(values.into_values().next().unwrap())
}

#[test]
fn preset_scalar_stream_prefixes_follow_native_types() {
    for (text, expected) in [
        ("50,50", 50.0),
        ("  +50.5suffix", 50.5),
        ("1.25e2tail", 125.0),
        ("1.2.3", 1.2),
        ("0x10", 0.0),
        ("1e-400", 0.0),
    ] {
        assert_eq!(
            decode::<OrcaFloat>("bridge_speed", text).unwrap().0,
            expected
        );
    }
    assert_eq!(decode::<OrcaInt>("wall_loops", "3.5tail").unwrap().0, 3);
    assert_eq!(
        decode::<OrcaInt>("wall_loops", "-2147483648tail")
            .unwrap()
            .0,
        i32::MIN
    );
    assert_eq!(
        decode::<Percent>("sparse_infill_density", "15tail%").unwrap(),
        Percent(15.0)
    );
    assert_eq!(
        decode::<FloatOrPercent>("line_width", "50,50%tail").unwrap(),
        FloatOrPercent::Percent(Percent(50.0))
    );
    assert_eq!(
        decode::<FloatOrPercent>("line_width", "0.4tail").unwrap(),
        FloatOrPercent::Float(0.4)
    );
    assert!(
        decode::<OrcaBool>("enable_overhang_speed", " 1 ,0junk")
            .unwrap()
            .0
    );
    assert!(
        !decode::<OrcaBool>("enable_overhang_speed", "0,1")
            .unwrap()
            .0
    );
}

#[test]
fn preset_scalar_invalid_numbers_and_exact_bool_enum_remain_errors() {
    for text in ["", "junk50", "1e", "1e+bad", "1e309", "nan", "inf"] {
        assert!(decode::<OrcaFloat>("bridge_speed", text).is_err(), "{text}");
    }
    assert!(decode::<OrcaInt>("wall_loops", "2147483648,0").is_err());
    for text in ["true,0", "1junk,0", "2,1", ""] {
        assert!(
            decode::<OrcaBool>("enable_overhang_speed", text).is_err(),
            "{text}"
        );
    }
    assert_eq!(
        decode::<GCodeFlavor>("gcode_flavor", "marlin").unwrap(),
        GCodeFlavor::MarlinLegacy
    );
    for text in ["marlin,marlin", "marlinjunk", " marlin "] {
        assert!(
            decode::<GCodeFlavor>("gcode_flavor", text).is_err(),
            "{text}"
        );
    }
}

#[test]
fn preset_scalar_conversion_does_not_loosen_typed_deserialization() {
    assert!(serde_json::from_str::<OrcaFloat>("\"50,50\"").is_err());
    assert!(serde_json::from_str::<OrcaInt>("\"3.5tail\"").is_err());
    assert!(serde_json::from_str::<Percent>("\"15tail%\"").is_err());
    assert!(serde_json::from_str::<FloatOrPercent>("\"50,50%tail\"").is_err());
    assert!(serde_json::from_str::<GCodeFlavor>("\"marlin,marlin\"").is_err());
}
