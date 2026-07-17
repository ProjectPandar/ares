use sha2::{Digest, Sha256};

use crate::{SliceError, slice_project};

use super::{
    super::state::prepare_project_slice,
    support::{ksr_project, metadata},
};

const CONFIG_BLOCK_SHA256: &str =
    "b33c979097a4900700d1e5dfcaa16f1454a79ce5fec48da7eb9458cfa2fdeeb8";

#[tokio::test]
async fn task22a_ksr_fixture_plans_exact_460_layers_from_3mf_only() {
    let state = prepare_project_slice(ksr_project()).unwrap();
    assert_eq!(state.project.objects().len(), 1);
    assert_eq!(state.resolved.objects.len(), 1);
    assert_eq!(state.intersected_objects.len(), 1);

    let plan = &state.intersected_objects[0].plan;
    assert_eq!(plan.source_object_index, 0);
    assert_eq!(plan.transform_index, 0);
    assert_eq!(
        plan.source_object_index,
        state.resolved.objects[0].source_object_index
    );
    assert_eq!(state.resolved.objects[0].print_objects.len(), 1);
    assert_eq!(plan.layers.len(), 460);

    let first = &plan.layers[0];
    assert_eq!(
        (first.id, first.height, first.print_z, first.slice_z),
        (0, 0.2, 0.2, 0.1)
    );
    let last = plan.layers.last().unwrap();
    assert_eq!(last.id, 459);
    assert_eq!(last.print_z.to_bits(), 0x4057_0000_0000_0036);
    assert!((last.print_z - 92.0).abs() < 1e-9);

    let mut previous_print_z = 0.0;
    let mut previous_slice_z = None;
    let mut expected_lo: f64 = 0.0;
    let mut expected_hi: f64 = 0.2;
    for (index, layer) in plan.layers.iter().enumerate() {
        assert_eq!(layer.id, index);
        assert!(layer.height.is_finite());
        assert!(layer.height > 0.0);
        assert!(layer.print_z.is_finite());
        assert!(layer.print_z > previous_print_z);
        assert!(layer.slice_z.is_finite());
        if let Some(previous) = previous_slice_z {
            assert!(layer.slice_z > previous);
        }
        assert_eq!(layer.height, layer.print_z - previous_print_z);
        assert_eq!(layer.slice_z, 0.5 * (previous_print_z + layer.print_z));
        assert_eq!(layer.print_z.to_bits(), expected_hi.to_bits());
        assert_eq!(
            layer.height.to_bits(),
            (expected_hi - expected_lo).to_bits()
        );
        assert_eq!(
            layer.slice_z.to_bits(),
            (0.5 * (expected_lo + expected_hi)).to_bits()
        );
        previous_print_z = layer.print_z;
        previous_slice_z = Some(layer.slice_z);
        expected_lo = expected_hi;
        expected_hi += 0.2;
    }

    assert_eq!(
        slice_project(ksr_project(), metadata()).await.unwrap_err(),
        SliceError::ProjectSlicingIncomplete
    );
}

#[test]
fn task22a_ksr_fixture_plan_is_deterministic_and_config_block_unchanged() {
    let first = prepare_project_slice(ksr_project()).unwrap();
    let second = prepare_project_slice(ksr_project()).unwrap();

    assert_eq!(first.intersected_objects, second.intersected_objects);
    assert_eq!(first.resolved, second.resolved);
    assert_eq!(first.config_block, second.config_block);

    let block = first.config_block.as_deref().unwrap();
    assert_eq!(block.len(), 49_004);
    assert_eq!(sha256(block), CONFIG_BLOCK_SHA256);
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
