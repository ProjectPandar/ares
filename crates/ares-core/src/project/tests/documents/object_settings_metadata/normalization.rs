use crate::options::ObjectOptionOverrides;

use super::{object_overrides, parse_settings, region_overrides};

#[test]
fn object_options_normalization_drivers_remain_ordered_and_outside_sparse_overrides() {
    let first = parse_settings(
        r#"<config><object id="2">
        <metadata key="extruder" value="2"/>
        <metadata key="spiral_mode" value="1"/>
        <metadata key="enable_prime_tower" value="1"/>
        <metadata key="enable_wrapping_detection" value="1"/>
        </object></config>"#,
    )
    .unwrap();
    let second = parse_settings(
        r#"<config><object id="2">
        <metadata key="extruder" value="1"/>
        <metadata key="spiral_mode" value="0"/>
        <metadata key="enable_prime_tower" value="0"/>
        <metadata key="enable_wrapping_detection" value="0"/>
        </object></config>"#,
    )
    .unwrap();
    let first = &first.objects[0];
    let second = &second.objects[0];

    assert_eq!(object_overrides(first), &ObjectOptionOverrides::default());
    assert_eq!(object_overrides(second), &ObjectOptionOverrides::default());
    assert_eq!(region_overrides(first).extruder, Some(crate::OrcaInt(2)));
    assert_eq!(region_overrides(second).extruder, Some(crate::OrcaInt(1)));
}
