use super::*;

fn square(min: Coord, max: Coord) -> Polygon {
    Polygon::new(vec![
        Point::new(min, min),
        Point::new(max, min),
        Point::new(max, max),
        Point::new(min, max),
    ])
}

/// A grid-aligned square passes through `extract_support` as a single
/// snapped island (`SupportMaterial.cpp:739-836`): the simplified
/// contour contains the shrunk input sample, so the island survives
/// the sample filter.
#[test]
fn grid_aligned_square_survives_extract() {
    let support = vec![square(2000, 8000)];
    let params = SupportGridParams::new(1000, 400);
    let pattern = SupportGridPattern::new(&support, &[], &params).unwrap();

    let extracted = pattern.extract_support(&support, &[], params.expansion_to_slice, true);

    assert_eq!(extracted.len(), 1);
    let points = extracted[0].points();
    assert_eq!(points.len(), 4, "grid-aligned corners only");
}

/// Islands that lose their sample after trimming are dropped
/// (`:776-820` keep-islands-with-samples rule).
#[test]
fn trimmed_away_island_is_dropped() {
    let support = vec![square(2000, 8000)];
    let params = SupportGridParams::new(1000, 400);
    let pattern = SupportGridPattern::new(&support, &[], &params).unwrap();

    // Trim everything away: no island contains a sample.
    let trimming = vec![square(-10000, 10000)];
    let extracted = pattern.extract_support(&support, &trimming, params.expansion_to_slice, true);

    assert!(extracted.is_empty());
}

/// `island_samples` returns up to 4 points per expolygon, taken from
/// the first `-20` shrunk contour (`SupportMaterial.cpp:1070-1095`).
#[test]
fn island_samples_takes_up_to_four_points() {
    let expolygon = ExPolygon::new(square(1000, 9000), Vec::new());
    let samples = island_samples(std::slice::from_ref(&expolygon));

    assert_eq!(samples.len(), 4);
    for sample in &samples {
        assert!(square(1000, 9000).contains(sample));
    }
}
