use sha2::{Digest, Sha256};

use crate::{
    ProcessSlicingMode, SliceError,
    mesh_slicer::{LoopedLayer, SlicingMode},
    slice_project,
};

use super::{
    super::{
        chained_intersections::chain_project_intersections,
        looped_intersections::loop_project_intersections,
        slicing_mode_intersections::apply_project_slicing_modes, state::prepare_project_slice,
    },
    looped_fixture::encode,
    support::{KsrArchive, ksr_project, metadata},
};

const PROJECT_SETTINGS: &str = "Metadata/project_settings.config";
const MODEL_SETTINGS: &str = "Metadata/model_settings.config";
const PROCESS_REGULAR: &str = r#""slicing_mode": "regular""#;
const PROCESS_EVEN_ODD: &str = r#""slicing_mode": "even_odd""#;
const PROCESS_CLOSE_HOLES: &str = r#""slicing_mode": "close_holes""#;

const OBJECT_PART_ANCHOR: &str = concat!(
    "    <metadata key=\"extruder\" value=\"1\"/>\n",
    "    <part id=\"1\" subtype=\"normal_part\">",
);
const OBJECT_EVEN_ODD_OVERRIDE: &str = concat!(
    "    <metadata key=\"extruder\" value=\"1\"/>\n",
    "    <metadata key=\"slicing_mode\" value=\"even_odd\"/>\n",
    "    <part id=\"1\" subtype=\"normal_part\">",
);
const LAYER_COUNT: usize = 460;
const POLYGON_COUNT: usize = 3_288;
const POINT_COUNT: usize = 116_472;
const ENCODING_LEN: usize = 2_190_993;
const CONFIG_BLOCK_LEN: usize = 49_004;
const FACE_ORDER_SHA256: &str = "6654d9a95ef1bb024f986552b0e8c866ad55dcbe5de3af0cf9c34ff52372adbe";
const SEMANTIC_SHA256: &str = "7df1e0f90f90e4ff5ca6249c1ceb61e5e1aca74dbdb7b9153fffeff4cd165cdd";
const CONFIG_BLOCK_SHA256: &str =
    "b33c979097a4900700d1e5dfcaa16f1454a79ce5fec48da7eb9458cfa2fdeeb8";

struct FixtureSnapshot {
    resolved_mode: ProcessSlicingMode,
    spiral_mode: bool,
    bottom_shell_layers: i32,
    bottom_shell_thickness: f64,
    modes: Vec<SlicingMode>,
    layers: Vec<LoopedLayer>,
    config_block: Vec<u8>,
}

#[test]
fn task22e_ksr_fixture_regular_projection_is_exact_and_preserves_task22d_facts() {
    let snapshot = fixture_snapshot(ksr_project()).unwrap();
    assert_eq!(snapshot.resolved_mode, ProcessSlicingMode::Regular);
    assert!(!snapshot.spiral_mode);
    assert_eq!(snapshot.bottom_shell_layers, 3);
    assert_eq!(snapshot.bottom_shell_thickness, 0.0);
    assert_eq!(snapshot.modes, vec![SlicingMode::Regular; LAYER_COUNT]);
    assert_fixture_geometry(&snapshot);
    assert_eq!(snapshot.config_block.len(), CONFIG_BLOCK_LEN);
    assert_eq!(sha256(&snapshot.config_block), CONFIG_BLOCK_SHA256);
}

#[tokio::test]
async fn task22e_ksr_fixture_projection_is_repeatable_and_keeps_public_lifecycle() {
    let first = fixture_snapshot(ksr_project()).unwrap();
    let second = fixture_snapshot(ksr_project()).unwrap();
    assert_eq!(first.modes, second.modes);
    assert_eq!(encode(&first.layers, false), encode(&second.layers, false));
    assert_eq!(encode(&first.layers, true), encode(&second.layers, true));
    assert_eq!(
        slice_project(ksr_project(), metadata()).await.unwrap_err(),
        SliceError::ProjectSlicingIncomplete
    );
}

#[test]
fn task22e_ksr_process_slicing_modes_come_only_from_mutated_3mf_options() {
    let baseline = fixture_snapshot(ksr_project()).unwrap();
    let even_odd = fixture_snapshot(process_mode(PROCESS_EVEN_ODD)).unwrap();
    assert_eq!(even_odd.resolved_mode, ProcessSlicingMode::EvenOdd);
    assert_eq!(even_odd.modes, vec![SlicingMode::EvenOdd; LAYER_COUNT]);
    assert_eq!(
        encode(&even_odd.layers, false),
        encode(&baseline.layers, false)
    );
    assert_eq!(
        encode(&even_odd.layers, true),
        encode(&baseline.layers, true)
    );

    let close_holes = fixture_snapshot(process_mode(PROCESS_CLOSE_HOLES)).unwrap();
    assert_eq!(close_holes.resolved_mode, ProcessSlicingMode::CloseHoles);
    assert_eq!(close_holes.modes, vec![SlicingMode::Positive; LAYER_COUNT]);
    assert_eq!(total_polygons(&close_holes.layers), POLYGON_COUNT);
    assert_eq!(total_points(&close_holes.layers), POINT_COUNT);
}

