use crate::options::{
    OrcaFloat, OrcaInt, Percent, ProcessFuzzySkinType, ProcessRegionSourceOptions,
    region_options::{RegionBase, RegionOptionOverrides, RegionOverrideSources},
};

use super::{RegionOptions, resolve_region};

const FEATURE_KEYS: [&str; 6] = [
    "sparse_infill_filament_id",
    "internal_solid_filament_id",
    "top_surface_filament_id",
    "bottom_surface_filament_id",
    "outer_wall_filament_id",
    "inner_wall_filament_id",
];

const FUZZY_VARIANTS: [ProcessFuzzySkinType; 5] = [
    ProcessFuzzySkinType::External,
    ProcessFuzzySkinType::Hole,
    ProcessFuzzySkinType::All,
    ProcessFuzzySkinType::AllWalls,
    ProcessFuzzySkinType::Disabled,
];

struct ModelPartScopes<'a> {
    object: Option<&'a RegionOptionOverrides>,
    volume: &'a RegionOptionOverrides,
    material: Option<&'a RegionOptionOverrides>,
    layer_range: Option<&'a RegionOptionOverrides>,
}

#[test]
fn every_feature_id_clamps_out_of_range_and_preserves_the_active_range() {
    for feature in FEATURE_KEYS {
        for value in [0, -1, 4] {
            let actual = resolve_process_feature(feature, value, 3);
            assert_eq!(feature_value(&actual, feature), OrcaInt(1), "{feature}={value}");
        }
        for value in 1..=3 {
            let actual = resolve_process_feature(feature, value, 3);
            assert_eq!(
                feature_value(&actual, feature),
                OrcaInt(value),
                "{feature}={value}"
            );
        }
    }
}

#[test]
fn sparse_density_uses_raw_percent_numbers_and_strict_bounds() {
    let source_threshold = f64::from(0.00011_f32);
    for (input, expected) in [
        (-1.0, 0.0),
        (0.0, 0.0),
        (0.00010, 0.0),
        (0.00011, 0.0),
        (source_threshold, source_threshold),
        (
            f64::from_bits(source_threshold.to_bits() + 1),
            f64::from_bits(source_threshold.to_bits() + 1),
        ),
        (35.0, 35.0),
        (100.0, 100.0),
        (100.1, 100.0),
    ] {
        let mut process = ProcessRegionSourceOptions {
            sparse_infill_density: Percent(input),
            ..ProcessRegionSourceOptions::default()
        };
        set_all_features(&mut process, 1);

        let actual = resolve_model_part(
            &process,
            ModelPartScopes {
                object: None,
                volume: &default_overrides(),
                material: None,
                layer_range: None,
            },
            1,
        );

        assert_eq!(actual.sparse_infill_density, Percent(expected), "{input}");
    }
}

#[test]
fn every_non_none_fuzzy_variant_uses_both_strict_guards() {
    for variant in FUZZY_VARIANTS {
        assert_eq!(
            resolve_fuzzy(variant, 0.009, 0.001).fuzzy_skin,
            ProcessFuzzySkinType::None,
            "distance guard for {variant:?}"
        );
        assert_eq!(
            resolve_fuzzy(variant, 0.01, 0.0009).fuzzy_skin,
            ProcessFuzzySkinType::None,
            "thickness guard for {variant:?}"
        );
        assert_eq!(
            resolve_fuzzy(variant, 0.01, 0.001).fuzzy_skin,
            variant,
            "threshold equality for {variant:?}"
        );
        assert_eq!(
            resolve_fuzzy(variant, 0.011, 0.002).fuzzy_skin,
            variant,
            "values above both thresholds for {variant:?}"
        );
    }
}

#[test]
fn fuzzy_none_stays_none_below_both_thresholds() {
    assert_eq!(
        resolve_fuzzy(ProcessFuzzySkinType::None, 0.0, 0.0).fuzzy_skin,
        ProcessFuzzySkinType::None
    );
}

