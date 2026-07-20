use crate::{
    OrcaBool, OrcaFloat, OrcaInt, Percent, SliceError, slice_project, task22k_browser_oracle,
};

use super::super::super::{
    prepare_post_top_empty_layers, task22l_browser_input_oracle, task22l_browser_oracle,
};
use super::super::{
    region_fixture::checkpoint,
    support::{KsrArchive, ksr_project, metadata},
};

const PROCESS: &str = "Metadata/project_settings.config";
const DISABLED: &str = r#""make_overhang_printable": "0""#;
const ENABLED: &str = r#""make_overhang_printable": "1""#;
const ANGLE_55: &str = r#""make_overhang_printable_angle": "55""#;
const ANGLE_45: &str = r#""make_overhang_printable_angle": "45""#;

#[test]
fn task22l_phase_a_stepped_archives_options_and_k_are_exact() {
    let disabled = stepped_archive(false);
    let enabled = stepped_archive(true);
    let disabled_repeat = stepped_archive(false);
    let enabled_repeat = stepped_archive(true);

    assert_eq!(disabled_repeat, disabled);
    assert_eq!(enabled_repeat, enabled);
    assert_identity(
        &disabled,
        181_446,
        "ee928a255109b491b0640da279b86d9282c573ec49a400e3cc4529eac915030e",
    );
    assert_identity(
        &enabled,
        181_447,
        "be286d7abb2bef8ab5e8b650657b114ea35c4dcff3a1463eba1a0dd278a89faa",
    );
    assert_eq!(
        checkpoint::semantic_identity(&disabled),
        (
            1_020_460,
            "ade484830a6492b50c3233e51debf5eab1db7d3e3bbf81fa8cd72f10226ea9ef".to_owned(),
        )
    );
    assert_eq!(
        checkpoint::semantic_identity(&enabled),
        (
            1_020_460,
            "f61089d040d1edf002f1dedca66b433e4982e18b9ce69a6385aa42dbf4c780b9".to_owned(),
        )
    );

    assert_loaded_options(&disabled, 45.0, false);
    assert_loaded_options(&enabled, 45.0, true);
    let disabled_k = task22k_browser_oracle(&disabled).unwrap();
    let enabled_k = task22k_browser_oracle(&enabled).unwrap();
    let disabled_k_repeat = task22k_browser_oracle(&disabled_repeat).unwrap();
    let enabled_k_repeat = task22k_browser_oracle(&enabled_repeat).unwrap();
    assert_identity(
        &disabled_k,
        490,
        "c6668cfbc56b20abe71606d59d2e28abf08ebb8b22f3ecebb3058d63ba05b44f",
    );
    assert_eq!(enabled_k, disabled_k);
    assert_eq!(disabled_k_repeat, disabled_k);
    assert_eq!(enabled_k_repeat, enabled_k);
}

#[test]
fn task22l_stepped_k_and_l_checkpoints_are_exact() {
    let disabled = stepped_archive(false);
    let enabled = stepped_archive(true);
    let disabled_k = task22l_browser_input_oracle(&disabled).unwrap();
    let enabled_k = task22l_browser_input_oracle(&enabled).unwrap();
    assert_eq!(disabled_k, task22k_browser_oracle(&disabled).unwrap());
    assert_eq!(enabled_k, disabled_k);
    assert_identity(
        &disabled_k,
        490,
        "c6668cfbc56b20abe71606d59d2e28abf08ebb8b22f3ecebb3058d63ba05b44f",
    );

    let disabled_l = task22l_browser_oracle(&disabled).unwrap();
    let enabled_l = task22l_browser_oracle(&enabled).unwrap();
    assert_identity(
        &disabled_l,
        490,
        "0834c61cc48aece1afd52d060c5c2a58f7243124664ad0a7dd3f500d6735b790",
    );
    assert_identity(
        &enabled_l,
        554,
        "33038c51ffe6f41b0bdb8b921d6976f43b0c47f6f3be8ec3bee6cc5b9c7c2505",
    );
    assert_eq!(&disabled_l[8..], &disabled_k[8..]);
    assert_eq!(task22l_browser_oracle(&disabled).unwrap(), disabled_l);
    assert_eq!(task22l_browser_oracle(&enabled).unwrap(), enabled_l);

    let input = checkpoint::parse_k(&disabled_k).stream;
    let disabled_output = checkpoint::parse_l(&disabled_l).stream;
    let enabled_output = checkpoint::parse_l(&enabled_l).stream;
    assert_eq!(disabled_output, input);
    let input = &input.objects[0];
    let enabled = &enabled_output.objects[0];
    assert_eq!(enabled.source_object_index, input.source_object_index);
    assert_eq!(enabled.transform_index, input.transform_index);
    assert_eq!(enabled.planned_layer_count, input.planned_layer_count);
    assert_eq!(enabled.sidecars, input.sidecars);
    assert_eq!(enabled.retained_layers[1], input.retained_layers[1]);
    assert_ne!(enabled.retained_layers[0], input.retained_layers[0]);
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
        assert_eq!(task22l_browser_oracle(&project), Err(error()));
        assert_eq!(slice_project(&project, metadata()).await, Err(error()));
    }
}

