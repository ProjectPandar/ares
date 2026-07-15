use crate::options::{
    ObjectOptionOverrides, ObjectOptions, RegionOptionOverrides, RegionOptions,
    deserialize_object_model_field, deserialize_region_model_field,
    registry::{OptionDefinition, OptionValueKind, option_definition},
};

#[test]
fn every_object_and_region_owner_has_exact_scope_without_overlap() {
    assert_eq!(ObjectOptions::DECLARATION_ORDER.len(), 126);
    assert_eq!(RegionOptions::PROCESS_DECLARATION_ORDER.len(), 149);

    for key in ObjectOptions::DECLARATION_ORDER {
        let mut object = ObjectOptionOverrides::default();
        let mut region = RegionOptionOverrides::default();
        deserialize_object_model_field(
            key.to_owned(),
            metadata_default(option_definition(key).unwrap()),
            &mut object,
            &mut region,
        )
        .unwrap();
        assert_ne!(object, ObjectOptionOverrides::default(), "{key}");
        assert!(region.present_keys().is_empty(), "object key leaked to region: {key}");

        let mut part = RegionOptionOverrides::default();
        deserialize_region_model_field(
            key.to_owned(),
            metadata_default(option_definition(key).unwrap()),
            &mut part,
        )
        .unwrap();
        assert!(part.present_keys().is_empty(), "object key leaked to part: {key}");

        let mut layer = RegionOptionOverrides::default();
        deserialize_region_model_field(
            key.to_owned(),
            metadata_default(option_definition(key).unwrap()),
            &mut layer,
        )
        .unwrap();
        assert!(layer.present_keys().is_empty(), "object key leaked to layer: {key}");
    }

    for key in RegionOptions::PROCESS_DECLARATION_ORDER {
        let mut object = ObjectOptionOverrides::default();
        let mut region = RegionOptionOverrides::default();
        deserialize_object_model_field(
            key.to_owned(),
            metadata_default(option_definition(key).unwrap()),
            &mut object,
            &mut region,
        )
        .unwrap();
        assert_eq!(object, ObjectOptionOverrides::default(), "{key}");
        assert_eq!(region.present_keys(), [key], "{key}");

        let mut part = RegionOptionOverrides::default();
        deserialize_region_model_field(
            key.to_owned(),
            metadata_default(option_definition(key).unwrap()),
            &mut part,
        )
        .unwrap();
        assert_eq!(part.present_keys(), [key], "{key}");

        let mut layer = RegionOptionOverrides::default();
        deserialize_region_model_field(
            key.to_owned(),
            metadata_default(option_definition(key).unwrap()),
            &mut layer,
        )
        .unwrap();
        assert_eq!(layer.present_keys(), [key], "{key}");
    }

    let mut object = ObjectOptionOverrides::default();
    let mut region = RegionOptionOverrides::default();
    deserialize_object_model_field(
        "extruder".to_owned(),
        "0".to_owned(),
        &mut object,
        &mut region,
    )
    .unwrap();
    assert_eq!(object, ObjectOptionOverrides::default());
    assert_eq!(region.present_keys(), ["extruder"]);
    for mut region in [RegionOptionOverrides::default(), RegionOptionOverrides::default()] {
        deserialize_region_model_field("extruder".to_owned(), "0".to_owned(), &mut region)
            .unwrap();
        assert_eq!(region.present_keys(), ["extruder"]);
    }
}

#[test]
fn owner_assignments_are_last_write_wins_at_object_part_and_layer_scopes() {
    let mut object = ObjectOptionOverrides::default();
    let mut object_region = RegionOptionOverrides::default();
    for value in ["2.5", "7.25"] {
        deserialize_object_model_field(
            "brim_width".to_owned(),
            value.to_owned(),
            &mut object,
            &mut object_region,
        )
        .unwrap();
    }
    for value in ["31%", "47%"] {
        deserialize_object_model_field(
            "sparse_infill_density".to_owned(),
            value.to_owned(),
            &mut object,
            &mut object_region,
        )
        .unwrap();
    }
    assert_eq!(object.brim_width, Some(crate::OrcaFloat(7.25)));
    assert_eq!(object_region.sparse_infill_density, Some(crate::Percent(47.0)));

    let mut part = RegionOptionOverrides::default();
    let mut layer = RegionOptionOverrides::default();
    for value in ["3", "5"] {
        deserialize_region_model_field("wall_loops".to_owned(), value.to_owned(), &mut part)
            .unwrap();
        deserialize_region_model_field("wall_loops".to_owned(), value.to_owned(), &mut layer)
            .unwrap();
    }
    assert_eq!(part.wall_loops, Some(crate::OrcaInt(5)));
    assert_eq!(layer.wall_loops, Some(crate::OrcaInt(5)));
}

fn metadata_default(definition: &OptionDefinition) -> String {
    match definition.kind {
        OptionValueKind::Bool | OptionValueKind::Bools | OptionValueKind::BoolsNullable => {
            definition
                .default_value
                .split(',')
                .map(|value| match value.trim() {
                    "true" => "1",
                    "false" => "0",
                    other => other,
                })
                .collect::<Vec<_>>()
                .join(",")
        }
        _ => definition.default_value.to_owned(),
    }
}
