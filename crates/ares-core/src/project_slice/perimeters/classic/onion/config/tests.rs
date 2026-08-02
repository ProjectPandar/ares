use crate::SliceError;

use super::{validate_densities, validate_density};

#[test]
fn task22o3_density_uses_the_source_int_local_after_domain_validation() {
    for (value, expected) in [
        (0.0, 0),
        (f64::MIN_POSITIVE, 0),
        (0.999_999, 0),
        (1.0, 1),
        (12.5, 12),
        (100.0, 100),
    ] {
        assert_eq!(validate_density(value).unwrap(), expected);
    }
}

#[test]
fn task22o3_later_invalid_record_rejects_the_whole_density_sequence() {
    assert_eq!(
        validate_densities([Some(20.0), None, Some(100.000_000_1)]),
        Err(SliceError::InvalidInput(
            "invalid Orca option sparse_infill_density".to_owned(),
        ))
    );
}

#[test]
fn task22o3_density_rejects_negative_over_domain_and_non_finite_values() {
    for value in [
        -f64::MIN_POSITIVE,
        -1.0,
        100.000_000_1,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NAN,
    ] {
        assert_eq!(
            validate_density(value),
            Err(SliceError::InvalidInput(
                "invalid Orca option sparse_infill_density".to_owned(),
            ))
        );
    }
}
