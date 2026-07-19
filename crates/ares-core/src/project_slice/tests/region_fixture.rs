pub(super) mod checkpoint;

use checkpoint::{
    ExPolygon, GeometryLayer, ILayer, IObject, IStream, IVolume, JObject, JStream, ParsedJ, Region,
    RetainedLayer, Sidecar, Surface, encode_j, parse_i, parse_j, render_j, semantic_hash, sha256,
};

use crate::{
    OrcaFloat, SliceError,
    project::{ProjectVolumeType, load_project},
    slice_project, task22i_browser_input_oracle, task22j_browser_input_oracle,
    task22j_browser_oracle,
};

use super::support::{KsrArchive, ksr_project, metadata};

const KSR_SHA: &str = "698f40f13c9075b818abedd3d10f022fbb5d8200aed48fbdde651f6bfb21b8a9";
const MODIFIER_PART: &str = r#"<part id="3" subtype="modifier_part"/>"#;
type Identity = (usize, &'static str);
type RecordPair = (Identity, Identity);
macro_rules! identities {
    ($($name:ident = $value:expr),+ $(,)?) => { $(const $name: Identity = $value;)+ };
}
#[rustfmt::skip]
identities!(
    KSR_I=(999_721,"0dea485aea9f003db4dbadfd524e82cc2ad33327d3b447a7d985d57d82da72ef"), KSR_J=(2_008_706,"2b474697f4afae95c9a55d709d8740d382a80b2969fc5118dc89e13c1906162d"), SYNTHETIC_J=(5_880,"cb681dd4761dc69482f626374079f851ace0b9ec8d02587300c4495d84e0f4aa"),
    MODIFIER_H=(478,"4bc72e587c1a7061624d6a20df20d1cb4482dcad84951152ad4640d622b11f7a"), MODIFIER_I=(478,"4b37ef7c7816a29076288647810bcfb6fe0b341785b5a4505f602ab72f69cb87"), MODIFIER_J=(1_054,"1b18edae9cfbb9cd405cb7d45b1bec1a26168fe12c28a16366da211a30eadc77"), CONTROL_J=(698,"f2185c996e62a897b6af721f043a8ac150df647780693e828845f594524fd3d4")
);
const SYNTHETIC_TEXT_SHA: &str = "938c8bcb02449c0ea77617973aed9b907313a2b0e4d9bb526c73ce158ee59691";
const MODIFIER_TEXT_SHA: &str = "02078fb14801f33ae561793931aa56a24a1dcdb6c135f8be07753a45f897876a";
const CONTROL_TEXT_SHA: &str = "cf4e1c4668caa3c3676dd84d35bbc00f3f44644e4b85f56a921fb5582d9d3bad";
type RenderMetadata = (&'static str, [usize; 6]);
#[rustfmt::skip]
const SYNTHETIC_RENDER: [RenderMetadata; 10] = [("single_all_z_model_part_fast_path_with_empty_middle_layer",[3,0,0,0,0,0]),("disjoint_active_model_parts_only_first_moves",[1,0,0,0,0,0]),("stable_ids_do_not_replace_source_clipping_order",[0,1,1,0,0,0]),("negative_after_part_subtracts_band",[0,1,1,0,0,0]),("negative_before_part_leaves_later_part_unchanged",[1,0,0,0,0,0]),("modifier_chain_partitions_parent_then_child",[0,1,2,2,0,0]),("same_modifier_source_forwards_from_parent_b_to_parent_a",[0,1,2,2,1,2]),("same_region_normal_delta100_closes_gap150_not_gap250",[0,2,2,0,0,2]),("same_region_large_delta10_closes_gap15_not_gap25",[0,2,2,0,0,2]),("final_planned_layer_is_retained_after_negative_empties_it",[0,2,1,0,0,0])];
#[rustfmt::skip]
const MODIFIER_RENDER: [RenderMetadata; 1] = [("loaded_bridge_angle_37_modifier", [0,2,2,2,0,0])];
#[rustfmt::skip]
const CONTROL_RENDER: [RenderMetadata; 1] = [("loaded_no_override_modifier_control", [0,2,2,2,0,2])];
#[rustfmt::skip]
const MODIFIER_ZIP: (usize, &str, &str) = (56_046, "83ac43d83487ad5f63b7c4b8f8c88ef20bb75b286d09e329fe24c8abc08807ce", "82a7bdd3571da52daf92ec11a7a243ec279e9f053542804e2dfc1e10365d6fa3");
#[rustfmt::skip]
const CONTROL_ZIP: (usize, &str, &str) = (56_027, "4e1847cf020e217f9b90bef61cdb06c8fc2a953ca9dce100a161d3bcb99eca69", "e59b8041e64297f880e19ab42b51cbbac9f9394bd3f287ffe845edba595176e5");
#[rustfmt::skip]
const MODIFIER_GATES: [(&str, &str); 10] = [("wall_loops","2"),("sparse_infill_density","20%"),("top_shell_layers","2"),("bottom_shell_layers","2"),("sparse_infill_filament_id","2"),("internal_solid_filament_id","2"),("top_surface_filament_id","2"),("bottom_surface_filament_id","2"),("outer_wall_filament_id","2"),("inner_wall_filament_id","2")];
#[rustfmt::skip]
const KSR_RECORDS: [RecordPair; 4] = [((11_680,"bbc99a45cc9a566fefdbc4a7fa1ae80865858126f2ba0a9b9ee9c412f8414581"),(11_702,"633fcb207ed0be4092a75c7ad6052fa68579c4ced58371afa8837cd99d65c21e")),((24_216,"47486ac767ceea0b822566a750abc913c326141ca91eef5b27cfc1b37d26de4d"),(24_248,"486a43246ef4bc94b2119a4b5787662ff65162c416137caf5d131c1ea5d458ec")),((23_512,"ec3c90e0e8d276b9995169285b5b5a939e60bbd7283e46d0fa2c299bd8756816"),(23_544,"59eaf433513f5c92203cbd58b10612fb7b3438c627666d6e7a5dae24711c86ea")),((736,"fd1b4912b9472d854d664769d1d0e5c5ec49e9bb9efd67e43c5707bca9189d0a"),(761,"a19b98ff4513317e141d1dac1c7f978f60b50602210b7d1bd4afd94c9b4fe82d"))];

#[test]
#[rustfmt::skip]
fn task22j_released_ksr_input_i_is_exact() {
    assert_eq!(sha256(ksr_project()), KSR_SHA);
    let i = task22j_browser_input_oracle(ksr_project()).unwrap();
    assert_bytes(&i, KSR_I);
    let parsed = parse_i(&i, b"ARES22I\0");
    assert_eq!((parsed.objects.len(), parsed.objects[0].planned_layer_count), (1, 460));
    assert_eq!((parsed.objects[0].volumes.len(), parsed.objects[0].volumes[0].ordinal), (1, 1));
    assert_eq!(task22j_browser_input_oracle(ksr_project()).unwrap(), i);
}

#[test]
fn task22j_modifier_control_archives_and_h_i_are_complete() {
    let (modifier, control) = modifier_projects();
    assert_archive(&modifier, MODIFIER_ZIP);
    assert_archive(&control, CONTROL_ZIP);
    for (project, expected) in [(&modifier, Some(OrcaFloat(37.0))), (&control, None)] {
        let loaded = load_project(project).unwrap();
        let modifier = loaded.objects()[0]
            .volumes()
            .iter()
            .find(|volume| volume.volume_type() == ProjectVolumeType::ParameterModifier)
            .unwrap();
        assert_eq!(modifier.region_overrides().bridge_angle, expected);
    }
    let modifier_h = task22i_browser_input_oracle(&modifier).unwrap();
    let control_h = task22i_browser_input_oracle(&control).unwrap();
    let modifier_i = task22j_browser_input_oracle(&modifier).unwrap();
    let control_i = task22j_browser_input_oracle(&control).unwrap();
    assert_eq!(modifier_h, control_h);
    assert_eq!(modifier_i, control_i);
    assert_bytes(&modifier_h, MODIFIER_H);
    assert_bytes(&modifier_i, MODIFIER_I);
    assert_eq!(&modifier_h[..8], b"ARES22H\0");
    assert_eq!(&modifier_i[..8], b"ARES22I\0");
    assert_eq!(&modifier_h[7..], &modifier_i[7..]);
    assert_eq!(parse_i(&modifier_h, b"ARES22H\0"), expected_modifier_i());
    assert_eq!(parse_i(&modifier_i, b"ARES22I\0"), expected_modifier_i());
}

#[test]
#[rustfmt::skip]
fn task22j_complete_expected_j_vectors_are_frozen() {
    for (expected, identity, metadata, text_sha) in [
        (expected_modifier_j(true), MODIFIER_J, &MODIFIER_RENDER[..], MODIFIER_TEXT_SHA),
        (expected_modifier_j(false), CONTROL_J, &CONTROL_RENDER[..], CONTROL_TEXT_SHA),
        (expected_synthetic_j(), SYNTHETIC_J, &SYNTHETIC_RENDER[..], SYNTHETIC_TEXT_SHA),
    ] {
        let bytes = encode_j(&expected);
        assert_bytes(&bytes, identity);
        assert_eq!(parse_j(&bytes).stream, expected);
        assert_eq!(sha256(render_j(&expected, metadata).as_bytes()), text_sha);
    }
}

#[tokio::test]
async fn task22j_committed_ksr_target_j_is_exact_and_public_stays_incomplete() {
    let first = task22j_browser_oracle(ksr_project()).unwrap();
    assert_ksr_j(&first);
    assert_eq!(task22j_browser_oracle(ksr_project()).unwrap(), first);
    assert_eq!(
        slice_project(ksr_project(), metadata()).await.unwrap_err(),
        SliceError::ProjectSlicingIncomplete
    );
}

#[tokio::test]
async fn task22j_loaded_modifier_control_target_j_is_exact_and_public_stays_incomplete() {
    let (modifier, control) = modifier_projects();
    for (project, expected, identity) in [
        (modifier, expected_modifier_j(true), MODIFIER_J),
        (control, expected_modifier_j(false), CONTROL_J),
    ] {
        let actual = task22j_browser_oracle(&project).unwrap();
        assert_bytes(&actual, identity);
        assert_eq!(parse_j(&actual).stream, expected);
        assert_eq!(task22j_browser_oracle(&project).unwrap(), actual);
        assert_eq!(
            slice_project(project, metadata()).await.unwrap_err(),
            SliceError::ProjectSlicingIncomplete
        );
    }
}

#[test]
#[rustfmt::skip]
fn task22j_absent_and_loaded_empty_ranges_have_identical_final_j() { let absent = KsrArchive::new().bytes(); let mut empty = KsrArchive::new(); empty.insert_text("Metadata/layer_config_ranges.xml", "<objects/>"); assert_eq!(task22j_browser_oracle(absent).unwrap(), task22j_browser_oracle(empty.bytes()).unwrap()); }

#[tokio::test]
#[rustfmt::skip]
async fn task22j_external_capability_gates_reject_through_j_and_public_paths() {
    for &(key, value) in &MODIFIER_GATES { let metadata = format!(r#"<part id="3" subtype="modifier_part"><metadata key="{key}" value="{value}"/></part>"#); let settings = CONTROL_SETTINGS.replace(MODIFIER_PART, &metadata); assert_external_gate(modifier_archive(&settings).bytes(), key).await; }
    let mut shared = modifier_archive(CONTROL_SETTINGS); shared.replace_unique("Metadata/model_settings.config", r#"<part id="1" subtype="normal_part"/>"#, r#"<part id="1" subtype="normal_part"><metadata key="mesh_shared" value="0"/></part>"#); assert_external_gate(shared.bytes(), "shared_mesh_centering").await;
    let mut duplicate = modifier_archive(CONTROL_SETTINGS); duplicate.replace("3D/3dmodel.model", r#"objectid="3""#, r#"objectid="1""#); duplicate.replace_unique("3D/Objects/task22j_modifier.model", r#"<object id="3""#, r#"<object id="1""#); let duplicate = duplicate.bytes(); assert_eq!(load_project(&duplicate).unwrap().objects()[0].volumes().iter().map(|volume| volume.id()).collect::<Vec<_>>(), vec![1, 1]); assert_external_gate(duplicate, "shared_mesh_centering").await;
    let mut centering = modifier_archive(CONTROL_SETTINGS); centering.replace("3D/3dmodel.model", "</build>", r#"<item objectid="2" transform="1 0 0 0 1 0 0 0 1 0 0 3" printable="1" auto_drop="1"/></build>"#); centering.replace("Metadata/model_settings.config", "</plate>", r#"<model_instance><metadata key="object_id" value="2"/><metadata key="instance_id" value="1"/><metadata key="identify_id" value="22002"/></model_instance></plate>"#); centering.replace("Metadata/model_settings.config", "</assemble>", r#"<assemble_item object_id="2" instance_id="1" transform="1 0 0 0 1 0 0 0 1 0 0 3" offset="0 0 0"/></assemble>"#); assert_external_gate(centering.bytes(), "print_object_centering").await;
}

#[rustfmt::skip]
async fn assert_external_gate(project: Vec<u8>, key: &str) { let expected = SliceError::UnsupportedProjectFeature(key.to_owned()); assert_eq!(task22j_browser_oracle(&project).unwrap_err(), expected); assert_eq!(slice_project(project, metadata()).await.unwrap_err(), expected); }

pub(super) fn modifier_projects() -> (Vec<u8>, Vec<u8>) {
    (modifier_project(true), modifier_project(false))
}

fn modifier_project(with_override: bool) -> Vec<u8> {
    let settings = if with_override {
        MODIFIER_SETTINGS
    } else {
        CONTROL_SETTINGS
    };
    modifier_archive(settings).bytes()
}

fn modifier_archive(settings: &str) -> KsrArchive {
    let mut archive = KsrArchive::new();
    for (path, text) in [
        ("3D/3dmodel.model", ROOT_MODEL),
        ("3D/_rels/3dmodel.model.rels", RELATIONSHIPS),
        ("3D/Objects/ksr_fdmtest_v4.drc_2.model", NORMAL_LEAF),
        ("3D/Objects/task22j_modifier.model", MODIFIER_LEAF),
        ("Metadata/model_settings.config", settings),
    ] {
        archive.insert_text(path, text);
    }
    archive
}

#[rustfmt::skip]
const ROOT_MODEL: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<model unit=\"millimeter\" xmlns=\"http://schemas.microsoft.com/3dmanufacturing/core/2015/02\" xmlns:p=\"http://schemas.microsoft.com/3dmanufacturing/production/2015/06\" requiredextensions=\"p\">\n <metadata name=\"OrcaSlicer\">2.4.2</metadata>\n <resources><object id=\"2\" type=\"model\"><components>\n  <component p:path=\"/3D/Objects/ksr_fdmtest_v4.drc_2.model\" objectid=\"1\"/>\n  <component p:path=\"/3D/Objects/task22j_modifier.model\" objectid=\"3\"/>\n </components></object></resources>\n <build><item objectid=\"2\" transform=\"1 0 0 0 1 0 0 0 1 0 0 0\" printable=\"1\" auto_drop=\"1\"/></build>\n</model>";
#[rustfmt::skip]
const RELATIONSHIPS: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\n <Relationship Target=\"/3D/Objects/ksr_fdmtest_v4.drc_2.model\" Id=\"normal\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>\n <Relationship Target=\"/3D/Objects/task22j_modifier.model\" Id=\"modifier\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\"/>\n</Relationships>";
#[rustfmt::skip]
const NORMAL_LEAF: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<model unit=\"millimeter\" xmlns=\"http://schemas.microsoft.com/3dmanufacturing/core/2015/02\">\n <resources><object id=\"1\" type=\"model\"><mesh><vertices>\n<vertex x=\"0\" y=\"0\" z=\"0\"/><vertex x=\"20\" y=\"0\" z=\"0\"/><vertex x=\"20\" y=\"2\" z=\"0\"/><vertex x=\"0\" y=\"2\" z=\"0\"/>\n<vertex x=\"0\" y=\"0\" z=\"0.4\"/><vertex x=\"20\" y=\"0\" z=\"0.4\"/><vertex x=\"20\" y=\"2\" z=\"0.4\"/><vertex x=\"0\" y=\"2\" z=\"0.4\"/>\n</vertices><triangles>\n<triangle v1=\"0\" v2=\"2\" v3=\"1\"/><triangle v1=\"0\" v2=\"3\" v3=\"2\"/>\n<triangle v1=\"4\" v2=\"5\" v3=\"6\"/><triangle v1=\"4\" v2=\"6\" v3=\"7\"/>\n<triangle v1=\"0\" v2=\"1\" v3=\"5\"/><triangle v1=\"0\" v2=\"5\" v3=\"4\"/>\n<triangle v1=\"1\" v2=\"2\" v3=\"6\"/><triangle v1=\"1\" v2=\"6\" v3=\"5\"/>\n<triangle v1=\"2\" v2=\"3\" v3=\"7\"/><triangle v1=\"2\" v2=\"7\" v3=\"6\"/>\n<triangle v1=\"3\" v2=\"0\" v3=\"4\"/><triangle v1=\"3\" v2=\"4\" v3=\"7\"/>\n</triangles></mesh></object></resources><build/>\n</model>";
#[rustfmt::skip]
const MODIFIER_LEAF: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<model unit=\"millimeter\" xmlns=\"http://schemas.microsoft.com/3dmanufacturing/core/2015/02\">\n <resources><object id=\"3\" type=\"model\"><mesh><vertices>\n<vertex x=\"5\" y=\"0\" z=\"0\"/><vertex x=\"15\" y=\"0\" z=\"0\"/><vertex x=\"15\" y=\"2\" z=\"0\"/><vertex x=\"5\" y=\"2\" z=\"0\"/>\n<vertex x=\"5\" y=\"0\" z=\"0.4\"/><vertex x=\"15\" y=\"0\" z=\"0.4\"/><vertex x=\"15\" y=\"2\" z=\"0.4\"/><vertex x=\"5\" y=\"2\" z=\"0.4\"/>\n</vertices><triangles>\n<triangle v1=\"0\" v2=\"2\" v3=\"1\"/><triangle v1=\"0\" v2=\"3\" v3=\"2\"/>\n<triangle v1=\"4\" v2=\"5\" v3=\"6\"/><triangle v1=\"4\" v2=\"6\" v3=\"7\"/>\n<triangle v1=\"0\" v2=\"1\" v3=\"5\"/><triangle v1=\"0\" v2=\"5\" v3=\"4\"/>\n<triangle v1=\"1\" v2=\"2\" v3=\"6\"/><triangle v1=\"1\" v2=\"6\" v3=\"5\"/>\n<triangle v1=\"2\" v2=\"3\" v3=\"7\"/><triangle v1=\"2\" v2=\"7\" v3=\"6\"/>\n<triangle v1=\"3\" v2=\"0\" v3=\"4\"/><triangle v1=\"3\" v2=\"4\" v3=\"7\"/>\n</triangles></mesh></object></resources><build/>\n</model>";
#[rustfmt::skip]
const MODIFIER_SETTINGS: &str = "<config><object id=\"2\"><part id=\"1\" subtype=\"normal_part\"/><part id=\"3\" subtype=\"modifier_part\"><metadata key=\"bridge_angle\" value=\"37\"/></part></object><plate><metadata key=\"plater_id\" value=\"1\"/><model_instance><metadata key=\"object_id\" value=\"2\"/><metadata key=\"instance_id\" value=\"0\"/><metadata key=\"identify_id\" value=\"22001\"/></model_instance></plate><assemble><assemble_item object_id=\"2\" instance_id=\"0\" transform=\"1 0 0 0 1 0 0 0 1 0 0 0\" offset=\"0 0 0\"/></assemble></config>";
#[rustfmt::skip]
const CONTROL_SETTINGS: &str = "<config><object id=\"2\"><part id=\"1\" subtype=\"normal_part\"/><part id=\"3\" subtype=\"modifier_part\"/></object><plate><metadata key=\"plater_id\" value=\"1\"/><model_instance><metadata key=\"object_id\" value=\"2\"/><metadata key=\"instance_id\" value=\"0\"/><metadata key=\"identify_id\" value=\"22001\"/></model_instance></plate><assemble><assemble_item object_id=\"2\" instance_id=\"0\" transform=\"1 0 0 0 1 0 0 0 1 0 0 0\" offset=\"0 0 0\"/></assemble></config>";

#[rustfmt::skip]
fn expected_modifier_i() -> IStream {
    let layers = |expolygon: ExPolygon| (0..2).map(|index| ILayer { index, mode: 0, expolygons: vec![expolygon.clone()] }).collect();
    IStream { objects: vec![IObject { source_object_index: 0, transform_index: 0, planned_layer_count: 2, volumes: vec![
    IVolume { source_volume_index: 0, ordinal: 1, volume_type: 0, layers: layers(q(10_000_000, 1_000_000)) },
    IVolume { source_volume_index: 1, ordinal: 2, volume_type: 2, layers: layers(q(5_000_000, 1_000_000)) },
] }] } }

#[rustfmt::skip]
fn expected_modifier_j(changed: bool) -> JStream {
    let sidecar = |id, layers: Vec<ExPolygon>| Sidecar { occurrence_id: id, layers: layers.into_iter().enumerate().map(|(index, expolygon)| GeometryLayer { index: index as u64, expolygons: (!expolygon.contour.is_empty()).then_some(expolygon).into_iter().collect() }).collect() };
    let retained = |index, regions: Vec<Vec<ExPolygon>>| RetainedLayer { index, regions: regions.into_iter().enumerate().map(|(id, polygons)| Region { id: id as u64, surfaces: polygons.into_iter().map(|expolygon| Surface { kind: 4, expolygon }).collect() }).collect() };
    let full = q(10_000_000, 1_000_000); let center = q(5_000_000, 1_000_000);
    let regions = if changed { vec![vec![p(&[(-5_000_000,1_000_000),(-10_000_000,1_000_000),(-10_000_000,-1_000_000),(-5_000_000,-1_000_000)]), p(&[(10_000_000,1_000_000),(5_000_000,1_000_000),(5_000_000,-1_000_000),(10_000_000,-1_000_000)])], vec![center.clone()]] } else { vec![vec![full.clone()]] };
    JStream { objects: vec![JObject { source_object_index: 0, transform_index: 0, planned_layer_count: 2, sidecars: vec![sidecar(1, vec![full.clone(), full]), sidecar(2, vec![center.clone(), center])], retained_layers: (0..2).map(|index| retained(index, regions.clone())).collect() }] }
}

#[rustfmt::skip]
fn expected_synthetic_j() -> JStream {
    let sidecar = |id, layers: Vec<ExPolygon>| Sidecar { occurrence_id: id, layers: layers.into_iter().enumerate().map(|(index, expolygon)| GeometryLayer { index: index as u64, expolygons: (!expolygon.contour.is_empty()).then_some(expolygon).into_iter().collect() }).collect() };
    let ret = |index, regions: Vec<Vec<ExPolygon>>| RetainedLayer { index, regions: regions.into_iter().enumerate().map(|(id, polygons)| Region { id: id as u64, surfaces: polygons.into_iter().map(|expolygon| Surface { kind: 4, expolygon }).collect() }).collect() };
    let object = |id, planned, sidecars, retained_layers| JObject { source_object_index: id, transform_index: 0, planned_layer_count: planned, sidecars, retained_layers };
    let q0 = || q4(0, 0, 1_000, 1_000);
    JStream { objects: vec![
        object(0,3,vec![sidecar(42,vec![q4(0,0,100,100),empty(),q4(0,0,300,100)])],vec![ret(0,vec![vec![q4(0,0,100,100)]]),ret(1,vec![vec![]]),ret(2,vec![vec![q4(0,0,300,100)]])]),
        object(1,1,vec![sidecar(20,vec![q4(1000,0,1400,400)]),sidecar(70,vec![q4(0,0,400,400)])],vec![ret(0,vec![vec![q4(0,0,400,400)],vec![]])]),
        object(2,1,vec![sidecar(10,vec![q4(500,0,1500,1000)]),sidecar(90,vec![q0()])],vec![ret(0,vec![vec![p(&[(500,1000),(0,1000),(0,0),(500,0)])],vec![q4(500,0,1500,1000)]])]),
        object(3,1,vec![sidecar(50,vec![q4(400,-100,600,1100)]),sidecar(100,vec![q0()])],vec![ret(0,vec![vec![p(&[(1000,1000),(600,1000),(600,0),(1000,0)]),p(&[(400,1000),(0,1000),(0,0),(400,0)])]])]),
        object(4,1,vec![sidecar(50,vec![q4(400,-100,600,1100)]),sidecar(100,vec![q0()])],vec![ret(0,vec![vec![q0()]])]),
        object(5,1,vec![sidecar(101,vec![q4(0,0,1200,1200)]),sidecar(202,vec![q4(300,0,900,1200)]),sidecar(303,vec![q4(0,400,1200,800)])],vec![ret(0,vec![vec![p(&[(300,1200),(0,1200),(0,0),(300,0)]),p(&[(1200,1200),(900,1200),(900,0),(1200,0)])],vec![p(&[(900,1200),(300,1200),(300,800),(900,800)]),p(&[(900,400),(300,400),(300,0),(900,0)])],vec![p(&[(900,800),(300,800),(300,400),(900,400)])]])]),
        object(6,1,vec![sidecar(30,vec![q4(600,0,1000,1000)]),sidecar(50,vec![q4(200,0,800,1000)]),sidecar(70,vec![q4(0,0,400,1000)])],vec![ret(0,vec![vec![p(&[(1000,1000),(800,1000),(800,0),(1000,0)]),p(&[(200,1000),(0,1000),(0,0),(200,0)])],vec![p(&[(800,1000),(600,1000),(600,0),(800,0)]),p(&[(400,1000),(200,1000),(200,0),(400,0)])]])]),
        object(7,2,vec![sidecar(17,vec![q4(1150,0,2150,1000),q4(1250,0,2250,1000)]),sidecar(900,vec![q0(),q0()])],vec![ret(0,vec![vec![p(&[(2150,1000),(0,1000),(0,0),(2150,0)])]]),ret(1,vec![vec![p(&[(2250,1000),(1250,1000),(1250,0),(2250,0)]),p(&[(1000,1000),(0,1000),(0,0),(1000,0)])]])]),
        object(8,2,vec![sidecar(17,vec![q4(1015,0,2015,1000),q4(1025,0,2025,1000)]),sidecar(900,vec![q0(),q0()])],vec![ret(0,vec![vec![p(&[(2015,1000),(0,1000),(0,0),(2015,0)])]]),ret(1,vec![vec![p(&[(2025,1000),(1025,1000),(1025,0),(2025,0)]),p(&[(1000,1000),(0,1000),(0,0),(1000,0)])]])]),
        object(9,2,vec![sidecar(10,vec![empty(),q0()]),sidecar(90,vec![q0(),q0()])],vec![ret(0,vec![vec![q0()]]),ret(1,vec![vec![]])]),
    ] }
}
#[rustfmt::skip]
pub(super) fn assert_synthetic_j(bytes: &[u8]) { assert_bytes(bytes, SYNTHETIC_J); let actual = parse_j(bytes).stream; assert_eq!(actual, expected_synthetic_j()); assert_eq!(sha256(render_j(&actual, &SYNTHETIC_RENDER).as_bytes()), SYNTHETIC_TEXT_SHA); }
fn q(x: i64, y: i64) -> ExPolygon {
    p(&[(x, y), (-x, y), (-x, -y), (x, -y)])
}
fn q4(x0: i64, y0: i64, x1: i64, y1: i64) -> ExPolygon {
    p(&[(x0, y0), (x1, y0), (x1, y1), (x0, y1)])
}
fn p(points: &[(i64, i64)]) -> ExPolygon {
    ExPolygon {
        contour: points.to_vec(),
        holes: Vec::new(),
    }
}
fn empty() -> ExPolygon {
    p(&[])
}

fn assert_bytes(bytes: &[u8], expected: (usize, &str)) {
    assert_eq!(bytes.len(), expected.0);
    assert_eq!(sha256(bytes), expected.1)
}
fn assert_archive(bytes: &[u8], expected: (usize, &str, &str)) {
    assert_bytes(bytes, (expected.0, expected.1));
    assert_eq!(semantic_hash(bytes), expected.2)
}
fn assert_ksr_j(bytes: &[u8]) {
    assert_bytes(bytes, KSR_J);
    let ParsedJ {
        stream,
        sidecar_records,
        retained_records,
    } = parse_j(bytes);
    assert_eq!(stream.objects.len(), 1);
    let object = &stream.objects[0];
    assert_eq!(
        (
            object.source_object_index,
            object.transform_index,
            object.planned_layer_count,
            object.sidecars.len(),
            object.retained_layers.len(),
        ),
        (0, 0, 460, 1, 460)
    );
    let sidecar = &object.sidecars[0];
    assert_eq!((sidecar.occurrence_id, sidecar.layers.len()), (1, 460));
    for (index, (side, retained)) in sidecar
        .layers
        .iter()
        .zip(&object.retained_layers)
        .enumerate()
    {
        let index = u64::try_from(index).unwrap();
        assert_eq!(
            (side.index, retained.index, retained.regions.len()),
            (index, index, 1)
        );
        assert_eq!(retained.regions[0].id, 0);
        assert!(
            retained.regions[0]
                .surfaces
                .iter()
                .all(|surface| surface.kind == 4)
        );
    }
    for (index, slot) in [0, 46, 49, 459].into_iter().enumerate() {
        for (records, expected) in [
            (&sidecar_records, KSR_RECORDS[index].0),
            (&retained_records, KSR_RECORDS[index].1),
        ] {
            assert_bytes(&bytes[records[slot].clone()], expected);
        }
    }
    assert_eq!(
        geometry_totals(object),
        ((2890, 395, 58902), (2890, 395, 58902))
    );
}

fn geometry_totals(object: &JObject) -> ((usize, usize, usize), (usize, usize, usize)) {
    fn total<'a>(expolygons: impl Iterator<Item = &'a ExPolygon>) -> (usize, usize, usize) {
        expolygons.fold((0, 0, 0), |(count, holes, points), expolygon| {
            (
                count + 1,
                holes + expolygon.holes.len(),
                points
                    + expolygon.contour.len()
                    + expolygon.holes.iter().map(Vec::len).sum::<usize>(),
            )
        })
    }
    let sidecar = object
        .sidecars
        .iter()
        .flat_map(|sidecar| &sidecar.layers)
        .flat_map(|layer| &layer.expolygons);
    let retained = object
        .retained_layers
        .iter()
        .flat_map(|layer| &layer.regions)
        .flat_map(|region| &region.surfaces)
        .map(|surface| &surface.expolygon);
    (total(sidecar), total(retained))
}
