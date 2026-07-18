use sha2::{Digest, Sha256};

use crate::{
    ProjectVolumeType, SliceError,
    geometry::{ExPolygon, Point, Polygon},
    mesh_slicer::{LoopedLayer, SlicingMode},
    slice_project,
};

use super::{
    super::{
        chained_intersections::chain_project_intersections,
        looped_intersections::loop_project_intersections,
        pre_closing_unions::{
            PreClosingLayer, PreClosingPrintObject, apply_project_pre_closing_unions,
        },
        slicing_mode_intersections::apply_project_slicing_modes,
        state::prepare_project_slice,
    },
    looped_fixture::encode as encode_looped,
    support::{ksr_project, metadata},
};

const LAYER_COUNT: usize = 460;
const RAW_ENCODING_LEN: usize = 2_190_993;
const RAW_ENCODING_SHA256: &str =
    "6654d9a95ef1bb024f986552b0e8c866ad55dcbe5de3af0cf9c34ff52372adbe";
const CONFIG_BLOCK_LEN: usize = 49_004;
const CONFIG_BLOCK_SHA256: &str =
    "b33c979097a4900700d1e5dfcaa16f1454a79ce5fec48da7eb9458cfa2fdeeb8";
const PROJECT_SHA256: &str = "698f40f13c9075b818abedd3d10f022fbb5d8200aed48fbdde651f6bfb21b8a9";
const ENCODER_VECTOR_LEN: usize = 255;
const ENCODER_VECTOR_SHA256: &str =
    "af7055df067e53aa48f789e31a09fbd3477391ebc3061ffea1153c0796877064";
const PRE_CLOSING_LEN: usize = 1_645_481;
const PRE_CLOSING_SHA256: &str = "209c6149c93994cc3ae6fa8e2f8f43dc9875b1b07b2320da9e67d8a2c43ab6e2";
const CONTOUR_COUNT: usize = 2_891;
const HOLE_COUNT: usize = 397;
const POINT_COUNT: usize = 99_260;

struct FixtureSnapshot {
    objects: Vec<PreClosingPrintObject>,
    raw_face_order: Vec<u8>,
    config_block: Vec<u8>,
}

#[test]
fn task22f_canonical_encoder_matches_independent_handwritten_nested_empty_vector() {
    let mut output = b"ARES22F\0".to_vec();
    put_u64(&mut output, 1);
    put_u64(&mut output, 7);
    put_u64(&mut output, 9);
    put_u64(&mut output, 2);
    put_u64(&mut output, 1);
    put_u64(&mut output, 11);
    output.extend_from_slice(&3_u32.to_le_bytes());
    output.push(2);
    put_u64(&mut output, 2);
    output.extend(encode_layer_record(0, SlicingMode::Regular, &[]));
    let nested = ExPolygon::new(
        polygon(&[(40, 40), (0, 40), (0, 0), (40, 0)]),
        vec![polygon(&[(10, 10), (10, 30), (30, 30), (30, 10)])],
    );
    output.extend(encode_layer_record(1, SlicingMode::EvenOdd, &[nested]));

    assert_eq!(output.len(), ENCODER_VECTOR_LEN);
    assert_eq!(sha256(&output), ENCODER_VECTOR_SHA256);
}

