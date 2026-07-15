use crate::{
    SliceError,
    options::{
        ObjectOptionOverrides, RegionOptionOverrides, deserialize_object_model_field,
        deserialize_region_model_field,
    },
};

#[test]
fn five_action_order_assigns_owners_then_discards_typed_and_registry_only_values() {
    let mut object = ObjectOptionOverrides::default();
    let mut region = RegionOptionOverrides::default();

    deserialize_object_model_field(
        "support_material_extruder".to_owned(),
        "3".to_owned(),
        &mut object,
        &mut region,
    )
    .unwrap();
    deserialize_object_model_field(
        "sparse_infill_density".to_owned(),
        "42%".to_owned(),
        &mut object,
        &mut region,
    )
    .unwrap();
    assert_eq!(object.support_filament, Some(crate::OrcaInt(3)));
    assert_eq!(region.sparse_infill_density, Some(crate::Percent(42.0)));

    for (key, value) in [("gcode_flavor", "marlin"), ("display_orientation", "portrait")] {
        deserialize_object_model_field(
            key.to_owned(),
            value.to_owned(),
            &mut object,
            &mut region,
        )
        .unwrap();
        let mut part = RegionOptionOverrides::default();
        deserialize_region_model_field(key.to_owned(), value.to_owned(), &mut part).unwrap();
        assert!(part.present_keys().is_empty());
        let mut layer = RegionOptionOverrides::default();
        deserialize_region_model_field(key.to_owned(), value.to_owned(), &mut layer).unwrap();
        assert!(layer.present_keys().is_empty());
    }
}

#[test]
fn model_path_completes_all_four_profile_bookkeeping_rules_without_storage() {
    for (source, value) in [
        ("inherits_cummulative", "base"),
        ("compatible_printers_condition_cummulative", "printer"),
        ("compatible_prints_condition_cummulative", "process"),
        ("different_settings_to_system", "brim_width"),
    ] {
        let mut object = ObjectOptionOverrides::default();
        let mut region = RegionOptionOverrides::default();
        deserialize_object_model_field(
            source.to_owned(),
            value.to_owned(),
            &mut object,
            &mut region,
        )
        .unwrap();
        assert_eq!(object, ObjectOptionOverrides::default());
        assert!(region.present_keys().is_empty());

        let mut part = RegionOptionOverrides::default();
        deserialize_region_model_field(source.to_owned(), value.to_owned(), &mut part).unwrap();
        assert!(part.present_keys().is_empty());
        let mut layer = RegionOptionOverrides::default();
        deserialize_region_model_field(source.to_owned(), value.to_owned(), &mut layer).unwrap();
        assert!(layer.present_keys().is_empty());
    }
}

#[test]
fn profile_bookkeeping_values_are_concretely_validated_with_the_source_name() {
    for source in [
        "inherits_cummulative",
        "compatible_printers_condition_cummulative",
        "compatible_prints_condition_cummulative",
        "different_settings_to_system",
    ] {
        let mut object = ObjectOptionOverrides::default();
        let mut region = RegionOptionOverrides::default();
        let error = deserialize_object_model_field(
            source.to_owned(),
            "\"unterminated".to_owned(),
            &mut object,
            &mut region,
        )
        .unwrap_err();
        let SliceError::InvalidInput(message) = error else {
            panic!("unexpected error: {error:?}");
        };
        assert!(message.contains(source), "{message}");
        assert!(message.len() <= 512, "{message}");
    }
}

#[test]
fn unknown_and_unported_alias_errors_are_bounded_and_name_the_original_source() {
    for source in ["unknown_model_option", "perimeter_feed_rate"] {
        let mut object = ObjectOptionOverrides::default();
        let mut region = RegionOptionOverrides::default();
        let error = deserialize_object_model_field(
            source.to_owned(),
            "1".to_owned(),
            &mut object,
            &mut region,
        )
        .unwrap_err();
        let SliceError::InvalidInput(message) = error else {
            panic!("unexpected error: {error:?}");
        };
        assert!(message.contains(source), "{message}");
        assert!(message.len() <= 512, "{message}");
    }

    let source = format!("unknown_{}", "x".repeat(10_000));
    let mut part = RegionOptionOverrides::default();
    let error =
        deserialize_region_model_field(source.clone(), "1".to_owned(), &mut part).unwrap_err();
    let SliceError::InvalidInput(message) = error else {
        panic!("unexpected error: {error:?}");
    };
    assert!(message.contains("unknown_"), "{message}");
    assert!(!message.contains(&source), "{message}");
    assert!(message.len() <= 512, "{message}");
}

#[test]
fn canonical_owner_errors_bound_untrusted_values() {
    let mut object = ObjectOptionOverrides::default();
    let mut region = RegionOptionOverrides::default();
    let value = "x".repeat(10_000);
    let error = deserialize_object_model_field(
        "brim_type".to_owned(),
        value.clone(),
        &mut object,
        &mut region,
    )
    .unwrap_err();
    let SliceError::InvalidInput(message) = error else {
        panic!("unexpected error: {error:?}");
    };
    assert!(message.contains("brim_type"), "{message}");
    assert!(!message.contains(&value), "{message}");
    assert!(message.len() <= 512, "{message}");
}
