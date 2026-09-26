use super::*;

#[test]
fn quartercubic_shifts_preserve_fractional_scaled_inputs() {
    // C++ FillQuarterCubic arithmetic with Boost's float constant, spacing
    // 0.40707963705062866, density 0.15f, multiline 1. The source period is
    // 10855456, with truncation only after division by density.
    for (z, dx1, dx2) in [
        (1.2, 848_528, 2_330_962),
        (1.2000009, 848_528, 2_330_961),
        (9.8, 3_925_810, 3_750_156),
    ] {
        let sweeps = quartercubic_sweeps(0.40707963705062866, 1, 0.15, z, CoordinateScale::Normal);
        let dx1 = (f64::from(dx1) * 1e-6) as f32;
        let dx2 = (f64::from(dx2) * 1e-6) as f32;
        assert_eq!(sweeps.map(|sweep| sweep.shift), [dx1, -dx1, dx2, -dx2]);
    }
}
