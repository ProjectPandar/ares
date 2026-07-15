use crate::SliceError;

use super::{pairs, parse_settings};

#[test]
fn object_structural_scope_is_only_name_and_module() {
    let settings = parse_settings(
        r#"<config><object id="2">
        <metadata key="name" value="first"/>
        <metadata key="module" value="module-a"/>
        <metadata key="name" value="last"/>
        <metadata key="module" value="module-b"/>
        </object></config>"#,
    )
    .unwrap();
    let object = &settings.objects[0];
    assert_eq!(object.name, "last");
    assert_eq!(object.module, "module-b");

    for key in [
        "volume_type",
        "part_type",
        "matrix",
        "mesh_shared",
        "source_file",
        "source_object_id",
        "source_volume_id",
        "source_offset_x",
        "source_offset_y",
        "source_offset_z",
        "source_in_inches",
        "source_in_meters",
    ] {
        assert_unknown(
            &format!(
                r#"<config><object id="2"><metadata key="{key}" value="x"/></object></config>"#
            ),
            key,
        );
    }
}

#[test]
fn part_structural_scope_retains_only_fixed_ordered_provenance() {
    let settings = parse_settings(
        r#"<config><object id="2"><part id="9" subtype="normal_part">
        <metadata key="name" value="part-a"/>
        <metadata key="volume_type" value="negative_part"/>
        <metadata key="part_type" value="modifier_part"/>
        <metadata key="matrix" value="matrix-a"/>
        <metadata key="mesh_shared" value="1"/>
        <metadata key="source_file" value="part.stl"/>
        <metadata key="source_object_id" value="7"/>
        <metadata key="source_volume_id" value="8"/>
        <metadata key="source_offset_x" value="1"/>
        <metadata key="source_offset_y" value="2"/>
        <metadata key="source_offset_z" value="3"/>
        <metadata key="source_in_inches" value="0"/>
        <metadata key="source_in_meters" value="1"/>
        <metadata key="name" value="part-b"/>
        </part></object></config>"#,
    )
    .unwrap();
    let part = &settings.objects[0].parts[0];
    assert_eq!(part.subtype, "normal_part");
    assert_eq!(
        pairs(&part.retained_metadata),
        [
            ("name", "part-a"),
            ("volume_type", "negative_part"),
            ("part_type", "modifier_part"),
            ("matrix", "matrix-a"),
            ("mesh_shared", "1"),
            ("source_file", "part.stl"),
            ("source_object_id", "7"),
            ("source_volume_id", "8"),
            ("source_offset_x", "1"),
            ("source_offset_y", "2"),
            ("source_offset_z", "3"),
            ("source_in_inches", "0"),
            ("source_in_meters", "1"),
            ("name", "part-b"),
        ]
    );

    for key in ["module", "source_future"] {
        assert_unknown(
            &format!(
                r#"<config><object id="2"><part id="9" subtype="normal_part"><metadata key="{key}" value="x"/></part></object></config>"#
            ),
            key,
        );
    }
}

fn assert_unknown(xml: &str, key: &str) {
    let error = parse_settings(xml).unwrap_err();
    let SliceError::InvalidInput(message) = error else {
        panic!("unexpected error: {error:?}");
    };
    assert!(message.contains(key), "{message}");
    assert!(message.len() <= 512, "{message}");
}