#[test]
fn task22e_ksr_object_slicing_mode_override_wins_over_process_base() {
    let baseline = fixture_snapshot(ksr_project()).unwrap();
    let mut archive = KsrArchive::new();
    archive.replace_unique(PROJECT_SETTINGS, PROCESS_REGULAR, PROCESS_CLOSE_HOLES);
    archive.replace_unique(MODEL_SETTINGS, OBJECT_PART_ANCHOR, OBJECT_EVEN_ODD_OVERRIDE);
    let overridden = fixture_snapshot(archive.bytes()).unwrap();
    assert_eq!(overridden.resolved_mode, ProcessSlicingMode::EvenOdd);
    assert_eq!(overridden.modes, vec![SlicingMode::EvenOdd; LAYER_COUNT]);
    assert_eq!(
        encode(&overridden.layers, false),
        encode(&baseline.layers, false)
    );
    assert_eq!(
        encode(&overridden.layers, true),
        encode(&baseline.layers, true)
    );
}

fn process_mode(mode: &str) -> Vec<u8> {
    let mut archive = KsrArchive::new();
    archive.replace_unique(PROJECT_SETTINGS, PROCESS_REGULAR, mode);
    archive.bytes()
}

fn fixture_snapshot(project: impl AsRef<[u8]>) -> Result<FixtureSnapshot, SliceError> {
    let state = prepare_project_slice(project)?;
    let resolved = state.resolved;
    let object = &resolved.objects[0];
    let region = &object.layer_candidates[0].model_parts[0].region;
    let resolved_mode = object.object.slicing_mode;
    let spiral_mode = resolved.views.full.process.print.spiral_mode.0;
    let bottom_shell_layers = region.bottom_shell_layers.0;
    let bottom_shell_thickness = region.bottom_shell_thickness.0;
    let config_block = state.config_block.unwrap();
    let max_gap_scaled = state.scale.checked_scale(2.0).unwrap();
    let chained = chain_project_intersections(state.intersected_objects);
    let looped = loop_project_intersections(chained, max_gap_scaled);
    let projected = apply_project_slicing_modes(looped, &resolved.objects, spiral_mode)?;
    let mut objects = projected.into_iter();
    let (plan, volumes) = objects.next().unwrap().into_parts();
    assert!(objects.next().is_none());
    assert_eq!(plan.source_object_index, 0);
    assert_eq!(plan.transform_index, 0);
    assert_eq!(plan.layers.len(), LAYER_COUNT);
    let mut volumes = volumes.into_iter();
    let (_, _, _, layers) = volumes.next().unwrap().into_parts();
    assert!(volumes.next().is_none());
    let (modes, layers) = layers.into_iter().map(|layer| layer.into_parts()).unzip();
    Ok(FixtureSnapshot {
        resolved_mode,
        spiral_mode,
        bottom_shell_layers,
        bottom_shell_thickness,
        modes,
        layers,
        config_block,
    })
}

fn assert_fixture_geometry(snapshot: &FixtureSnapshot) {
    assert_eq!(snapshot.layers.len(), LAYER_COUNT);
    assert_eq!(total_polygons(&snapshot.layers), POLYGON_COUNT);
    assert_eq!(total_points(&snapshot.layers), POINT_COUNT);
    assert_eq!(
        sorted_polygon_lengths(&snapshot.layers[0]),
        [67, 68, 69, 70, 71, 80, 80, 80, 80, 80, 88, 213]
    );
    assert_eq!(sorted_polygon_lengths(&snapshot.layers[230]), [38]);
    assert_eq!(sorted_polygon_lengths(&snapshot.layers[459]), [8; 9]);
    for (semantic, expected) in [(false, FACE_ORDER_SHA256), (true, SEMANTIC_SHA256)] {
        let encoded = encode(&snapshot.layers, semantic);
        assert_eq!(encoded.len(), ENCODING_LEN);
        assert_eq!(sha256(&encoded), expected);
    }
}

fn total_polygons(layers: &[LoopedLayer]) -> usize {
    layers.iter().map(|layer| layer.polygons().len()).sum()
}

fn total_points(layers: &[LoopedLayer]) -> usize {
    layers
        .iter()
        .flat_map(LoopedLayer::polygons)
        .map(|polygon| polygon.points().len())
        .sum()
}

fn sorted_polygon_lengths(layer: &LoopedLayer) -> Vec<usize> {
    let mut lengths = layer
        .polygons()
        .iter()
        .map(|polygon| polygon.points().len())
        .collect::<Vec<_>>();
    lengths.sort_unstable();
    lengths
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
