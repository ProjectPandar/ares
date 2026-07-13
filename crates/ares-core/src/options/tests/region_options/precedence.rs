use serde_json::{Map, Value, json};

use crate::options::{
    OrcaInt, ProcessRegionSourceOptions,
    region_options::{RegionBase, RegionOptionOverrides, RegionOverrideSources},
};

use super::{RegionOptions, base::non_default_value, inventory, region_rows, resolve_region, types};

const FEATURE_KEYS: [&str; 6] = [
    "sparse_infill_filament_id",
    "internal_solid_filament_id",
    "top_surface_filament_id",
    "bottom_surface_filament_id",
    "outer_wall_filament_id",
    "inner_wall_filament_id",
];

struct FeatureScopes<'a> {
    object: &'a [(&'a str, &'a str)],
    volume: &'a [(&'a str, &'a str)],
    material: &'a [(&'a str, &'a str)],
    layer_range: &'a [(&'a str, &'a str)],
}

#[test]
fn ordinary_fields_follow_process_object_volume_material_layer_order() {
    assert_eq!(resolve_bottom_shell(None, None, None, None), OrcaInt(10));
    assert_eq!(resolve_bottom_shell(Some(20), None, None, None), OrcaInt(20));
    assert_eq!(
        resolve_bottom_shell(Some(20), Some(30), None, None),
        OrcaInt(30)
    );
    assert_eq!(
        resolve_bottom_shell(Some(20), Some(30), Some(40), None),
        OrcaInt(40)
    );
    assert_eq!(
        resolve_bottom_shell(Some(20), Some(30), Some(40), Some(50)),
        OrcaInt(50)
    );
}

#[test]
fn every_non_feature_region_field_uses_the_concrete_sparse_merge() {
    let rows = inventory();
    let mut object = RegionOptionOverrides::default();
    let process: ProcessRegionSourceOptions = serde_json::from_value(Value::Object(
        FEATURE_KEYS
            .into_iter()
            .map(|feature| (feature.to_owned(), Value::String("1".to_owned())))
            .collect(),
    ))
    .unwrap();
    let mut expected_values = serde_json::to_value(&process)
        .unwrap()
        .as_object()
        .unwrap()
        .clone();
    let mut merged = 0;

    for row in region_rows(&rows) {
        if FEATURE_KEYS.contains(&row.key.as_str()) {
            continue;
        }
        let value = non_default_value(&row.key, &row.option_type, &row.default_serialized);
        let metadata = metadata_value(&row.option_type, &value);
        assert!(object
            .deserialize_known_field(&row.key, &metadata)
            .unwrap());
        expected_values.insert(row.key.clone(), value);
        merged += 1;
    }

    assert_eq!(merged, 143);
    let expected: ProcessRegionSourceOptions =
        serde_json::from_value(Value::Object(expected_values)).unwrap();
    let volume = RegionOptionOverrides::default();
    let actual = resolve_region(
        RegionOverrideSources {
            base: RegionBase::ModelPart {
                process: &process,
                object: Some(&object),
                layer_range: None,
            },
            volume: &volume,
            material: None,
        },
        20,
    );

    types::assert_concrete_types_and_identity(&actual, &expected);
}

#[test]
fn every_feature_process_positive_seed_blocks_an_object_fallback() {
    for feature in FEATURE_KEYS {
        let actual = resolve_feature(
            feature,
            2,
            FeatureScopes {
                object: &[("extruder", "3")],
                volume: &[],
                material: &[],
                layer_range: &[],
            },
        );
        assert_eq!(feature_value(&actual, feature), OrcaInt(2), "{feature}");
    }
}

#[test]
fn every_feature_positive_scope_value_is_explicit_before_same_scope_fallback() {
    for feature in FEATURE_KEYS {
        let actual = resolve_feature(feature, 2, FeatureScopes {
            object: &[(feature, "4"), ("extruder", "5")],
            volume: &[], material: &[], layer_range: &[],
        });
        assert_eq!(feature_value(&actual, feature), OrcaInt(4), "{feature}");
    }
}

#[test]
fn every_feature_zero_and_negative_clear_without_assigning() {
    for feature in FEATURE_KEYS {
        for cleared in ["0", "-1"] {
            let actual = resolve_feature(feature, 2, FeatureScopes {
                object: &[(feature, cleared)], volume: &[], material: &[], layer_range: &[],
            });
            assert_eq!(feature_value(&actual, feature), OrcaInt(2), "{feature}");
        }
    }
}

