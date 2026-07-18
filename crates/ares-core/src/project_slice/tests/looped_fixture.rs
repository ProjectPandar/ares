mod encoding;

use sha2::{Digest, Sha256};

use crate::{
    ProjectVolumeType, SliceError,
    mesh_slicer::{ChainedLayer, LoopedLayer},
    slice_project,
};

use super::{
    super::{
        chained_intersections::{ChainedPrintObject, chain_project_intersections},
        looped_intersections::{LoopedPrintObject, loop_project_intersections},
        state::prepare_project_slice,
    },
    support::{ksr_project, metadata},
};
use encoding::encode;

const LAYER_COUNT: usize = 460;
const CLOSED_POLYGON_COUNT: usize = 3_288;
const CLOSED_POINT_COUNT: usize = 116_472;
const ENCODING_LEN: usize = 2_190_993;
const CONFIG_BLOCK_LEN: usize = 49_004;
const FACE_ORDER_SHA256: &str = "6654d9a95ef1bb024f986552b0e8c866ad55dcbe5de3af0cf9c34ff52372adbe";
const SEMANTIC_SHA256: &str = "7df1e0f90f90e4ff5ca6249c1ceb61e5e1aca74dbdb7b9153fffeff4cd165cdd";
const CONFIG_BLOCK_SHA256: &str =
    "b33c979097a4900700d1e5dfcaa16f1454a79ce5fec48da7eb9458cfa2fdeeb8";

#[test]
fn task22d_ksr_fixture_loop_repair_is_an_exact_noop() {
    let state = prepare_project_slice(ksr_project()).unwrap();
    let max_gap_scaled = state.scale.checked_scale(2.0).unwrap();
    let chained = chain_project_intersections(state.intersected_objects);
    assert_no_open_polylines(&chained);

    let looped = loop_project_intersections(chained, max_gap_scaled);
    assert_eq!(looped.len(), 1);
    assert_eq!(looped[0].plan().source_object_index, 0);
    assert_eq!(looped[0].plan().transform_index, 0);
    assert_eq!(looped[0].plan().layers.len(), LAYER_COUNT);
    assert_eq!(looped[0].volumes().len(), 1);
    assert_eq!(looped[0].volumes()[0].ordinal(), 1);
    assert_eq!(
        looped[0].volumes()[0].volume_type(),
        ProjectVolumeType::ModelPart
    );

    let layers = fixture_layers(&looped);
    assert_eq!(layers.len(), LAYER_COUNT);
    assert_eq!(
        layers
            .iter()
            .map(|layer| layer.polygons().len())
            .sum::<usize>(),
        CLOSED_POLYGON_COUNT
    );
    assert_eq!(
        layers
            .iter()
            .flat_map(LoopedLayer::polygons)
            .map(|polygon| polygon.points().len())
            .sum::<usize>(),
        CLOSED_POINT_COUNT
    );
    assert_eq!(
        sorted_polygon_lengths(&layers[0]),
        [67, 68, 69, 70, 71, 80, 80, 80, 80, 80, 88, 213]
    );
    assert_eq!(sorted_polygon_lengths(&layers[230]), [38]);
    assert_eq!(sorted_polygon_lengths(&layers[459]), [8; 9]);

    assert_encoding(layers, false, FACE_ORDER_SHA256);
    assert_encoding(layers, true, SEMANTIC_SHA256);
}

#[tokio::test]
async fn task22d_ksr_fixture_loop_repair_is_repeatable_and_keeps_public_lifecycle() {
    let state = prepare_project_slice(ksr_project()).unwrap();
    let config_block = state.config_block.as_deref().unwrap();
    assert_eq!(config_block.len(), CONFIG_BLOCK_LEN);
    assert_eq!(sha256(config_block), CONFIG_BLOCK_SHA256);
    let max_gap_scaled = state.scale.checked_scale(2.0).unwrap();

    let first_chained = chain_project_intersections(state.intersected_objects.clone());
    let second_chained = chain_project_intersections(state.intersected_objects);
    assert_no_open_polylines(&first_chained);
    assert_no_open_polylines(&second_chained);
    let first = loop_project_intersections(first_chained, max_gap_scaled);
    let second = loop_project_intersections(second_chained, max_gap_scaled);
    assert_eq!(
        encode(fixture_layers(&first), false),
        encode(fixture_layers(&second), false)
    );
    assert_eq!(
        encode(fixture_layers(&first), true),
        encode(fixture_layers(&second), true)
    );

    assert_eq!(
        slice_project(ksr_project(), metadata()).await.unwrap_err(),
        SliceError::ProjectSlicingIncomplete
    );
}

fn assert_no_open_polylines(objects: &[ChainedPrintObject]) {
    assert_eq!(
        objects
            .iter()
            .flat_map(ChainedPrintObject::volumes)
            .flat_map(|volume| volume.layers())
            .flat_map(ChainedLayer::open_polylines)
            .count(),
        0
    );
}

fn fixture_layers(objects: &[LoopedPrintObject]) -> &[LoopedLayer] {
    objects[0].volumes()[0].layers()
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

fn assert_encoding(layers: &[LoopedLayer], semantic_order: bool, expected_sha: &str) {
    let encoded = encode(layers, semantic_order);
    assert_eq!(encoded.len(), ENCODING_LEN);
    assert_eq!(sha256(&encoded), expected_sha);
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
