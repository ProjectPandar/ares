mod encoding;
mod oracles;

use sha2::{Digest, Sha256};

use crate::{ProjectVolumeType, mesh_slicer::ChainedLayer};

use super::{
    super::{
        chained_intersections::{ChainedPrintObject, chain_project_intersections},
        state::prepare_project_slice,
    },
    support::ksr_project,
};
use encoding::encode;
use oracles::{
    CLOSED_POINT_COUNT, CLOSED_POLYGON_COUNT, CONFIG_BLOCK_LEN, CONFIG_BLOCK_SHA256,
    FACE_ORDER_SHA256, LAYER_0_POLYGON_LENGTHS, LAYER_230_POLYGON_LENGTHS,
    LAYER_459_POLYGON_LENGTHS, LAYER_COUNT, RAW_LINE_COUNT, REPRESENTATIVE_LAYERS,
    SEMANTIC_ENCODING_LEN, SEMANTIC_SHA256,
};

#[test]
fn task22c_ksr_fixture_matches_exact_counts_lengths_and_digests() {
    let state = prepare_project_slice(ksr_project()).unwrap();
    assert_eq!(state.intersected_objects.len(), 1);
    let raw_object = &state.intersected_objects[0];
    assert_eq!(raw_object.volumes().len(), 1);
    let raw_volume = &raw_object.volumes()[0];
    let raw_layers = raw_volume.layers();
    assert_eq!(raw_layers.len(), LAYER_COUNT);
    assert_eq!(
        raw_layers.iter().map(Vec::len).sum::<usize>(),
        RAW_LINE_COUNT
    );

    let chained = chain_project_intersections(state.intersected_objects.clone());
    assert_eq!(chained.len(), 1);
    let object = &chained[0];
    assert_eq!(object.plan(), &raw_object.plan);
    assert_eq!(object.plan().source_object_index, 0);
    assert_eq!(object.plan().transform_index, 0);
    assert_eq!(object.plan().layers.len(), LAYER_COUNT);
    assert_eq!(object.volumes().len(), 1);

    let volume = &object.volumes()[0];
    assert_eq!(volume.ordinal(), raw_volume.ordinal());
    assert_eq!(volume.ordinal(), 1);
    assert_eq!(volume.volume_type(), raw_volume.volume_type());
    assert_eq!(volume.volume_type(), ProjectVolumeType::ModelPart);
    let layers = volume.layers();
    assert_eq!(layers.len(), LAYER_COUNT);

    assert_fixture_totals(raw_layers, layers);
    assert_representative_layers(raw_layers, layers);

    let face_order = encode(layers, false);
    let semantic = encode(layers, true);
    let face_order_sha = sha256(&face_order);
    let semantic_sha = sha256(&semantic);
    assert_eq!(
        (
            face_order.len(),
            face_order_sha.as_str(),
            semantic.len(),
            semantic_sha.as_str(),
        ),
        (
            SEMANTIC_ENCODING_LEN,
            FACE_ORDER_SHA256,
            SEMANTIC_ENCODING_LEN,
            SEMANTIC_SHA256,
        )
    );
}

#[test]
fn task22c_ksr_fixture_chaining_is_repeatable() {
    let state = prepare_project_slice(ksr_project()).unwrap();
    let config_block = state.config_block.as_deref().unwrap();
    assert_eq!(config_block.len(), CONFIG_BLOCK_LEN);
    assert_eq!(sha256(config_block), CONFIG_BLOCK_SHA256);

    let first = chain_project_intersections(state.intersected_objects.clone());
    let second = chain_project_intersections(state.intersected_objects);
    let first_layers = fixture_layers(&first);
    let second_layers = fixture_layers(&second);
    assert_eq!(encode(first_layers, false), encode(second_layers, false));
    assert_eq!(encode(first_layers, true), encode(second_layers, true));
}

fn assert_fixture_totals(
    raw_layers: &[Vec<crate::mesh_slicer::IntersectionLine>],
    layers: &[ChainedLayer],
) {
    let closed_count = layers
        .iter()
        .map(|layer| layer.polygons().len())
        .sum::<usize>();
    let open_count = layers
        .iter()
        .map(|layer| layer.open_polylines().len())
        .sum::<usize>();
    let closed_points = layers
        .iter()
        .flat_map(ChainedLayer::polygons)
        .map(|polygon| polygon.points().len())
        .sum::<usize>();
    assert_eq!(closed_count, CLOSED_POLYGON_COUNT);
    assert_eq!(open_count, 0);
    assert_eq!(closed_points, CLOSED_POINT_COUNT);

    assert!(
        layers
            .iter()
            .flat_map(ChainedLayer::polygons)
            .all(|polygon| polygon.points().first() != polygon.points().last())
    );
    for (raw, chained) in raw_layers.iter().zip(layers) {
        let closed_edges = chained
            .polygons()
            .iter()
            .map(|polygon| polygon.points().len())
            .sum::<usize>();
        let open_edges = chained
            .open_polylines()
            .iter()
            .map(|polyline| polyline.points().len() - 1)
            .sum::<usize>();
        assert_eq!(closed_edges + open_edges, raw.len());
    }
}

fn assert_representative_layers(
    raw_layers: &[Vec<crate::mesh_slicer::IntersectionLine>],
    layers: &[ChainedLayer],
) {
    for &(layer, raw_count, closed_count, open_count) in REPRESENTATIVE_LAYERS {
        assert_eq!(raw_layers[layer].len(), raw_count, "layer {layer}");
        assert_eq!(
            layers[layer].polygons().len(),
            closed_count,
            "layer {layer}"
        );
        assert_eq!(
            layers[layer].open_polylines().len(),
            open_count,
            "layer {layer}"
        );
    }
    assert_eq!(sorted_polygon_lengths(&layers[0]), LAYER_0_POLYGON_LENGTHS);
    assert_eq!(
        sorted_polygon_lengths(&layers[230]),
        LAYER_230_POLYGON_LENGTHS
    );
    assert_eq!(
        sorted_polygon_lengths(&layers[459]),
        LAYER_459_POLYGON_LENGTHS
    );
}

fn sorted_polygon_lengths(layer: &ChainedLayer) -> Vec<usize> {
    let mut lengths = layer
        .polygons()
        .iter()
        .map(|polygon| polygon.points().len())
        .collect::<Vec<_>>();
    lengths.sort_unstable();
    lengths
}

fn fixture_layers(objects: &[ChainedPrintObject]) -> &[ChainedLayer] {
    objects[0].volumes()[0].layers()
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
