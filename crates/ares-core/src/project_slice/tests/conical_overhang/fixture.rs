use crate::{OrcaBool, OrcaFloat, OrcaInt, Percent, SliceError, slice_project};

use super::super::super::prepare_post_top_empty_layers;
use super::super::support::{KsrArchive, metadata};
use crate::project_slice::ProjectSource;

const PROCESS: &str = "Metadata/project_settings.config";
const DISABLED: &str = r#""make_overhang_printable": "0""#;
const ENABLED: &str = r#""make_overhang_printable": "1""#;
const ANGLE_55: &str = r#""make_overhang_printable_angle": "55""#;
const ANGLE_45: &str = r#""make_overhang_printable_angle": "45""#;

#[test]
fn stepped_archives_load_project_overhang_options() {
    assert_loaded_options(&stepped_builder(false).bytes(), 45.0, false);
    assert_loaded_options(&stepped_builder(true).bytes(), 45.0, true);
}

#[tokio::test]
async fn task22l_stepped_invalid_options_fail_before_disabled_gate() {
    let cases = [
        (
            ANGLE_45,
            r#""make_overhang_printable_angle": "-0.1""#,
            "invalid Orca option make_overhang_printable_angle",
        ),
        (
            ANGLE_45,
            r#""make_overhang_printable_angle": "90.1""#,
            "invalid Orca option make_overhang_printable_angle",
        ),
        (
            r#""make_overhang_printable_hole_size": "0""#,
            r#""make_overhang_printable_hole_size": "-0.1""#,
            "invalid Orca option make_overhang_printable_hole_size",
        ),
    ];
    for (from, to, message) in cases {
        let mut archive = stepped_builder(false);
        archive.replace_unique(PROCESS, from, to);
        let project = archive.bytes();
        let error = || SliceError::InvalidInput(message.to_owned());
        assert_eq!(slice_project(&project, metadata()).await, Err(error()));
    }
}

fn stepped_builder(enabled: bool) -> KsrArchive {
    let mut archive = KsrArchive::new();
    for (path, text) in [
        ("3D/3dmodel.model", ROOT),
        ("3D/_rels/3dmodel.model.rels", RELATIONSHIPS),
        ("3D/Objects/task22l_step.model", LEAF),
        ("Metadata/model_settings.config", SETTINGS),
    ] {
        archive.insert_text(path, text);
    }
    archive.replace_unique(PROCESS, ANGLE_55, ANGLE_45);
    if enabled {
        archive.replace_unique(PROCESS, DISABLED, ENABLED);
    }
    archive
}

fn assert_loaded_options(project: &[u8], angle: f64, enabled: bool) {
    let prepared = prepare_post_top_empty_layers(ProjectSource::from(project)).unwrap();
    let object = &prepared.resolved.objects[0].object;
    assert_eq!(
        (
            object.layer_height,
            object.make_overhang_printable_angle,
            object.make_overhang_printable_hole_size,
        ),
        (OrcaFloat(0.2), OrcaFloat(angle), OrcaFloat(0.0))
    );
    let region = &prepared.objects[0].regions[0].options;
    assert_eq!(
        (
            region.make_overhang_printable,
            region.bottom_shell_layers,
            region.top_shell_layers,
            region.sparse_infill_density,
            region.wall_loops,
        ),
        (
            OrcaBool(enabled),
            OrcaInt(3),
            OrcaInt(5),
            Percent(15.0),
            OrcaInt(2),
        )
    );
}

#[rustfmt::skip]
const ROOT: &str = r#"<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:p="http://schemas.microsoft.com/3dmanufacturing/production/2015/06" requiredextensions="p"><resources><object id="2" type="model"><components><component p:path="/3D/Objects/task22l_step.model" objectid="1"/></components></object></resources><build><item objectid="2" transform="1 0 0 0 1 0 0 0 1 0 0 0" printable="1" auto_drop="1"/></build></model>"#;
#[rustfmt::skip]
const RELATIONSHIPS: &str = r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Target="/3D/Objects/task22l_step.model" Id="step" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/></Relationships>"#;
#[rustfmt::skip]
const SETTINGS: &str = r#"<config><object id="2"><part id="1" subtype="normal_part"/></object><plate><metadata key="plater_id" value="1"/><model_instance><metadata key="object_id" value="2"/><metadata key="instance_id" value="0"/><metadata key="identify_id" value="22001"/></model_instance></plate><assemble><assemble_item object_id="2" instance_id="0" transform="1 0 0 0 1 0 0 0 1 0 0 0" offset="0 0 0"/></assemble></config>"#;
#[rustfmt::skip]
const LEAF: &str = r#"<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"><resources><object id="1" type="model"><mesh><vertices><vertex x="0" y="0" z="0"/><vertex x="6" y="0" z="0"/><vertex x="6" y="6" z="0"/><vertex x="0" y="6" z="0"/><vertex x="0" y="0" z="0.2"/><vertex x="6" y="0" z="0.2"/><vertex x="6" y="6" z="0.2"/><vertex x="0" y="6" z="0.2"/><vertex x="4" y="0" z="0.2"/><vertex x="10" y="0" z="0.2"/><vertex x="10" y="6" z="0.2"/><vertex x="4" y="6" z="0.2"/><vertex x="4" y="0" z="0.4"/><vertex x="10" y="0" z="0.4"/><vertex x="10" y="6" z="0.4"/><vertex x="4" y="6" z="0.4"/></vertices><triangles><triangle v1="0" v2="2" v3="1"/><triangle v1="0" v2="3" v3="2"/><triangle v1="4" v2="5" v3="6"/><triangle v1="4" v2="6" v3="7"/><triangle v1="0" v2="1" v3="5"/><triangle v1="0" v2="5" v3="4"/><triangle v1="1" v2="2" v3="6"/><triangle v1="1" v2="6" v3="5"/><triangle v1="2" v2="3" v3="7"/><triangle v1="2" v2="7" v3="6"/><triangle v1="3" v2="0" v3="4"/><triangle v1="3" v2="4" v3="7"/><triangle v1="8" v2="10" v3="9"/><triangle v1="8" v2="11" v3="10"/><triangle v1="12" v2="13" v3="14"/><triangle v1="12" v2="14" v3="15"/><triangle v1="8" v2="9" v3="13"/><triangle v1="8" v2="13" v3="12"/><triangle v1="9" v2="10" v3="14"/><triangle v1="9" v2="14" v3="13"/><triangle v1="10" v2="11" v3="15"/><triangle v1="10" v2="15" v3="14"/><triangle v1="11" v2="8" v3="12"/><triangle v1="11" v2="12" v3="15"/></triangles></mesh></object></resources><build/></model>"#;
