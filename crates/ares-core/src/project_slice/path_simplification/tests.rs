use crate::ExtrusionRole;

use super::fill_tolerance;

#[test]
fn task22o213_sparse_infill_uses_source_coarse_resolution() {
    assert_eq!(fill_tolerance(ExtrusionRole::InternalInfill, 0.0125), 0.04);
    assert_eq!(fill_tolerance(ExtrusionRole::SolidInfill, 0.0125), 0.0125);
}
