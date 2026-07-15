use crate::{ObjectOptions, OrcaFloat, OrcaInt, Percent, ProcessObjectSourceOptions};

use super::{object_overrides, parse_settings, region_overrides};

fn project(
    base: &ProcessObjectSourceOptions,
    overrides: &crate::options::ObjectOptionOverrides,
) -> ObjectOptions {
    ObjectOptions::overlay(base, overrides)
}

#[test]
fn object_options_projection_ordered_duplicate_xml_handoff_uses_last_value() {
    let forward = parse_settings(
        r#"<config><object id="2">
        <metadata key="brim_width" value="7.25"/>
        <metadata key="brim_width" value="0"/>
        </object></config>"#,
    )
    .unwrap();
    let reverse = parse_settings(
        r#"<config><object id="2">
        <metadata key="brim_width" value="0"/>
        <metadata key="brim_width" value="7.25"/>
        </object></config>"#,
    )
    .unwrap();
    let forward = &forward.objects[0];
    let reverse = &reverse.objects[0];

    assert_eq!(object_overrides(forward).brim_width, Some(OrcaFloat(0.0)));
    assert_eq!(object_overrides(reverse).brim_width, Some(OrcaFloat(7.25)));

    let base = ProcessObjectSourceOptions {
        brim_width: OrcaFloat(4.5),
        ..Default::default()
    };
    let mut forward_expected = ObjectOptions::from_base(&base);
    forward_expected.brim_width = OrcaFloat(0.0);
    let mut reverse_expected = ObjectOptions::from_base(&base);
    reverse_expected.brim_width = OrcaFloat(7.25);

    assert_eq!(project(&base, object_overrides(forward)), forward_expected);
    assert_eq!(project(&base, object_overrides(reverse)), reverse_expected);
}

#[test]
fn object_options_projection_retained_entries_stay_ordered_and_isolated() {
    let settings = parse_settings(
        r#"<config><object id="2">
        <metadata key="brim_width" value="6.75"/>
        <metadata key="extruder" value="2"/>
        <metadata key="sparse_infill_density" value="37.5%"/>
        </object></config>"#,
    )
    .unwrap();
    let object = &settings.objects[0];
    assert_eq!(
        object_overrides(object),
        &crate::options::ObjectOptionOverrides {
            brim_width: Some(OrcaFloat(6.75)),
            ..Default::default()
        }
    );
    assert_eq!(region_overrides(object).extruder, Some(OrcaInt(2)));
    assert_eq!(
        region_overrides(object).sparse_infill_density,
        Some(Percent(37.5))
    );

    let base = ProcessObjectSourceOptions {
        brim_width: OrcaFloat(4.5),
        support_filament: OrcaInt(11),
        support_interface_filament: OrcaInt(12),
        ..Default::default()
    };
    let mut expected = ObjectOptions::from_base(&base);
    expected.brim_width = OrcaFloat(6.75);

    assert_eq!(project(&base, object_overrides(object)), expected);
}