#[tokio::test]
async fn task22l_committed_ksr_checkpoint_is_exact_and_public_stays_incomplete() {
    assert_loaded_options(ksr_project(), 55.0, false);
    let k = task22l_browser_input_oracle(ksr_project()).unwrap();
    assert_eq!(k, task22k_browser_oracle(ksr_project()).unwrap());
    let expected = checkpoint::encode_with_magic(&checkpoint::parse_k(&k).stream, b"ARES22L\0");
    let l = task22l_browser_oracle(ksr_project()).unwrap();
    assert_eq!(l, expected);
    assert_eq!(&l[8..], &k[8..]);
    assert_identity(
        &l,
        2_008_706,
        "7a71db2912970141adc436679621c25888c412e2010c44eccf1b49d7e8048b07",
    );
    assert_eq!(task22l_browser_oracle(ksr_project()).unwrap(), l);
    assert_eq!(
        slice_project(ksr_project(), metadata()).await,
        Err(SliceError::ProjectSlicingIncomplete)
    );
}

#[tokio::test]
async fn task22l_stepped_projects_stay_publicly_incomplete() {
    for project in [stepped_archive(false), stepped_archive(true)] {
        assert_eq!(
            slice_project(project, metadata()).await,
            Err(SliceError::ProjectSlicingIncomplete)
        );
    }
}

pub(super) fn stepped_archive(enabled: bool) -> Vec<u8> {
    stepped_builder(enabled).bytes()
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
    let prepared = prepare_post_top_empty_layers(project).unwrap();
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

fn assert_identity(bytes: &[u8], len: usize, sha: &str) {
    assert_eq!(bytes.len(), len);
    assert_eq!(checkpoint::sha256(bytes), sha);
}

#[rustfmt::skip]
const ROOT: &str = r#"<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:p="http://schemas.microsoft.com/3dmanufacturing/production/2015/06" requiredextensions="p"><resources><object id="2" type="model"><components><component p:path="/3D/Objects/task22l_step.model" objectid="1"/></components></object></resources><build><item objectid="2" transform="1 0 0 0 1 0 0 0 1 0 0 0" printable="1" auto_drop="1"/></build></model>"#;
#[rustfmt::skip]
const RELATIONSHIPS: &str = r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Target="/3D/Objects/task22l_step.model" Id="step" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/></Relationships>"#;
#[rustfmt::skip]
const SETTINGS: &str = r#"<config><object id="2"><part id="1" subtype="normal_part"/></object><plate><metadata key="plater_id" value="1"/><model_instance><metadata key="object_id" value="2"/><metadata key="instance_id" value="0"/><metadata key="identify_id" value="22001"/></model_instance></plate><assemble><assemble_item object_id="2" instance_id="0" transform="1 0 0 0 1 0 0 0 1 0 0 0" offset="0 0 0"/></assemble></config>"#;
#[rustfmt::skip]
const LEAF: &str = r#"<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"><resources><object id="1" type="model"><mesh><vertices><vertex x="0" y="0" z="0"/><vertex x="6" y="0" z="0"/><vertex x="6" y="6" z="0"/><vertex x="0" y="6" z="0"/><vertex x="0" y="0" z="0.2"/><vertex x="6" y="0" z="0.2"/><vertex x="6" y="6" z="0.2"/><vertex x="0" y="6" z="0.2"/><vertex x="4" y="0" z="0.2"/><vertex x="10" y="0" z="0.2"/><vertex x="10" y="6" z="0.2"/><vertex x="4" y="6" z="0.2"/><vertex x="4" y="0" z="0.4"/><vertex x="10" y="0" z="0.4"/><vertex x="10" y="6" z="0.4"/><vertex x="4" y="6" z="0.4"/></vertices><triangles><triangle v1="0" v2="2" v3="1"/><triangle v1="0" v2="3" v3="2"/><triangle v1="4" v2="5" v3="6"/><triangle v1="4" v2="6" v3="7"/><triangle v1="0" v2="1" v3="5"/><triangle v1="0" v2="5" v3="4"/><triangle v1="1" v2="2" v3="6"/><triangle v1="1" v2="6" v3="5"/><triangle v1="2" v2="3" v3="7"/><triangle v1="2" v2="7" v3="6"/><triangle v1="3" v2="0" v3="4"/><triangle v1="3" v2="4" v3="7"/><triangle v1="8" v2="10" v3="9"/><triangle v1="8" v2="11" v3="10"/><triangle v1="12" v2="13" v3="14"/><triangle v1="12" v2="14" v3="15"/><triangle v1="8" v2="9" v3="13"/><triangle v1="8" v2="13" v3="12"/><triangle v1="9" v2="10" v3="14"/><triangle v1="9" v2="14" v3="13"/><triangle v1="10" v2="11" v3="15"/><triangle v1="10" v2="15" v3="14"/><triangle v1="11" v2="8" v3="12"/><triangle v1="11" v2="12" v3="15"/></triangles></mesh></object></resources><build/></model>"#;
