use crate::{OrcaInt, Percent, SliceError};

use super::{pairs, parse_settings, region_overrides, retained_config};

#[test]
fn object_region_metadata_is_typed_last_write_wins_and_not_retained() {
    let settings = parse_settings(
        r#"<config><object id="2">
        <metadata key="future_a" value="alpha"/>
        <metadata key="sparse_infill_density" value="31%"/>
        <metadata key="extruder" value="1"/>
        <metadata key="future_b" value="beta"/>
        <metadata key="sparse_infill_density" value="47%"/>
        <metadata key="extruder" value="2"/>
        <metadata key="future_a" value="omega"/>
        </object></config>"#,
    )
    .unwrap();
    let object = &settings.objects[0];

    assert_eq!(
        region_overrides(object).sparse_infill_density,
        Some(Percent(47.0))
    );
    assert_eq!(region_overrides(object).extruder, Some(OrcaInt(2)));
    assert_eq!(
        pairs(retained_config(object)),
        [
            ("future_a", "alpha"),
            ("future_b", "beta"),
            ("future_a", "omega")
        ]
    );
}

#[test]
fn object_region_metadata_reports_the_malformed_key() {
    let error = parse_settings(
        r#"<config><object id="2"><metadata key="sparse_infill_density" value="bad"/></object></config>"#,
    )
    .unwrap_err();
    let SliceError::InvalidInput(message) = error else {
        panic!("unexpected error: {error}");
    };

    assert!(
        message.contains("invalid Orca region option sparse_infill_density"),
        "{message}"
    );
}

#[test]
fn part_region_metadata_is_typed_last_write_wins_and_preserves_residual_order() {
    let settings = parse_settings(
        r#"<config><object id="2"><part id="9" subtype="normal_part">
        <metadata key="name" value="part-a"/>
        <metadata key="wall_loops" value="3"/>
        <metadata key="matrix" value="1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1"/>
        <metadata key="extruder" value="1"/>
        <metadata key="future_part" value="alpha"/>
        <metadata key="wall_loops" value="5"/>
        <metadata key="extruder" value="2"/>
        <metadata key="future_part" value="omega"/>
        </part></object></config>"#,
    )
    .unwrap();
    let part = &settings.objects[0].parts[0];

    assert_eq!(part.region_overrides.wall_loops, Some(OrcaInt(5)));
    assert_eq!(part.region_overrides.extruder, Some(OrcaInt(2)));
    assert_eq!(
        pairs(&part.retained_metadata),
        [
            ("name", "part-a"),
            ("matrix", "1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1"),
            ("future_part", "alpha"),
            ("future_part", "omega")
        ]
    );
}

#[test]
fn part_region_metadata_reports_the_malformed_key() {
    let error = parse_settings(
        r#"<config><object id="2"><part id="9" subtype="normal_part"><metadata key="wall_loops" value="bad"/></part></object></config>"#,
    )
    .unwrap_err();
    let SliceError::InvalidInput(message) = error else {
        panic!("unexpected error: {error}");
    };

    assert!(
        message.contains("invalid Orca region option wall_loops"),
        "{message}"
    );
}
