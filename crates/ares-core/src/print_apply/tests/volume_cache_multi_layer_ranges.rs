use super::super::{
    mesh_state::StagedLayerHeightRange,
    volume_cache_state::staged_update_volume_bboxes_multi_layer_expanded_ranges,
};

fn range(first: f64, second: f64) -> StagedLayerHeightRange {
    StagedLayerHeightRange::new(first, second)
}

#[test]
fn update_volume_bboxes_multi_layer_expanded_ranges_expands_single_range() {
    let ranges =
        staged_update_volume_bboxes_multi_layer_expanded_ranges(&[range(0.25, 0.5)], 0.125);

    assert_eq!(ranges, vec![range(0.125, 0.625)]);
}

#[test]
fn update_volume_bboxes_multi_layer_expanded_ranges_preserves_input_order() {
    let ranges = staged_update_volume_bboxes_multi_layer_expanded_ranges(
        &[range(0.25, 0.5), range(0.75, 1.0), range(1.25, 1.5)],
        0.125,
    );

    assert_eq!(
        ranges,
        vec![
            range(0.125, 0.625),
            range(0.625, 1.125),
            range(1.125, 1.625),
        ]
    );
}

#[test]
fn update_volume_bboxes_multi_layer_expanded_ranges_accepts_empty_input() {
    let ranges = staged_update_volume_bboxes_multi_layer_expanded_ranges(&[], 0.125);

    assert!(ranges.is_empty());
}

#[test]
fn update_volume_bboxes_multi_layer_expanded_ranges_zero_epsilon_copies_ranges() {
    let ranges = staged_update_volume_bboxes_multi_layer_expanded_ranges(
        &[range(0.25, 0.5), range(0.75, 1.0)],
        0.0,
    );

    assert_eq!(ranges, vec![range(0.25, 0.5), range(0.75, 1.0)]);
}

#[test]
fn update_volume_bboxes_multi_layer_expanded_ranges_expands_negative_bounds_without_clamping() {
    let ranges =
        staged_update_volume_bboxes_multi_layer_expanded_ranges(&[range(-0.5, 0.25)], 0.25);

    assert_eq!(ranges, vec![range(-0.75, 0.5)]);
}
