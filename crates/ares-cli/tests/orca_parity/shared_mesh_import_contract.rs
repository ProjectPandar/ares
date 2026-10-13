//! bbs_3mf.cpp:2058-2082/3851-3872/4867-4891, approved SPEC415d796bd.
use crate::shared_mesh_centering::compare_scope;

const MANIFEST: &str =
    include_str!("../../../../tests/parity/shared-mesh-centering/import-contract-provenance.json");
const DEFAULT: &[u8] =
    include_bytes!("../../../../tests/parity/shared-mesh-centering/default-scope.3mf");

#[test]
fn c2_contract_unassigned_owner() {
    let bytes =
        include_bytes!("../../../../tests/parity/shared-mesh-centering/unassigned-owner.3mf");
    compare_scope("unassigned-owner", bytes, 1, Some(1), MANIFEST);
    let project = ares_core::load_project(bytes).unwrap();
    let selected = project.select_plate(Some(1)).unwrap();
    assert_eq!(
        selected
            .objects()
            .iter()
            .map(|object| object.id())
            .collect::<Vec<_>>(),
        vec![3],
        "explicit importer membership must exclude the unassigned owner"
    );
}

#[test]
fn c2_contract_default_all_import_scope() {
    compare_scope("default-scope", DEFAULT, 0, None, MANIFEST);
    assert_selected_width(None, 10.0);
}

#[test]
fn c2_contract_explicit_plate_import_scope() {
    compare_scope("default-scope", DEFAULT, 1, Some(1), MANIFEST);
    assert_selected_width(Some(1), 16.0);
}

fn assert_selected_width(plate: Option<u32>, expected: f64) {
    let project = ares_core::load_project(DEFAULT).unwrap();
    let selected = project.select_plate(plate).unwrap();
    assert_eq!(selected.objects().len(), 1);
    let object = selected.objects().first().unwrap();
    assert_eq!(object.id(), 3);
    assert_eq!(object.volumes().len(), 1);
    let vertices = object.volumes().first().unwrap().mesh().vertices();
    let min = vertices
        .iter()
        .map(|point| point.x)
        .fold(f64::INFINITY, f64::min);
    let max = vertices
        .iter()
        .map(|point| point.x)
        .fold(f64::NEG_INFINITY, f64::max);
    assert_eq!(
        max - min,
        expected,
        "Native all-import vs plate-import owner width"
    );
    let retained = selected.select_plate(None).unwrap();
    let retained_object = retained.objects().first().unwrap();
    let retained_vertices = retained_object.volumes().first().unwrap().mesh().vertices();
    assert_eq!(retained_vertices, vertices);
}

#[test]
fn c2_contract_literal_id_membership() {
    compare_scope(
        "nested-bfs",
        include_bytes!("../../../../tests/parity/shared-mesh-centering/nested-bfs.3mf"),
        1,
        Some(1),
        include_str!("../../../../tests/parity/shared-mesh-centering/provenance.json"),
    );
}

#[test]
fn c2_contract_local_id_bfs_control() {
    compare_scope(
        "local-id-bfs",
        include_bytes!("../../../../tests/parity/shared-mesh-centering/local-id-bfs.3mf"),
        1,
        Some(1),
        MANIFEST,
    );
}
