use super::coverage_delta;
use crate::geometry::CoordinateScale;

#[test]
fn source_coverage_truncates_scaled_spacing_before_halving() {
    assert_eq!(
        coverage_delta(0.45, 0.2, CoordinateScale::Normal),
        203_549.5
    );
    assert_eq!(
        coverage_delta(0.45, 0.2, CoordinateScale::LargeBed),
        20_363.5
    );
}
