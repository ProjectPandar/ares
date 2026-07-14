use crate::{
    OrcaBool, OrcaFloat, OrcaInt, SliceError,
    options::{ProcessSupportStyle, ProcessSupportType, ProcessWallSequence},
};

use super::{pairs, parse_settings};

#[test]
fn object_and_part_legacy_owner_targets_are_typed_directly() {
    let settings = parse_settings(
        r#"<config><object id="2">
        <metadata key="support_material_extruder" value="3"/>
        <metadata key="initial_layer_flow_ratio" value="0.85"/>
        <part id="9" subtype="normal_part">
        <metadata key="wall_filament" value="2"/>
        </part></object></config>"#,
    )
    .unwrap();
    let object = &settings.objects[0];
    let part = &object.parts[0];

    assert_eq!(object.overrides.support_filament, Some(OrcaInt(3)));
    assert_eq!(
        object.region_overrides.bottom_solid_infill_flow_ratio,
        Some(OrcaFloat(0.85))
    );
    assert_eq!(
        part.region_overrides.outer_wall_filament_id,
        Some(OrcaInt(2))
    );
    assert!(object.retained_config.is_empty());
    assert!(part.retained_metadata.is_empty());
}

#[test]
fn non_owner_targets_are_canonicalized_in_their_original_order() {
    let settings = parse_settings(
        r#"<config><object id="2">
        <metadata key="future_a" value="alpha"/>
        <metadata key="enable_wipe_tower" value="1"/>
        <metadata key="future_b" value="beta"/>
        <part id="9" subtype="normal_part">
        <metadata key="future_part_a" value="one"/>
        <metadata key="thumbnail_size" value="96x96"/>
        <metadata key="future_part_b" value="two"/>
        </part></object></config>"#,
    )
    .unwrap();
    let object = &settings.objects[0];

    assert_eq!(
        pairs(&object.retained_config),
        [
            ("future_a", "alpha"),
            ("enable_prime_tower", "1"),
            ("future_b", "beta"),
        ]
    );
    assert_eq!(
        pairs(&object.parts[0].retained_metadata),
        [
            ("future_part_a", "one"),
            ("thumbnails", "96x96"),
            ("future_part_b", "two"),
        ]
    );
}

#[test]
fn canonical_and_legacy_owner_assignments_are_xml_last_write_wins() {
    let settings = parse_settings(
        r#"<config>
        <object id="1">
        <metadata key="support_filament" value="4"/>
        <metadata key="support_material_extruder" value="2"/>
        <part id="11" subtype="normal_part">
        <metadata key="outer_wall_filament_id" value="5"/>
        <metadata key="wall_filament" value="3"/>
        </part></object>
        <object id="2">
        <metadata key="support_material_extruder" value="2"/>
        <metadata key="support_filament" value="4"/>
        <part id="12" subtype="normal_part">
        <metadata key="wall_filament" value="3"/>
        <metadata key="outer_wall_filament_id" value="5"/>
        </part></object>
        </config>"#,
    )
    .unwrap();

    assert_eq!(
        settings.objects[0].overrides.support_filament,
        Some(OrcaInt(2))
    );
    assert_eq!(
        settings.objects[0].parts[0]
            .region_overrides
            .outer_wall_filament_id,
        Some(OrcaInt(3))
    );
    assert_eq!(
        settings.objects[1].overrides.support_filament,
        Some(OrcaInt(4))
    );
    assert_eq!(
        settings.objects[1].parts[0]
            .region_overrides
            .outer_wall_filament_id,
        Some(OrcaInt(5))
    );
}

#[test]
fn feature_filament_inherit_and_obsolete_entries_apply_on_both_xml_paths() {
    let settings = parse_settings(
        r#"<config><object id="2">
        <metadata key="wall_filament" value="1"/>
        <metadata key="outer_wall_speed" value="50%"/>
        <metadata key="acceleration" value="object-obsolete"/>
        <part id="9" subtype="normal_part">
        <metadata key="infill_extruder" value="1"/>
        <metadata key="outer_wall_speed" value="60%"/>
        <metadata key="scale" value="part-obsolete"/>
        </part></object></config>"#,
    )
    .unwrap();
    let object = &settings.objects[0];

    assert_eq!(
        object.region_overrides.outer_wall_filament_id,
        Some(OrcaInt(0))
    );
    assert_eq!(
        object.parts[0].region_overrides.sparse_infill_filament_id,
        Some(OrcaInt(0))
    );
    assert_eq!(object.region_overrides.outer_wall_speed, None);
    assert_eq!(object.parts[0].region_overrides.outer_wall_speed, None);
    assert!(object.retained_config.is_empty());
    assert!(object.parts[0].retained_metadata.is_empty());
}

