use serde_json::{Value, json};

use super::super::super::{
    FilamentOptions, FilamentPrintSourceOptions, FilamentRetractOverrideOptions,
    RawOverhangFanThreshold,
};
use super::fixture_fields;

#[test]
fn strict_enum_domains_accept_every_canonical_token() {
    for token in ["0%", "10%", "25%", "50%", "75%", "95%"] {
        assert_round_trip::<FilamentPrintSourceOptions>("overhang_fan_threshold", token);
        assert_round_trip::<FilamentOptions>("overhang_fan_threshold", token);
    }
    for token in ["All Surfaces", "Top Only", "Bottom Only", "Top and Bottom"] {
        assert_round_trip::<FilamentRetractOverrideOptions>(
            "filament_retract_lift_enforce",
            token,
        );
        assert_round_trip::<FilamentOptions>("filament_retract_lift_enforce", token);
    }
    for token in ["Auto Lift", "Normal Lift", "Slope Lift", "Spiral Lift"] {
        assert_round_trip::<FilamentRetractOverrideOptions>("filament_z_hop_types", token);
        assert_round_trip::<FilamentOptions>("filament_z_hop_types", token);
    }
}

#[test]
fn strict_enum_domains_reject_unknown_case_variants_and_legacy_five_percent() {
    for (key, invalid) in [
        ("overhang_fan_threshold", "5%"),
        ("overhang_fan_threshold", "95 %"),
        ("overhang_fan_threshold", "95% "),
        ("filament_retract_lift_enforce", "all surfaces"),
        ("filament_retract_lift_enforce", "Top"),
        ("filament_z_hop_types", "Spiral lift"),
        ("filament_z_hop_types", "Normal"),
    ] {
        let input = json!({key: [invalid]});
        let child_error = if key == "overhang_fan_threshold" {
            serde_json::from_value::<FilamentPrintSourceOptions>(input.clone())
                .unwrap_err()
                .to_string()
        } else {
            serde_json::from_value::<FilamentRetractOverrideOptions>(input.clone())
                .unwrap_err()
                .to_string()
        };
        let parent_error = serde_json::from_value::<FilamentOptions>(input)
            .unwrap_err()
            .to_string();
        assert!(child_error.contains(key), "{child_error}");
        assert!(parent_error.contains(key), "{parent_error}");
    }
}

#[test]
fn enum_defaults_and_fixture_payloads_are_exact() {
    let print = FilamentPrintSourceOptions::default();
    let retract = FilamentRetractOverrideOptions::default();
    assert_eq!(print.overhang_fan_threshold, vec![RawOverhangFanThreshold::Percent95]);
    assert_eq!(
        retract.filament_retract_lift_enforce,
        vec![super::super::super::Nullable::Nil]
    );
    assert_eq!(
        retract.filament_z_hop_types,
        vec![super::super::super::Nullable::Nil]
    );
    let fixture = fixture_fields([
        "overhang_fan_threshold",
        "filament_retract_lift_enforce",
        "filament_z_hop_types",
    ]);
    assert_eq!(fixture["overhang_fan_threshold"], json!(["50%", "50%"]));
    assert_eq!(
        fixture["filament_retract_lift_enforce"],
        Value::Array(vec![json!("nil"); 8])
    );
    assert_eq!(
        fixture["filament_z_hop_types"],
        Value::Array(vec![json!("Spiral Lift"); 8])
    );
}

fn assert_round_trip<T>(key: &str, token: &str)
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    let input = json!({key: [token]});
    let parsed: T = serde_json::from_value(input.clone()).unwrap();
    assert_eq!(serde_json::to_value(parsed).unwrap()[key], Value::Array(vec![json!(token)]));
}
