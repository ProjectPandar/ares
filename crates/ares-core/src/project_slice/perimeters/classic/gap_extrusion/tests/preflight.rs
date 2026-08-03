use crate::{SliceError, geometry::CoordinateScale};

use super::super::preflight::validate_threshold;

#[test]
fn task22o14_threshold_validation_is_literal_at_both_scales() {
    assert_eq!(
        validate_threshold(0.000_100_5, CoordinateScale::Normal)
            .unwrap()
            .to_bits(),
        0x4059_2000_0000_0001,
    );
    assert_eq!(
        validate_threshold(0.000_100_5, CoordinateScale::LargeBed)
            .unwrap()
            .to_bits(),
        0x4024_1999_9999_9999,
    );
}

#[test]
fn task22o14_threshold_validation_rejects_every_invalid_f64() {
    for value in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            validate_threshold(value, CoordinateScale::Normal),
            Err(SliceError::InvalidInput(
                "invalid Orca option filter_out_gap_fill".to_owned(),
            )),
        );
    }
}
