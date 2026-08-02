use crate::{FloatOrPercent, Percent, geometry::CoordinateScale};

use super::{absolute, scaled_option};

#[test]
fn task22o2_percent_width_stays_f64_until_clipper_call() {
    let absolute = absolute(
        "min_width_top_surface",
        FloatOrPercent::Percent(Percent(137.123_456_789)),
        0.419_999_986_886_978_15,
    )
    .unwrap();
    let scaled = scaled_option(CoordinateScale::Normal, "min_width_top_surface", absolute).unwrap();

    assert_eq!(
        scaled.to_bits(),
        (absolute / CoordinateScale::Normal.factor()).to_bits()
    );
    assert_ne!(scaled, scaled.trunc());
}

#[test]
fn task22o2_sparse_width_scaling_does_not_truncate_large_bed_units() {
    let width = 0.451_234_567_89;
    let scaled =
        scaled_option(CoordinateScale::LargeBed, "sparse_infill_line_width", width).unwrap();

    assert_eq!(
        scaled.to_bits(),
        (width / CoordinateScale::LargeBed.factor()).to_bits()
    );
    assert_ne!(scaled, scaled.trunc());
}