#[test]
fn structural_and_unclassified_metadata_bypass_legacy_dispatch() {
    let settings = parse_settings(
        r#"<config><object id="2">
        <metadata key="name" value="object-name"/>
        <metadata key="module" value="object-module"/>
        <metadata key="volume_type" value="object-volume-type"/>
        <metadata key="matrix" value="object-matrix"/>
        <metadata key="source_file" value="object-source"/>
        <metadata key="source_in_inches" value="1"/>
        <metadata key="source_future" value="object-future-source"/>
        <metadata key="future_object" value="opaque-object"/>
        <part id="9" subtype="normal_part">
        <metadata key="name" value="part-name"/>
        <metadata key="module" value="part-module"/>
        <metadata key="part_type" value="normal_part"/>
        <metadata key="matrix" value="part-matrix"/>
        <metadata key="mesh_shared" value="1"/>
        <metadata key="source_file" value="part-source"/>
        <metadata key="source_in_meters" value="1"/>
        <metadata key="source_future" value="part-future-source"/>
        <metadata key="future_part" value="opaque-part"/>
        <mesh_stat edges_fixed="1" degenerate_facets="2" facets_removed="3"
          facets_reversed="4" backwards_edges="5"/>
        </part></object></config>"#,
    )
    .unwrap();
    let object = &settings.objects[0];
    let part = &object.parts[0];

    assert_eq!(object.name, "object-name");
    assert_eq!(object.module, "object-module");
    assert_eq!(
        pairs(&object.retained_config),
        [
            ("volume_type", "object-volume-type"),
            ("matrix", "object-matrix"),
            ("source_file", "object-source"),
            ("source_in_inches", "1"),
            ("source_future", "object-future-source"),
            ("future_object", "opaque-object"),
        ]
    );
    assert_eq!(
        pairs(&part.retained_metadata),
        [
            ("name", "part-name"),
            ("module", "part-module"),
            ("part_type", "normal_part"),
            ("matrix", "part-matrix"),
            ("mesh_shared", "1"),
            ("source_file", "part-source"),
            ("source_in_meters", "1"),
            ("source_future", "part-future-source"),
            ("future_part", "opaque-part"),
        ]
    );
    let mesh = part.mesh_stat.as_ref().unwrap();
    assert_eq!(
        (
            mesh.edges_fixed,
            mesh.degenerate_facets,
            mesh.facets_removed,
            mesh.facets_reversed,
            mesh.backwards_edges,
        ),
        (1, 2, 3, 4, 5)
    );
}

#[test]
fn xml_applies_no_json_side_effects_or_thumbnail_composite() {
    let settings = parse_settings(
        r#"<config><object id="2">
        <metadata key="support_style" value="grid"/>
        <metadata key="support_type" value="hybrid(auto)"/>
        <metadata key="is_infill_first" value="0"/>
        <metadata key="wall_infill_order" value="infill/inner wall/outer wall"/>
        <metadata key="thumbnails_format" value="JPG"/>
        <metadata key="thumbnail_size" value="96x96"/>
        <part id="9" subtype="normal_part">
        <metadata key="is_infill_first" value="0"/>
        <metadata key="wall_infill_order" value="infill/outer wall/inner wall"/>
        </part>
        </object></config>"#,
    )
    .unwrap();
    let object = &settings.objects[0];

    assert_eq!(
        object.overrides.support_style,
        Some(ProcessSupportStyle::Grid)
    );
    assert_eq!(
        object.overrides.support_type,
        Some(ProcessSupportType::TreeAuto)
    );
    assert_eq!(
        object.region_overrides.is_infill_first,
        Some(OrcaBool(false))
    );
    assert_eq!(
        object.region_overrides.wall_sequence,
        Some(ProcessWallSequence::InnerOuter)
    );
    assert_eq!(
        pairs(&object.retained_config),
        [("thumbnails_format", "JPG"), ("thumbnails", "96x96")]
    );
    let part = &object.parts[0];
    assert_eq!(part.region_overrides.is_infill_first, Some(OrcaBool(false)));
    assert_eq!(
        part.region_overrides.wall_sequence,
        Some(ProcessWallSequence::OuterInner)
    );
}

#[test]
fn invalid_legacy_owner_values_report_source_and_canonical_target() {
    for (xml, source, target) in [
        (
            r#"<config><object id="2"><metadata key="support_material_extruder" value="bad"/></object></config>"#,
            "support_material_extruder",
            "support_filament",
        ),
        (
            r#"<config><object id="2"><part id="9" subtype="normal_part"><metadata key="wall_filament" value="bad"/></part></object></config>"#,
            "wall_filament",
            "outer_wall_filament_id",
        ),
    ] {
        let error = parse_settings(xml).unwrap_err();
        let SliceError::InvalidInput(message) = error else {
            panic!("unexpected error: {error}");
        };
        assert!(message.contains(source), "{message}");
        assert!(message.contains(target), "{message}");
    }
}

#[test]
fn deferred_profile_metadata_is_not_stored_as_model_state() {
    for (source, part_path) in [
        ("inherits_cummulative", false),
        ("compatible_printers_condition_cummulative", false),
        ("compatible_prints_condition_cummulative", false),
        ("different_settings_to_system", true),
    ] {
        let xml = if part_path {
            format!(
                r#"<config><object id="2"><part id="9" subtype="normal_part"><metadata key="{source}" value="profile"/></part></object></config>"#
            )
        } else {
            format!(
                r#"<config><object id="2"><metadata key="{source}" value="profile"/></object></config>"#
            )
        };
        let error = parse_settings(&xml).unwrap_err();
        let SliceError::InvalidInput(message) = error else {
            panic!("unexpected error: {error}");
        };
        assert!(message.contains(source), "{message}");
        assert!(message.contains("deferred"), "{message}");
    }
}
