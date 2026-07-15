use crate::{OrcaFloat, OrcaInt, Percent, ProjectVolumeType, load_project};

use super::fixture::ProjectParts;

fn mesh(id: u32) -> String {
    format!(
        r#"<object id="{id}" type="model"><mesh><vertices><vertex x="0" y="0" z="0"/><vertex x="1" y="0" z="0"/><vertex x="0" y="1" z="0"/></vertices><triangles><triangle v1="0" v2="1" v3="2"/></triangles></mesh></object>"#
    )
}

fn configured_project() -> ProjectParts {
    let leaves = [101, 102, 201, 202, 203, 204, 205]
        .into_iter()
        .map(mesh)
        .collect::<String>();
    let model = format!(
        r#"<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"><resources>
        {leaves}
        <object id="10" type="model"><components><component objectid="102"/><component objectid="101"/></components></object>
        <object id="20" type="model"><components><component objectid="201"/><component objectid="202"/><component objectid="203"/><component objectid="204"/><component objectid="205"/></components></object>
        </resources><build><item objectid="20"/><item objectid="10"/></build></model>"#
    );
    let objects = r#"
        <object id="10">
          <metadata key="name" value="Ten"/><metadata key="module" value="module-ten"/>
          <metadata key="brim_width" value="7.5"/><metadata key="extruder" value="2"/>
          <part id="101" subtype="normal_part"><metadata key="name" value="part-101"/><metadata key="wall_loops" value="3"/></part>
          <part id="102" subtype="negative_part"><metadata key="name" value="part-102"/><metadata key="sparse_infill_density" value="35%"/></part>
        </object>
        <object id="20">
          <metadata key="name" value="Twenty"/><metadata key="module" value="module-twenty"/>
          <metadata key="brim_width" value="8.5"/><metadata key="extruder" value="3"/>
          <part id="201" subtype="normal_part"><metadata key="name" value="model"/></part>
          <part id="202" subtype="negative_part"><metadata key="name" value="negative"/></part>
          <part id="203" subtype="modifier_part"><metadata key="name" value="modifier"/></part>
          <part id="204" subtype="support_enforcer"><metadata key="name" value="enforcer"/></part>
          <part id="205" subtype="normal_part">
            <metadata key="name" value="old"/><metadata key="volume_type" value="negative_part"/>
            <metadata key="name" value="blocker"/><metadata key="part_type" value="support_blocker"/>
          </part>
        </object>"#;
    let mut parts = ProjectParts::valid();
    parts.make_single_model(&model);
    parts.set_model_settings_objects(objects, &[20, 10]);
    parts
}

#[test]
fn configured_owners_follow_build_identity_and_bfs_leaf_identity() {
    let project = load_project(configured_project().bytes()).unwrap();
    let [twenty, ten] = project.objects() else {
        panic!("expected two objects")
    };

    assert_eq!(
        (twenty.id(), twenty.name(), twenty.module()),
        (20, "Twenty", "module-twenty")
    );
    assert_eq!(twenty.object_overrides().brim_width, Some(OrcaFloat(8.5)));
    assert_eq!(twenty.region_overrides().extruder, Some(OrcaInt(3)));

    assert_eq!(
        (ten.id(), ten.name(), ten.module()),
        (10, "Ten", "module-ten")
    );
    assert_eq!(ten.object_overrides().brim_width, Some(OrcaFloat(7.5)));
    assert_eq!(ten.region_overrides().extruder, Some(OrcaInt(2)));
    assert_eq!(
        ten.volumes()
            .iter()
            .map(|volume| volume.id())
            .collect::<Vec<_>>(),
        [102, 101]
    );
    assert_eq!(ten.volumes()[0].name(), "part-102");
    assert_eq!(
        ten.volumes()[0].volume_type(),
        ProjectVolumeType::NegativeVolume
    );
    assert_eq!(
        ten.volumes()[0].region_overrides().sparse_infill_density,
        Some(Percent(35.0))
    );
    assert_eq!(ten.volumes()[1].name(), "part-101");
    assert_eq!(
        ten.volumes()[1].region_overrides().wall_loops,
        Some(OrcaInt(3))
    );
}

