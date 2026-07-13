use serde_json::{Map, Value};

use crate::options::{
    OrcaFloat, OrcaInt, Percent, ProcessRegionSourceOptions,
    region_options::{RegionBase, RegionOptionOverrides, RegionOverrideSources},
};

use super::{
    RegionOptions, base::non_default_value, inventory, region_rows, resolve_region, types,
};

const FEATURE_KEYS: [&str; 6] = [
    "sparse_infill_filament_id",
    "internal_solid_filament_id",
    "top_surface_filament_id",
    "bottom_surface_filament_id",
    "outer_wall_filament_id",
    "inner_wall_filament_id",
];

#[test]
fn modifier_copies_every_parent_region_field_from_a_non_default_oracle() {
    let rows = inventory();
    let values = region_rows(&rows)
        .into_iter()
        .map(|row| {
            (
                row.key.clone(),
                non_default_value(&row.key, &row.option_type, &row.default_serialized),
            )
        })
        .collect::<Map<_, _>>();
    let source: ProcessRegionSourceOptions =
        serde_json::from_value(Value::Object(values)).unwrap();
    let mut parent = RegionOptions::from_base(&source);
    parent.filament_ironing_flow = Percent(901.0);
    parent.filament_ironing_spacing = OrcaFloat(902.0);
    parent.filament_ironing_inset = OrcaFloat(903.0);
    parent.filament_ironing_speed = OrcaFloat(904.0);
    let volume = RegionOptionOverrides::default();

    let actual = resolve_modifier(&parent, &volume, None);

    types::assert_concrete_types_and_identity(&actual, &source);
    assert_eq!(actual.filament_ironing_flow, parent.ironing_flow);
    assert_eq!(actual.filament_ironing_spacing, parent.ironing_spacing);
    assert_eq!(actual.filament_ironing_inset, parent.ironing_inset);
    assert_eq!(actual.filament_ironing_speed, parent.ironing_speed);
    assert_ne!(actual.filament_ironing_flow, parent.filament_ironing_flow);
    assert_ne!(
        actual.filament_ironing_spacing,
        parent.filament_ironing_spacing
    );
    assert_ne!(actual.filament_ironing_inset, parent.filament_ironing_inset);
    assert_ne!(actual.filament_ironing_speed, parent.filament_ironing_speed);
}

#[test]
fn modifier_volume_and_material_replace_ordinary_fields_in_order() {
    let parent = RegionOptions::from_base(&ProcessRegionSourceOptions::default());
    let volume = sparse(&[("bottom_shell_layers", "7")]);
    let material = sparse(&[("bottom_shell_layers", "11")]);

    let after_volume = resolve_modifier(&parent, &volume, None);
    let after_material = resolve_modifier(&parent, &volume, Some(&material));

    assert_eq!(after_volume.bottom_shell_layers, OrcaInt(7));
    assert_eq!(after_material.bottom_shell_layers, OrcaInt(11));
}

#[test]
fn every_ordinary_modifier_field_applies_volume_then_material() {
    let rows = inventory();
    let mut volume = RegionOptionOverrides::default();
    let mut material = RegionOptionOverrides::default();
    let source: ProcessRegionSourceOptions = serde_json::from_value(Value::Object(
        FEATURE_KEYS
            .into_iter()
            .map(|feature| (feature.to_owned(), Value::String("1".to_owned())))
            .collect(),
    ))
    .unwrap();
    let mut volume_values = serde_json::to_value(&source)
        .unwrap()
        .as_object()
        .unwrap()
        .clone();
    let mut applied = 0;

    for row in region_rows(&rows) {
        if FEATURE_KEYS.contains(&row.key.as_str()) {
            continue;
        }
        let non_default = non_default_value(&row.key, &row.option_type, &row.default_serialized);
        assert!(volume
            .deserialize_known_field(&row.key, &metadata_value(&row.option_type, &non_default))
            .unwrap());
        assert!(material
            .deserialize_known_field(&row.key, &row.default_serialized)
            .unwrap());
        volume_values.insert(row.key.clone(), non_default);
        applied += 1;
    }

    assert_eq!(applied, 143);
    let parent = RegionOptions::from_base(&source);
    let volume_expected: ProcessRegionSourceOptions =
        serde_json::from_value(Value::Object(volume_values)).unwrap();
    let after_volume = resolve_modifier(&parent, &volume, None);
    types::assert_concrete_types_and_identity(&after_volume, &volume_expected);

    let after_material = resolve_modifier(&parent, &volume, Some(&material));
    let material_expected = source;
    types::assert_concrete_types_and_identity(&after_material, &material_expected);
}

