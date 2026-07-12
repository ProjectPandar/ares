use super::super::{NormalizedLayerRange, layer_range_config_id};

fn range(start: f64, end: f64, config_id: Option<usize>) -> NormalizedLayerRange {
    NormalizedLayerRange {
        start,
        end,
        config_id,
    }
}

#[test]
fn layer_range_lookup_returns_configured_match() {
    let ranges = [range(0.0, 0.2, None), range(0.2, 0.5, Some(7))];

    assert_eq!(layer_range_config_id(&ranges, (0.2, 0.5)), Some(Some(7)));
}

#[test]
fn layer_range_lookup_returns_matched_unconfigured_range() {
    let ranges = [range(0.0, 0.2, None), range(0.2, f64::MAX, None)];

    assert_eq!(layer_range_config_id(&ranges, (0.2, f64::MAX)), Some(None));
}

#[test]
fn layer_range_lookup_returns_none_when_no_lower_bound_candidate_exists() {
    let ranges = [range(0.0, 0.2, None), range(0.2, 0.5, Some(7))];

    assert_eq!(layer_range_config_id(&ranges, (0.6, 0.8)), None);
}

#[test]
fn layer_range_lookup_returns_none_when_start_mismatch_exceeds_epsilon() {
    let ranges = [range(0.2, 0.5, Some(7))];

    assert_eq!(layer_range_config_id(&ranges, (0.1998, 0.5)), None);
}

#[test]
fn layer_range_lookup_returns_none_when_end_mismatch_exceeds_epsilon() {
    let ranges = [range(0.2, 0.5, Some(7))];

    assert_eq!(layer_range_config_id(&ranges, (0.2, 0.4998)), None);
}

#[test]
fn layer_range_lookup_matches_within_orca_epsilon() {
    let ranges = [range(0.2, 0.5, Some(7))];

    assert_eq!(
        layer_range_config_id(&ranges, (0.20005, 0.50005)),
        Some(Some(7))
    );
}
