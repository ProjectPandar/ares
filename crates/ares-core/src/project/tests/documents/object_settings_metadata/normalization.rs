use crate::options::ObjectOptionOverrides;

use super::{object_overrides, pairs, parse_settings, retained_config};

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

    assert_eq!(
        pairs(retained_config(first)),
        [
            ("extruder", "2"),
            ("spiral_mode", "1"),
            ("enable_prime_tower", "1"),
            ("enable_wrapping_detection", "1"),
        ]
    );
    assert_eq!(
        pairs(retained_config(second)),
        [
            ("extruder", "1"),
            ("spiral_mode", "0"),
            ("enable_prime_tower", "0"),
            ("enable_wrapping_detection", "0"),
        ]
    );
    assert_eq!(object_overrides(first), &ObjectOptionOverrides::default());
    assert_eq!(object_overrides(second), &ObjectOptionOverrides::default());
}
