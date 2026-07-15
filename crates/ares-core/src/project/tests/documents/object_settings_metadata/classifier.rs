use crate::{OrcaFloat, OrcaInt, Percent, SliceError};

use super::parse_settings;

#[test]
fn object_and_part_follow_owner_project_registry_unknown_order() {
    let settings = parse_settings(
        r#"<config><object id="2">
        <metadata key="brim_width" value="3.5"/>
        <metadata key="sparse_infill_density" value="41%"/>
        <metadata key="gcode_flavor" value="marlin"/>
        <metadata key="display_orientation" value="portrait"/>
        <part id="9" subtype="normal_part">
        <metadata key="wall_loops" value="4"/>
        <metadata key="gcode_flavor" value="klipper"/>
        <metadata key="display_orientation" value="landscape"/>
        </part></object></config>"#,
    )
    .unwrap();
    let object = &settings.objects[0];
    let part = &object.parts[0];

    assert_eq!(object.overrides.brim_width, Some(OrcaFloat(3.5)));
    assert_eq!(
        object.region_overrides.sparse_infill_density,
        Some(Percent(41.0))
    );
    assert_eq!(part.region_overrides.wall_loops, Some(OrcaInt(4)));
    assert!(part.retained_metadata.is_empty());

    for xml in [
        r#"<config><object id="2"><metadata key="future_object" value="x"/></object></config>"#,
        r#"<config><object id="2"><part id="9" subtype="normal_part"><metadata key="future_part" value="x"/></part></object></config>"#,
        r#"<config><object id="2"><metadata key="perimeter_feed_rate" value="40"/></object></config>"#,
    ] {
        let error = parse_settings(xml).unwrap_err();
        let SliceError::InvalidInput(message) = error else {
            panic!("unexpected error: {error:?}");
        };
        assert!(message.contains("future_") || message.contains("perimeter_feed_rate"));
        assert!(message.len() <= 512, "{message}");
    }
}

#[test]
fn canonical_and_legacy_assignments_remain_xml_last_write_wins() {
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
