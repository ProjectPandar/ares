use std::cell::Cell;

use crate::{
    ProjectVolumeType, SliceError,
    geometry::CoordinateScale,
    mesh_slicer::{EndpointReference, FacetEdgeType},
    slice_project,
};

use super::super::super::{
    state::{ProjectSliceState, prepare_project_slice},
    tests::support::{KsrArchive, ksr_project, metadata},
};
use super::{
    CONFIG_BLOCK_SHA256, FACE_ORDER_SHA256, SEMANTIC_SHA256,
    encoding::{ObjectView, VolumeView, encode},
    sha256,
};

const FIRST_VERTEX: &str = r#"<vertex x="17.6525421" y="-26.3965759" z="-45.5"/>"#;
const MUTATED_FIRST_VERTEX: &str = r#"<vertex x="17.7525421" y="-26.3965759" z="-45.5"/>"#;
const NORMAL_PRINTABLE_AREA: &str = concat!(
    "\t\"printable_area\": [\r\n",
    "\t\t\"0x0\",\r\n",
    "\t\t\"256x0\",\r\n",
    "\t\t\"256x256\",\r\n",
    "\t\t\"0x256\"\r\n",
    "\t]",
);
const LARGE_PRINTABLE_AREA: &str = concat!(
    "\t\"printable_area\": [\r\n",
    "\t\t\"0x0\",\r\n",
    "\t\t\"2148x0\",\r\n",
    "\t\t\"2148x256\",\r\n",
    "\t\t\"0x256\"\r\n",
    "\t]",
);

#[test]
fn task22b_private_state_owns_plan_inside_intersections_and_builds_once() {
    let calls = Cell::new(0);
    let input = CountingInput {
        bytes: ksr_project(),
        calls: &calls,
    };
    let ProjectSliceState {
        project,
        resolved,
        config_block,
        scale,
        intersected_objects,
    } = prepare_project_slice(input).unwrap();

    assert_eq!(calls.get(), 1);
    assert_eq!(project.objects().len(), 1);
    assert_eq!(resolved.objects.len(), 1);
    assert_eq!(resolved.print_object_count, 1);
    assert_eq!(scale, CoordinateScale::Normal);

    let block = config_block.unwrap();
    assert_eq!(block.len(), 49_004);
    assert_eq!(sha256(&block), CONFIG_BLOCK_SHA256);
    assert_eq!(occurrences(&block, b"; CONFIG_BLOCK_START\n"), 1);
    assert_eq!(occurrences(&block, b"; CONFIG_BLOCK_END\n\n"), 1);

    let mut objects = intersected_objects.into_iter();
    let object = objects.next().unwrap();
    assert!(objects.next().is_none());
    let (plan, volumes) = object.into_parts();
    assert_eq!(plan.source_object_index, 0);
    assert_eq!(plan.transform_index, 0);
    assert_eq!(plan.layers.len(), 460);
    assert_eq!(
        plan.source_object_index,
        resolved.objects[0].source_object_index
    );

    let mut volumes = volumes.into_iter();
    let volume = volumes.next().unwrap();
    assert!(volumes.next().is_none());
    let (source_volume_index, ordinal, volume_type, layers) = volume.into_parts();
    assert_eq!(source_volume_index, 0);
    assert_eq!(ordinal, 1);
    assert_eq!(volume_type, ProjectVolumeType::ModelPart);
    assert_eq!(layers.len(), plan.layers.len());
}

#[tokio::test]
async fn task22b_ksr_fixture_is_repeatable_config_unchanged_and_publicly_incomplete() {
    let first = prepare_project_slice(ksr_project()).unwrap();
    let second = prepare_project_slice(ksr_project()).unwrap();

    assert_project_eq(&first, &second);
    assert_eq!(first.resolved, second.resolved);
    assert_eq!(first.config_block, second.config_block);
    assert_eq!(first.intersected_objects, second.intersected_objects);

    let first_config = first.config_block.as_deref().unwrap();
    let second_config = second.config_block.as_deref().unwrap();
    assert_eq!(first_config.len(), 49_004);
    assert_eq!(second_config.len(), 49_004);
    assert_eq!(sha256(first_config), CONFIG_BLOCK_SHA256);
    assert_eq!(sha256(second_config), CONFIG_BLOCK_SHA256);

    let (first_semantic, first_face_order) = encodings(&first);
    let (second_semantic, second_face_order) = encodings(&second);
    assert_eq!(first_semantic, second_semantic);
    assert_eq!(first_face_order, second_face_order);
    assert_eq!(first_semantic.len(), 5_012_035);
    assert_eq!(first_face_order.len(), 5_012_035);
    assert_eq!(sha256(&first_semantic), SEMANTIC_SHA256);
    assert_eq!(sha256(&second_semantic), SEMANTIC_SHA256);
    assert_eq!(sha256(&first_face_order), FACE_ORDER_SHA256);
    assert_eq!(sha256(&second_face_order), FACE_ORDER_SHA256);

    for _ in 0..2 {
        assert_eq!(
            slice_project(ksr_project(), metadata()).await.unwrap_err(),
            SliceError::ProjectSlicingIncomplete
        );
    }
}