#[test]
fn task22f_ksr_pre_closing_union_matches_complete_fixed_oracle() {
    let snapshot = fixture_snapshot().unwrap();
    assert_eq!(snapshot.raw_face_order.len(), RAW_ENCODING_LEN);
    assert_eq!(sha256(&snapshot.raw_face_order), RAW_ENCODING_SHA256);
    assert_eq!(snapshot.config_block.len(), CONFIG_BLOCK_LEN);
    assert_eq!(sha256(&snapshot.config_block), CONFIG_BLOCK_SHA256);

    assert_eq!(snapshot.objects.len(), 1);
    let object = &snapshot.objects[0];
    assert_eq!(object.plan().source_object_index, 0);
    assert_eq!(object.plan().transform_index, 0);
    assert_eq!(object.plan().layers.len(), LAYER_COUNT);
    assert_eq!(object.volumes().len(), 1);
    let volume = &object.volumes()[0];
    assert_eq!(volume.source_volume_index(), 0);
    assert_eq!(volume.ordinal(), 1);
    assert_eq!(volume.volume_type(), ProjectVolumeType::ModelPart);
    assert_eq!(volume.layers().len(), LAYER_COUNT);
    assert!(
        volume
            .layers()
            .iter()
            .all(|layer| layer.mode() == SlicingMode::Regular)
    );

    assert_eq!(
        totals(&snapshot.objects),
        (CONTOUR_COUNT, HOLE_COUNT, POINT_COUNT)
    );
    let encoded = encode_pre_closing(&snapshot.objects);
    assert_eq!(encoded.len(), PRE_CLOSING_LEN);
    assert_eq!(sha256(&encoded), PRE_CLOSING_SHA256);
    assert_representative_layers(volume.layers());
}

#[tokio::test]
async fn task22f_ksr_pre_closing_is_repeatable_and_keeps_public_lifecycle_incomplete() {
    let first = fixture_snapshot().unwrap();
    let second = fixture_snapshot().unwrap();
    assert_eq!(
        encode_pre_closing(&first.objects),
        encode_pre_closing(&second.objects)
    );
    assert_eq!(sha256(ksr_project()), PROJECT_SHA256);
    assert_eq!(
        slice_project(ksr_project(), metadata()).await.unwrap_err(),
        SliceError::ProjectSlicingIncomplete
    );
}

fn fixture_snapshot() -> Result<FixtureSnapshot, SliceError> {
    let state = prepare_project_slice(ksr_project())?;
    let resolved = state.resolved;
    let config_block = state.config_block.unwrap();
    let max_gap_scaled = state.scale.checked_scale(2.0).unwrap();
    let chained = chain_project_intersections(state.intersected_objects);
    let looped = loop_project_intersections(chained, max_gap_scaled);
    let spiral_mode = resolved.views.full.process.print.spiral_mode.0;
    let slicing_mode = apply_project_slicing_modes(looped, &resolved.objects, spiral_mode)?;
    let raw_layers = slicing_mode[0].volumes()[0]
        .layers()
        .iter()
        .map(|layer| layer.looped_layer().clone())
        .collect::<Vec<LoopedLayer>>();
    let raw_face_order = encode_looped(&raw_layers, false);
    let objects = apply_project_pre_closing_unions(slicing_mode)?;
    Ok(FixtureSnapshot {
        objects,
        raw_face_order,
        config_block,
    })
}

fn assert_representative_layers(layers: &[PreClosingLayer]) {
    let first_hole = layers
        .iter()
        .position(|layer| hole_count(layer.expolygons()) != 0)
        .unwrap();
    let mut maximum_index = 0;
    let mut maximum_loops = 0;
    for (index, layer) in layers.iter().enumerate() {
        let loops = layer.expolygons().len() + hole_count(layer.expolygons());
        if loops > maximum_loops {
            maximum_loops = loops;
            maximum_index = index;
        }
    }
    assert_eq!((first_hole, maximum_index, maximum_loops), (0, 46, 41));
    for (index, expected_len, expected_sha) in [
        (
            0,
            14_913,
            "e1fd7ce4f9a013b0fcdf2d287dc7a9d37ff7b4818bfe3fa709a32c93aaef7b3c",
        ),
        (
            first_hole,
            14_913,
            "e1fd7ce4f9a013b0fcdf2d287dc7a9d37ff7b4818bfe3fa709a32c93aaef7b3c",
        ),
        (
            maximum_index,
            46_073,
            "1e3ef580ecf6989c1440280db78f94548467918b6ccd264bcdf2fb0dc9cca097",
        ),
        (
            LAYER_COUNT - 1,
            737,
            "c8822b67958531cb4b043d338b53f7329e0b00cb4f08108306763e763cd52f80",
        ),
    ] {
        let encoded = encode_layer_record(index, layers[index].mode(), layers[index].expolygons());
        assert_eq!(encoded.len(), expected_len);
        assert_eq!(sha256(&encoded), expected_sha);
    }
}