#[test]
fn model_part_normalizes_only_after_the_final_layer_overlay() {
    let mut process = ProcessRegionSourceOptions::default();
    set_all_features(&mut process, 1);
    let object = sparse(&[
        ("top_surface_filament_id", "2"),
        ("sparse_infill_density", "25%"),
        ("fuzzy_skin", "external"),
        ("fuzzy_skin_point_distance", "0.02"),
        ("fuzzy_skin_thickness", "0.002"),
    ]);
    let volume = sparse(&[
        ("top_surface_filament_id", "3"),
        ("sparse_infill_density", "50%"),
    ]);
    let material = sparse(&[
        ("top_surface_filament_id", "2"),
        ("sparse_infill_density", "75%"),
    ]);
    let layer_range = sparse(&[
        ("top_surface_filament_id", "4"),
        ("sparse_infill_density", "101%"),
        ("fuzzy_skin", "all"),
        ("fuzzy_skin_point_distance", "0.009"),
    ]);

    let actual = resolve_model_part(
        &process,
        ModelPartScopes {
            object: Some(&object),
            volume: &volume,
            material: Some(&material),
            layer_range: Some(&layer_range),
        },
        3,
    );

    assert_eq!(actual.top_surface_filament_id, OrcaInt(1));
    assert_eq!(actual.sparse_infill_density, Percent(100.0));
    assert_eq!(actual.fuzzy_skin, ProcessFuzzySkinType::None);
}

#[test]
fn model_part_later_layer_repair_prevents_early_irreversible_fuzzy_disable() {
    let mut process = ProcessRegionSourceOptions::default();
    set_all_features(&mut process, 1);
    let object = sparse(&[
        ("fuzzy_skin", "external"),
        ("fuzzy_skin_point_distance", "0.009"),
        ("fuzzy_skin_thickness", "0.002"),
    ]);
    let volume = default_overrides();
    let layer_range = sparse(&[
        ("fuzzy_skin_point_distance", "0.01"),
        ("fuzzy_skin_thickness", "0.001"),
    ]);

    let actual = resolve_model_part(
        &process,
        ModelPartScopes {
            object: Some(&object),
            volume: &volume,
            material: None,
            layer_range: Some(&layer_range),
        },
        1,
    );

    assert_eq!(actual.fuzzy_skin, ProcessFuzzySkinType::External);
}

#[test]
fn modifier_normalizes_only_after_the_final_material_overlay() {
    let mut parent_source = ProcessRegionSourceOptions::default();
    set_all_features(&mut parent_source, 1);
    let parent = RegionOptions::from_base(&parent_source);
    let volume = sparse(&[
        ("inner_wall_filament_id", "2"),
        ("sparse_infill_density", "50%"),
        ("fuzzy_skin", "hole"),
        ("fuzzy_skin_point_distance", "0.02"),
        ("fuzzy_skin_thickness", "0.002"),
    ]);
    let material = sparse(&[
        ("inner_wall_filament_id", "4"),
        ("sparse_infill_density", "0.00010%"),
        ("fuzzy_skin", "disabled_fuzzy"),
        ("fuzzy_skin_thickness", "0.0009"),
    ]);

    let actual = resolve_region(
        RegionOverrideSources {
            base: RegionBase::Modifier { parent: &parent },
            volume: &volume,
            material: Some(&material),
        },
        3,
    );

    assert_eq!(actual.inner_wall_filament_id, OrcaInt(1));
    assert_eq!(actual.sparse_infill_density, Percent(0.0));
    assert_eq!(actual.fuzzy_skin, ProcessFuzzySkinType::None);
}

