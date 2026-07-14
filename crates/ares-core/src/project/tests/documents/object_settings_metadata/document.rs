use crate::options::ObjectOptionOverrides;
use crate::project::{
    ArchiveLimits, ProjectArchive,
    model_settings::ModelSettings,
    xml::{XmlRole, deserialize_xml},
};
use crate::{OrcaFloat, SliceError};

use super::super::{FIXTURE, read};
use super::{
    object_module, object_name, object_overrides, pairs, parse_settings, region_overrides,
    retained_config,
};

#[test]
fn object_settings_metadata_order_is_last_write_wins_for_names_and_typed_option() {
    let settings = parse_settings(
        r#"<config><object id="2">
        <metadata key="name" value="first"/>
        <metadata key="module" value="module-a"/>
        <metadata key="layer_height" value="0.10"/>
        <metadata key="name" value="second"/>
        <metadata key="module" value="module-b"/>
        <metadata key="layer_height" value="0.30"/>
        </object></config>"#,
    )
    .unwrap();
    let object = &settings.objects[0];

    assert_eq!(object_name(object), "second");
    assert_eq!(object_module(object), "module-b");
    assert_eq!(object_overrides(object).layer_height, Some(OrcaFloat(0.30)));
    assert!(retained_config(object).is_empty());
}

#[test]
fn object_settings_metadata_missing_and_explicit_empty_names_use_source_default_empty_strings() {
    let settings = parse_settings(
        r#"<config>
        <object id="1"/>
        <object id="2">
        <metadata key="name" value=""/>
        <metadata key="module" value=""/>
        </object></config>"#,
    )
    .unwrap();

    assert_eq!(settings.objects.len(), 2);
    assert_eq!(
        settings
            .objects
            .iter()
            .map(|object| object.id)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    for id in [1, 2] {
        let object = settings
            .objects
            .iter()
            .find(|object| object.id == id)
            .unwrap();
        assert_eq!(object_name(object), "");
        assert_eq!(object_module(object), "");
    }
    assert!(!settings.objects.iter().any(|object| object.id == 3));
}

#[test]
fn object_settings_metadata_duplicate_empty_name_and_module_are_ordered_last_write_wins() {
    let settings = parse_settings(
        r#"<config>
        <object id="10">
        <metadata key="name" value="named-before-empty"/>
        <metadata key="name" value=""/>
        <metadata key="module" value="module-before-empty"/>
        <metadata key="module" value=""/>
        </object>
        <object id="11">
        <metadata key="name" value=""/>
        <metadata key="name" value="named-after-empty"/>
        <metadata key="module" value=""/>
        <metadata key="module" value="module-after-empty"/>
        </object></config>"#,
    )
    .unwrap();

    assert_eq!(settings.objects.len(), 2);
    assert_eq!(
        settings
            .objects
            .iter()
            .map(|object| object.id)
            .collect::<Vec<_>>(),
        [10, 11]
    );
    let empty_final = &settings.objects[0];
    assert_eq!(object_name(empty_final), "");
    assert_eq!(object_module(empty_final), "");
    let nonempty_final = &settings.objects[1];
    assert_eq!(object_name(nonempty_final), "named-after-empty");
    assert_eq!(object_module(nonempty_final), "module-after-empty");
}

#[test]
fn object_settings_metadata_later_malformed_duplicate_fails_at_that_entry() {
    let error = parse_settings(
        r#"<config><object id="2">
        <metadata key="layer_height" value="0.10"/>
        <metadata key="layer_height" value="not-float"/>
        </object></config>"#,
    )
    .unwrap_err();
    let SliceError::InvalidInput(message) = error else {
        panic!("unexpected error: {error}");
    };
    assert!(
        message.contains("invalid project model settings XML:"),
        "{message}"
    );
    assert!(
        message.contains("invalid Orca object option layer_height:"),
        "{message}"
    );
    assert!(message.contains("invalid float literal"), "{message}");
}

#[test]
fn object_settings_metadata_retains_exact_non_126_sequence_with_duplicates() {
    let settings = parse_settings(
        r#"<config><object id="2">
        <metadata key="extruder" value="1"/>
        <metadata key="support_material_extruder" value="2"/>
        <metadata key="brim_width" value="123.456789"/>
        <metadata key="sparse_infill_density" value="37%"/>
        <metadata key="unregistered_future_key" value="beta"/>
        <metadata key="extruder" value="2"/>
        <metadata key="support_material_extruder" value="3"/>
        </object></config>"#,
    )
    .unwrap();
    let object = &settings.objects[0];
    let overrides = object_overrides(object);

    assert_eq!(overrides.brim_width, Some(OrcaFloat(123.456789)));
    assert_eq!(overrides.support_filament, Some(crate::OrcaInt(3)));
    assert_eq!(
        pairs(retained_config(object)),
        [("unregistered_future_key", "beta")]
    );
    assert_eq!(region_overrides(object).extruder, Some(crate::OrcaInt(2)));
    assert_eq!(
        region_overrides(object).sparse_infill_density,
        Some(crate::Percent(37.0))
    );
}

#[test]
fn object_settings_metadata_keeps_matrix_and_canonical_key_on_nested_part_path() {
    let settings = parse_settings(
        r#"<config><object id="7">
        <metadata key="brim_width" value="123.456789"/>
        <part id="9" subtype="normal_part">
        <metadata key="matrix" value="1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1"/>
        <metadata key="layer_height" value="0.77"/>
        </part></object></config>"#,
    )
    .unwrap();
    let object = &settings.objects[0];
    let overrides = object_overrides(object);

    assert_eq!(overrides.brim_width, Some(OrcaFloat(123.456789)));
    assert_eq!(overrides.layer_height, None);
    assert!(retained_config(object).is_empty());
    assert_eq!(
        pairs(&object.parts[0].retained_metadata),
        [
            ("matrix", "1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1"),
            ("layer_height", "0.77"),
        ]
    );
}

#[test]
fn object_settings_metadata_real_fixture_preserves_named_retained_and_part_boundaries() {
    let mut archive = ProjectArchive::open(FIXTURE, ArchiveLimits::PROJECT).unwrap();
    let settings: ModelSettings = deserialize_xml(
        &read(&mut archive, "Metadata/model_settings.config"),
        XmlRole::ModelSettings,
    )
    .unwrap();
    let object = settings
        .objects
        .iter()
        .find(|object| object.id == 2)
        .unwrap();

    assert_eq!(object_name(object), "ksr_fdmtest_v4.drc");
    assert_eq!(object_module(object), "");
    assert_eq!(object_overrides(object), &ObjectOptionOverrides::default());
    assert!(retained_config(object).is_empty());
    assert_eq!(region_overrides(object).extruder, Some(crate::OrcaInt(1)));
    let part = object.parts.iter().find(|part| part.id == 1).unwrap();
    assert_eq!(
        pairs(&part.retained_metadata),
        [
            ("name", "ksr_fdmtest_v4.drc"),
            ("matrix", "1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1"),
            ("source_file", "ksr_fdmtest_v4.drc"),
            ("source_object_id", "0"),
            ("source_volume_id", "0"),
            ("source_offset_x", "128.5"),
            ("source_offset_y", "128.5"),
            ("source_offset_z", "46"),
        ]
    );
    assert_eq!(
        part.region_overrides,
        crate::options::RegionOptionOverrides::default()
    );
    assert_eq!(part.mesh_stat.as_ref().unwrap().edges_fixed, 0);
}
