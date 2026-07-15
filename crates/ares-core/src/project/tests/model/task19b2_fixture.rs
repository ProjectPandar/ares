use crate::options::{ObjectOptionOverrides, RegionOptionOverrides};
use crate::{OrcaInt, ProjectVolumeType, load_project};

use super::fixture::FIXTURE;

#[test]
fn real_fixture_associates_typed_model_configuration_with_domain_owners() {
    let project = load_project(FIXTURE).unwrap();
    let [object] = project.objects() else {
        panic!("expected one project object")
    };
    let [volume] = object.volumes() else {
        panic!("expected one project volume")
    };

    let object_region = RegionOptionOverrides {
        extruder: Some(OrcaInt(1)),
        ..Default::default()
    };
    assert_eq!(object.id(), 2);
    assert_eq!(object.name(), "ksr_fdmtest_v4.drc");
    assert_eq!(object.module(), "");
    assert_eq!(object.object_overrides(), &ObjectOptionOverrides::default());
    assert_eq!(object.region_overrides(), &object_region);
    assert!(object.layer_config_ranges().is_empty());

    assert_eq!(volume.id(), 1);
    assert_eq!(volume.name(), "ksr_fdmtest_v4.drc");
    assert_eq!(volume.volume_type(), ProjectVolumeType::ModelPart);
    assert_eq!(volume.region_overrides(), &RegionOptionOverrides::default());

    let [source_object] = project.documents().model_settings.objects.as_slice() else {
        panic!("expected one retained model-settings object")
    };
    let [source_part] = source_object.parts.as_slice() else {
        panic!("expected one retained model-settings part")
    };
    assert_eq!(source_object.id, 2);
    assert_eq!(source_object.name, "ksr_fdmtest_v4.drc");
    assert_eq!(source_object.module, "");
    assert_eq!(source_object.overrides, ObjectOptionOverrides::default());
    assert_eq!(source_object.region_overrides, object_region);
    assert_eq!(source_part.id, 1);
    assert_eq!(source_part.subtype, "normal_part");
    assert_eq!(
        source_part.region_overrides,
        RegionOptionOverrides::default()
    );
    assert_eq!(
        source_part
            .retained_metadata
            .iter()
            .map(|entry| (entry.key.as_str(), entry.value.as_str()))
            .collect::<Vec<_>>(),
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
        source_part.mesh_stat.as_ref().map(|statistics| [
            statistics.edges_fixed,
            statistics.degenerate_facets,
            statistics.facets_removed,
            statistics.facets_reversed,
            statistics.backwards_edges,
        ]),
        Some([0; 5])
    );
}