#[test]
fn modifier_material_repair_prevents_early_irreversible_fuzzy_disable() {
    let mut parent_source = ProcessRegionSourceOptions::default();
    set_all_features(&mut parent_source, 1);
    let parent = RegionOptions::from_base(&parent_source);
    let volume = sparse(&[
        ("fuzzy_skin", "hole"),
        ("fuzzy_skin_point_distance", "0.009"),
        ("fuzzy_skin_thickness", "0.002"),
    ]);
    let material = sparse(&[
        ("fuzzy_skin_point_distance", "0.01"),
        ("fuzzy_skin_thickness", "0.001"),
    ]);

    let actual = resolve_region(
        RegionOverrideSources {
            base: RegionBase::Modifier { parent: &parent },
            volume: &volume,
            material: Some(&material),
        },
        1,
    );

    assert_eq!(actual.fuzzy_skin, ProcessFuzzySkinType::Hole);
}

fn resolve_process_feature(feature: &str, value: i32, num_extruders: usize) -> RegionOptions {
    let mut process = ProcessRegionSourceOptions::default();
    set_feature(&mut process, feature, value);
    resolve_region(
        RegionOverrideSources {
            base: RegionBase::ModelPart {
                process: &process,
                object: None,
                layer_range: None,
            },
            volume: &default_overrides(),
            material: None,
        },
        num_extruders,
    )
}

fn resolve_fuzzy(
    fuzzy_skin: ProcessFuzzySkinType,
    point_distance: f64,
    thickness: f64,
) -> RegionOptions {
    let mut process = ProcessRegionSourceOptions {
        fuzzy_skin,
        fuzzy_skin_point_distance: OrcaFloat(point_distance),
        fuzzy_skin_thickness: OrcaFloat(thickness),
        ..ProcessRegionSourceOptions::default()
    };
    set_all_features(&mut process, 1);
    resolve_model_part(
        &process,
        ModelPartScopes {
            object: None,
            volume: &default_overrides(),
            material: None,
            layer_range: None,
        },
        1,
    )
}

fn resolve_model_part(
    process: &ProcessRegionSourceOptions,
    scopes: ModelPartScopes<'_>,
    num_extruders: usize,
) -> RegionOptions {
    resolve_region(
        RegionOverrideSources {
            base: RegionBase::ModelPart {
                process,
                object: scopes.object,
                layer_range: scopes.layer_range,
            },
            volume: scopes.volume,
            material: scopes.material,
        },
        num_extruders,
    )
}

fn sparse(entries: &[(&str, &str)]) -> RegionOptionOverrides {
    let mut overrides = RegionOptionOverrides::default();
    for &(key, value) in entries {
        assert!(overrides.deserialize_known_field(key, value).unwrap());
    }
    overrides
}

fn default_overrides() -> RegionOptionOverrides {
    RegionOptionOverrides::default()
}

fn set_all_features(options: &mut ProcessRegionSourceOptions, value: i32) {
    for feature in FEATURE_KEYS {
        set_feature(options, feature, value);
    }
}

fn set_feature(options: &mut ProcessRegionSourceOptions, feature: &str, value: i32) {
    let value = OrcaInt(value);
    match feature {
        "sparse_infill_filament_id" => options.sparse_infill_filament_id = value,
        "internal_solid_filament_id" => options.internal_solid_filament_id = value,
        "top_surface_filament_id" => options.top_surface_filament_id = value,
        "bottom_surface_filament_id" => options.bottom_surface_filament_id = value,
        "outer_wall_filament_id" => options.outer_wall_filament_id = value,
        "inner_wall_filament_id" => options.inner_wall_filament_id = value,
        _ => unreachable!(),
    }
}

fn feature_value(options: &RegionOptions, feature: &str) -> OrcaInt {
    match feature {
        "sparse_infill_filament_id" => options.sparse_infill_filament_id,
        "internal_solid_filament_id" => options.internal_solid_filament_id,
        "top_surface_filament_id" => options.top_surface_filament_id,
        "bottom_surface_filament_id" => options.bottom_surface_filament_id,
        "outer_wall_filament_id" => options.outer_wall_filament_id,
        "inner_wall_filament_id" => options.inner_wall_filament_id,
        _ => unreachable!(),
    }
}
