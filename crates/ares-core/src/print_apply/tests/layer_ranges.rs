use super::super::{LayerConfigRangeInput, NormalizedLayerRange, normalize_layer_ranges};

fn input(start: f64, end: f64, config_id: usize) -> LayerConfigRangeInput {
    LayerConfigRangeInput {
        start,
        end,
        config_id,
    }
}

fn range(start: f64, end: f64, config_id: Option<usize>) -> NormalizedLayerRange {
    NormalizedLayerRange {
        start,
        end,
        config_id,
    }
}

#[test]
fn layer_ranges_empty_input_returns_unconfigured_infinite_range() {
    assert_eq!(
        normalize_layer_ranges(&[]),
        vec![range(0.0, f64::MAX, None)]
    );
}

#[test]
fn layer_ranges_clamps_negative_start_and_appends_trailing_gap() {
    assert_eq!(
        normalize_layer_ranges(&[input(-1.0, 0.3, 7)]),
        vec![range(0.0, 0.3, Some(7)), range(0.3, f64::MAX, None)]
    );
}

#[test]
fn layer_ranges_inserts_unconfigured_gap_before_configured_range() {
    assert_eq!(
        normalize_layer_ranges(&[input(0.2, 0.5, 3)]),
        vec![
            range(0.0, 0.2, None),
            range(0.2, 0.5, Some(3)),
            range(0.5, f64::MAX, None),
        ]
    );
}

#[test]
fn layer_ranges_extends_trailing_gap_to_infinite_when_configured_range_is_tiny() {
    assert_eq!(
        normalize_layer_ranges(&[input(0.2, 0.20005, 3)]),
        vec![range(0.0, f64::MAX, None)]
    );
}

#[test]
fn layer_ranges_skips_ranges_covered_by_last_z() {
    assert_eq!(
        normalize_layer_ranges(&[input(0.0, 0.5, 1), input(0.1, 0.4, 2), input(0.5, 0.8, 3),]),
        vec![
            range(0.0, 0.5, Some(1)),
            range(0.5, 0.8, Some(3)),
            range(0.8, f64::MAX, None),
        ]
    );
}

#[test]
fn layer_ranges_uses_orca_epsilon_to_skip_tiny_configured_range() {
    assert_eq!(
        normalize_layer_ranges(&[input(0.0, 0.00005, 1)]),
        vec![range(0.0, f64::MAX, None)]
    );
}

#[test]
fn layer_ranges_uses_orca_epsilon_to_skip_tiny_gap() {
    assert_eq!(
        normalize_layer_ranges(&[input(0.0, 0.5, 1), input(0.50005, 0.7, 2)]),
        vec![
            range(0.0, 0.5, Some(1)),
            range(0.5, 0.7, Some(2)),
            range(0.7, f64::MAX, None),
        ]
    );
}

#[test]
fn layer_ranges_preserves_gap_beyond_orca_epsilon() {
    assert_eq!(
        normalize_layer_ranges(&[input(0.0, 0.5, 1), input(0.5002, 0.7, 2)]),
        vec![
            range(0.0, 0.5, Some(1)),
            range(0.5, 0.5002, None),
            range(0.5002, 0.7, Some(2)),
            range(0.7, f64::MAX, None),
        ]
    );
}