#[test]
fn task22b_anti_hardcoding_vertex_mutation_changes_semantic_digest() {
    let original = fingerprint(ksr_project());
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "3D/Objects/ksr_fdmtest_v4.drc_2.model",
        FIRST_VERTEX,
        MUTATED_FIRST_VERTEX,
    );
    let mutated = fingerprint(&archive.bytes());

    assert_eq!(original.semantic_digest, SEMANTIC_SHA256);
    assert_eq!(
        original.structure,
        RawStructure {
            object_count: 1,
            source_object_index: 0,
            transform_index: 0,
            volume_count: 1,
            ordinal: 1,
            volume_type: ProjectVolumeType::ModelPart,
            layer_count: 460,
            line_count: 116_472,
            semantic_len: 5_012_035,
        }
    );
    assert_eq!(mutated.structure, original.structure);
    assert_eq!(mutated.scale, original.scale);
    assert_ne!(mutated.semantic_digest, original.semantic_digest);
}

#[test]
fn task22b_anti_hardcoding_printable_area_mutation_changes_scale_and_coordinates() {
    let original = fingerprint(ksr_project());
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        NORMAL_PRINTABLE_AREA,
        LARGE_PRINTABLE_AREA,
    );
    let mutated = fingerprint(&archive.bytes());

    assert_eq!(original.scale, CoordinateScale::Normal);
    assert_eq!(mutated.scale, CoordinateScale::LargeBed);
    assert_eq!(mutated.structure, original.structure);
    assert_eq!(mutated.first_provenance, original.first_provenance);
    assert_ne!(mutated.first_coordinates, original.first_coordinates);
    assert_ne!(mutated.semantic_digest, original.semantic_digest);
}

#[derive(Debug, Eq, PartialEq)]
struct RawFingerprint {
    structure: RawStructure,
    scale: CoordinateScale,
    first_coordinates: [i64; 4],
    first_provenance: (EndpointReference, EndpointReference, FacetEdgeType),
    semantic_digest: String,
}

#[derive(Debug, Eq, PartialEq)]
struct RawStructure {
    object_count: usize,
    source_object_index: usize,
    transform_index: usize,
    volume_count: usize,
    ordinal: u32,
    volume_type: ProjectVolumeType,
    layer_count: usize,
    line_count: usize,
    semantic_len: usize,
}

fn fingerprint(bytes: &[u8]) -> RawFingerprint {
    let state = prepare_project_slice(bytes).unwrap();
    let scale = CoordinateScale::from_printable_area(
        &state.resolved.views.full.printer.remaining.printable_area,
    );
    let object = &state.intersected_objects[0];
    let volume = &object.volumes()[0];
    let first = volume.layers().iter().flatten().next().unwrap();
    let a = first.a();
    let b = first.b();
    let a_point = a.point();
    let b_point = b.point();
    let semantic = encodings(&state).0;
    RawFingerprint {
        structure: RawStructure {
            object_count: state.intersected_objects.len(),
            source_object_index: object.plan.source_object_index,
            transform_index: object.plan.transform_index,
            volume_count: object.volumes().len(),
            ordinal: volume.ordinal(),
            volume_type: volume.volume_type(),
            layer_count: volume.layers().len(),
            line_count: volume.layers().iter().map(Vec::len).sum(),
            semantic_len: semantic.len(),
        },
        scale,
        first_coordinates: [a_point.x(), a_point.y(), b_point.x(), b_point.y()],
        first_provenance: (a.reference(), b.reference(), first.edge_type()),
        semantic_digest: sha256(&semantic),
    }
}

fn encodings(state: &ProjectSliceState) -> (Vec<u8>, Vec<u8>) {
    let views = state
        .intersected_objects
        .iter()
        .map(|object| ObjectView {
            source_object_index: object.plan.source_object_index,
            transform_index: object.plan.transform_index,
            volumes: object
                .volumes()
                .iter()
                .map(|volume| VolumeView {
                    ordinal: volume.ordinal(),
                    volume_type: volume.volume_type(),
                    layers: volume.layers(),
                })
                .collect(),
        })
        .collect::<Vec<_>>();
    (encode(&views, true), encode(&views, false))
}

fn assert_project_eq(first: &ProjectSliceState, second: &ProjectSliceState) {
    assert_eq!(first.project.models(), second.project.models());
    assert_eq!(first.project.objects(), second.project.objects());
    assert_eq!(first.project.plates(), second.project.plates());
    assert_eq!(first.project.settings(), second.project.settings());

    let first_documents = first.project.documents();
    let second_documents = second.project.documents();
    assert_eq!(
        first_documents.model_settings,
        second_documents.model_settings
    );
    assert_eq!(first_documents.slice_info, second_documents.slice_info);
    assert_eq!(
        first_documents.filament_sequences,
        second_documents.filament_sequences
    );
    assert_eq!(
        first_documents.plate_documents,
        second_documents.plate_documents
    );
    assert_eq!(
        first_documents.has_painted_layer_height_profile,
        second_documents.has_painted_layer_height_profile
    );
}

struct CountingInput<'a> {
    bytes: &'a [u8],
    calls: &'a Cell<usize>,
}

impl AsRef<[u8]> for CountingInput<'_> {
    fn as_ref(&self) -> &[u8] {
        self.calls.set(self.calls.get() + 1);
        self.bytes
    }
}

fn occurrences(bytes: &[u8], needle: &[u8]) -> usize {
    bytes
        .windows(needle.len())
        .filter(|window| *window == needle)
        .count()
}