fn totals(objects: &[PreClosingPrintObject]) -> (usize, usize, usize) {
    let expolygons = objects
        .iter()
        .flat_map(PreClosingPrintObject::volumes)
        .flat_map(|volume| volume.layers())
        .flat_map(PreClosingLayer::expolygons);
    let mut contours = 0;
    let mut holes = 0;
    let mut points = 0;
    for expolygon in expolygons {
        contours += 1;
        holes += expolygon.holes().len();
        points += expolygon.contour().points().len();
        points += expolygon
            .holes()
            .iter()
            .map(|hole| hole.points().len())
            .sum::<usize>();
    }
    (contours, holes, points)
}

fn encode_pre_closing(objects: &[PreClosingPrintObject]) -> Vec<u8> {
    let mut output = b"ARES22F\0".to_vec();
    put_u64(&mut output, objects.len());
    for object in objects {
        put_u64(&mut output, object.plan().source_object_index);
        put_u64(&mut output, object.plan().transform_index);
        put_u64(&mut output, object.plan().layers.len());
        put_u64(&mut output, object.volumes().len());
        for volume in object.volumes() {
            put_u64(&mut output, volume.source_volume_index());
            output.extend_from_slice(&volume.ordinal().to_le_bytes());
            output.push(volume_type_code(volume.volume_type()));
            put_u64(&mut output, volume.layers().len());
            for (layer_index, layer) in volume.layers().iter().enumerate() {
                output.extend(encode_layer_record(
                    layer_index,
                    layer.mode(),
                    layer.expolygons(),
                ));
            }
        }
    }
    output
}

fn encode_layer_record(index: usize, mode: SlicingMode, expolygons: &[ExPolygon]) -> Vec<u8> {
    let mut output = Vec::new();
    put_u64(&mut output, index);
    output.push(mode_code(mode));
    put_u64(&mut output, expolygons.len());
    for expolygon in expolygons {
        encode_polygon(&mut output, expolygon.contour());
        put_u64(&mut output, expolygon.holes().len());
        for hole in expolygon.holes() {
            encode_polygon(&mut output, hole);
        }
    }
    output
}

fn encode_polygon(output: &mut Vec<u8>, polygon: &Polygon) {
    put_u64(output, polygon.points().len());
    for point in polygon.points() {
        output.extend_from_slice(&point.x().to_le_bytes());
        output.extend_from_slice(&point.y().to_le_bytes());
    }
}

fn hole_count(expolygons: &[ExPolygon]) -> usize {
    expolygons
        .iter()
        .map(|expolygon| expolygon.holes().len())
        .sum()
}

const fn mode_code(mode: SlicingMode) -> u8 {
    match mode {
        SlicingMode::Regular => 0,
        SlicingMode::EvenOdd => 1,
        SlicingMode::Positive => 2,
        SlicingMode::PositiveLargestContour => 3,
    }
}

const fn volume_type_code(volume_type: ProjectVolumeType) -> u8 {
    match volume_type {
        ProjectVolumeType::ModelPart => 0,
        ProjectVolumeType::NegativeVolume => 1,
        ProjectVolumeType::ParameterModifier => 2,
        ProjectVolumeType::SupportEnforcer | ProjectVolumeType::SupportBlocker => {
            unreachable!()
        }
    }
}

fn put_u64(output: &mut Vec<u8>, value: usize) {
    output.extend_from_slice(&u64::try_from(value).unwrap().to_le_bytes());
}

fn polygon(points: &[(i64, i64)]) -> Polygon {
    Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