#[test]
fn nested_components_use_bfs_leaf_order_for_part_association() {
    let leaves = [1, 2, 3].into_iter().map(mesh).collect::<String>();
    let model = format!(
        r#"<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"><resources>
        {leaves}
        <object id="11" type="model"><components><component objectid="1"/><component objectid="2"/></components></object>
        <object id="10" type="model"><components><component objectid="11"/><component objectid="3"/></components></object>
        </resources><build><item objectid="10"/></build></model>"#
    );
    let object = r#"<object id="10"><metadata key="name" value="Nested"/>
        <part id="3" subtype="normal_part"><metadata key="name" value="three"/><metadata key="wall_loops" value="3"/></part>
        <part id="1" subtype="normal_part"><metadata key="name" value="one"/><metadata key="wall_loops" value="1"/></part>
        <part id="2" subtype="normal_part"><metadata key="name" value="two"/><metadata key="wall_loops" value="2"/></part>
        </object>"#;
    let mut parts = ProjectParts::valid();
    parts.make_single_model(&model);
    parts.set_model_settings_objects(object, &[10]);

    let project = load_project(parts.bytes()).unwrap();
    let volumes = project.objects()[0].volumes();

    assert_eq!(
        volumes.iter().map(|volume| volume.id()).collect::<Vec<_>>(),
        [3, 1, 2]
    );
    assert_eq!(
        volumes
            .iter()
            .map(|volume| volume.name())
            .collect::<Vec<_>>(),
        ["three", "one", "two"]
    );
    assert_eq!(
        volumes
            .iter()
            .map(|volume| volume.region_overrides().wall_loops)
            .collect::<Vec<_>>(),
        [Some(OrcaInt(3)), Some(OrcaInt(1)), Some(OrcaInt(2))]
    );
}

#[test]
fn all_volume_types_and_ordered_type_and_name_replacement_are_typed() {
    let project = load_project(configured_project().bytes()).unwrap();
    let volumes = project.objects()[0].volumes();

    assert_eq!(
        volumes
            .iter()
            .map(|volume| volume.volume_type())
            .collect::<Vec<_>>(),
        [
            ProjectVolumeType::ModelPart,
            ProjectVolumeType::NegativeVolume,
            ProjectVolumeType::ParameterModifier,
            ProjectVolumeType::SupportEnforcer,
            ProjectVolumeType::SupportBlocker,
        ]
    );
    assert_eq!(volumes[4].name(), "blocker");
}

#[test]
fn every_unknown_volume_type_source_is_a_bounded_keyed_error() {
    for (key, from, to) in [
        (
            "subtype",
            "subtype=\"normal_part\"",
            "subtype=\"future_part\"",
        ),
        (
            "volume_type",
            "</part>",
            "<metadata key=\"volume_type\" value=\"future_part\"/></part>",
        ),
        (
            "part_type",
            "</part>",
            "<metadata key=\"part_type\" value=\"future_part\"/></part>",
        ),
    ] {
        let mut parts = ProjectParts::valid();
        parts.replace("Metadata/model_settings.config", from, to);
        let message = load_project(parts.bytes()).unwrap_err().to_string();
        assert!(message.contains(key), "{message}");
        assert!(message.contains("future_part"), "{message}");
        assert!(message.len() <= 512, "unbounded error: {}", message.len());
    }
}

#[test]
fn unknown_type_on_an_extra_unmatched_part_is_still_rejected() {
    let unknown_type = format!("future_extra{}", "x".repeat(4_096));
    let mut parts = ProjectParts::valid();
    parts.replace(
        "Metadata/model_settings.config",
        "</object>",
        &format!(r#"<part id="999" subtype="{unknown_type}"/></object>"#),
    );
    let message = load_project(parts.bytes()).unwrap_err().to_string();

    assert!(message.contains("subtype"), "{message}");
    assert!(message.contains("future_extra"), "{message}");
    assert!(message.len() <= 512);
}

#[test]
fn duplicate_object_settings_ids_remain_rejected() {
    let mut parts = ProjectParts::valid();
    let object = r#"<object id="2"><part id="1" subtype="normal_part"/></object>"#;
    parts.set_model_settings_objects(&format!("{object}{object}"), &[2]);
    let message = load_project(parts.bytes()).unwrap_err().to_string();

    assert!(
        message.contains("repeat") && message.contains('2'),
        "{message}"
    );
    assert!(message.len() <= 512);
}