#[test]
fn modifier_mask_starts_clear_for_every_positive_parent_feature() {
    let source: ProcessRegionSourceOptions = serde_json::from_value(Value::Object(
        FEATURE_KEYS
            .into_iter()
            .map(|feature| (feature.to_owned(), Value::String("2".to_owned())))
            .collect(),
    ))
    .unwrap();
    let parent = RegionOptions::from_base(&source);
    let volume = sparse(&[("extruder", "3")]);
    let actual = resolve_modifier(&parent, &volume, None);

    for feature in FEATURE_KEYS {
        assert_eq!(feature_value(&actual, feature), OrcaInt(3), "{feature}");
    }
}

#[test]
fn modifier_positive_feature_blocks_same_and_later_extruder_fallbacks() {
    for feature in FEATURE_KEYS {
        let parent = parent_with_feature(feature, 2);
        let volume = sparse(&[(feature, "4"), ("extruder", "5")]);
        let material = sparse(&[("extruder", "6")]);
        let actual = resolve_modifier(&parent, &volume, Some(&material));

        assert_eq!(feature_value(&actual, feature), OrcaInt(4), "{feature}");
    }
}

#[test]
fn modifier_nonpositive_feature_clears_without_assigning_then_allows_fallback() {
    for feature in FEATURE_KEYS {
        for cleared in ["0", "-1"] {
            let parent = parent_with_feature(feature, 2);
            let volume = sparse(&[(feature, cleared)]);
            let inherited = resolve_modifier(&parent, &volume, None);
            assert_eq!(feature_value(&inherited, feature), OrcaInt(2), "{feature}");

            let material = sparse(&[("extruder", "6")]);
            let replaced = resolve_modifier(&parent, &volume, Some(&material));
            assert_eq!(feature_value(&replaced, feature), OrcaInt(6), "{feature}");
        }
    }
}

#[test]
fn modifier_same_scope_extruder_fills_every_cleared_feature() {
    for feature in FEATURE_KEYS {
        let parent = parent_with_feature(feature, 2);
        let volume = sparse(&[(feature, "0"), ("extruder", "5")]);
        let actual = resolve_modifier(&parent, &volume, None);

        assert_eq!(feature_value(&actual, feature), OrcaInt(5), "{feature}");
    }
}

#[test]
fn modifier_material_feature_and_extruder_follow_volume() {
    for feature in FEATURE_KEYS {
        let parent = parent_with_feature(feature, 2);
        let volume = sparse(&[(feature, "3")]);
        let material = sparse(&[(feature, "0"), ("extruder", "5")]);
        let actual = resolve_modifier(&parent, &volume, Some(&material));

        assert_eq!(feature_value(&actual, feature), OrcaInt(5), "{feature}");
    }
}

fn resolve_modifier(
    parent: &RegionOptions,
    volume: &RegionOptionOverrides,
    material: Option<&RegionOptionOverrides>,
) -> RegionOptions {
    resolve_region(
        RegionOverrideSources {
            base: RegionBase::Modifier { parent },
            volume,
            material,
        },
        20,
    )
}

fn parent_with_feature(feature: &str, value: i32) -> RegionOptions {
    let source: ProcessRegionSourceOptions = serde_json::from_value(Value::Object(Map::from_iter([
        (feature.to_owned(), Value::String(value.to_string())),
    ])))
    .unwrap();
    RegionOptions::from_base(&source)
}

fn sparse(entries: &[(&str, &str)]) -> RegionOptionOverrides {
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