#[test]
fn every_feature_same_scope_fallback_fills_a_cleared_mask() {
    for feature in FEATURE_KEYS {
        let actual = resolve_feature(feature, 2, FeatureScopes {
            object: &[(feature, "0"), ("extruder", "5")],
            volume: &[], material: &[], layer_range: &[],
        });
        assert_eq!(feature_value(&actual, feature), OrcaInt(5), "{feature}");
    }
}

#[test]
fn every_feature_clear_without_fallback_allows_later_fallback_replacement() {
    for feature in FEATURE_KEYS {
        let actual = resolve_feature(feature, 2, FeatureScopes {
            object: &[(feature, "0")],
            volume: &[("extruder", "3")],
            material: &[("extruder", "4")],
            layer_range: &[],
        });
        assert_eq!(feature_value(&actual, feature), OrcaInt(4), "{feature}");
    }
}

#[test]
fn every_feature_follows_object_volume_material_layer_order() {
    for feature in FEATURE_KEYS {
        let actual = resolve_feature(feature, 1, FeatureScopes {
            object: &[(feature, "2")],
            volume: &[(feature, "3")],
            material: &[(feature, "4")],
            layer_range: &[(feature, "5")],
        });
        assert_eq!(feature_value(&actual, feature), OrcaInt(5), "{feature}");
    }
}

fn resolve_bottom_shell(
    object: Option<i32>,
    volume: Option<i32>,
    material: Option<i32>,
    layer_range: Option<i32>,
) -> OrcaInt {
    let process: ProcessRegionSourceOptions =
        serde_json::from_value(json!({"bottom_shell_layers": "10"})).unwrap();
    let object = object.map(|value| sparse(&[("bottom_shell_layers", value.to_string())]));
    let volume = sparse_optional("bottom_shell_layers", volume);
    let material = material.map(|value| sparse(&[("bottom_shell_layers", value.to_string())]));
    let layer_range =
        layer_range.map(|value| sparse(&[("bottom_shell_layers", value.to_string())]));

    resolve_region(RegionOverrideSources {
        base: RegionBase::ModelPart {
            process: &process,
            object: object.as_ref(),
            layer_range: layer_range.as_ref(),
        },
        volume: &volume,
        material: material.as_ref(),
    }, 10)
    .bottom_shell_layers
}

fn resolve_feature(
    feature: &str,
    process_value: i32,
    scopes: FeatureScopes<'_>,
) -> RegionOptions {
    let process: ProcessRegionSourceOptions = serde_json::from_value(Value::Object(Map::from_iter([
        (feature.to_owned(), json!(process_value.to_string())),
    ])))
    .unwrap();
    let has_object = !scopes.object.is_empty();
    let has_material = !scopes.material.is_empty();
    let has_layer_range = !scopes.layer_range.is_empty();
    let object = sparse_borrowed(scopes.object);
    let volume = sparse_borrowed(scopes.volume);
    let material = sparse_borrowed(scopes.material);
    let layer_range = sparse_borrowed(scopes.layer_range);

    resolve_region(RegionOverrideSources {
        base: RegionBase::ModelPart {
            process: &process,
            object: has_object.then_some(&object),
            layer_range: has_layer_range.then_some(&layer_range),
        },
        volume: &volume,
        material: has_material.then_some(&material),
    }, 10)
}

fn sparse_optional(key: &str, value: Option<i32>) -> RegionOptionOverrides {
    value.map_or_else(RegionOptionOverrides::default, |value| {
        sparse(&[(key, value.to_string())])
    })
}

fn sparse(entries: &[(&str, String)]) -> RegionOptionOverrides {
    let mut overrides = RegionOptionOverrides::default();
    for (key, value) in entries {
        assert!(overrides.deserialize_known_field(key, value).unwrap());
    }
    overrides
}

fn sparse_borrowed(entries: &[(&str, &str)]) -> RegionOptionOverrides {
    let mut overrides = RegionOptionOverrides::default();
    for &(key, value) in entries {
        assert!(overrides.deserialize_known_field(key, value).unwrap());
    }
    overrides
}

fn metadata_value(option_type: &str, value: &Value) -> String {
    match option_type {
        "coInts" => "7,8".to_owned(),
        "coStrings" => r#""first";"second""#.to_owned(),
        _ => value.as_str().unwrap().to_owned(),
    }
}

fn feature_value(options: &RegionOptions, key: &str) -> OrcaInt {
    match key {
        "sparse_infill_filament_id" => options.sparse_infill_filament_id,
        "internal_solid_filament_id" => options.internal_solid_filament_id,
        "top_surface_filament_id" => options.top_surface_filament_id,
        "bottom_surface_filament_id" => options.bottom_surface_filament_id,
        "outer_wall_filament_id" => options.outer_wall_filament_id,
        "inner_wall_filament_id" => options.inner_wall_filament_id,
        _ => unreachable!(),
    }
}
