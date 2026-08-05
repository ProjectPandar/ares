use crate::project_slice::prepare_infill::vertical_shell_regularization::regularize::{
    min_perimeter_infill_spacing, radii,
};

#[test]
fn task22o22_radii_preserve_source_f32_expression_order() {
    for spacing in [1_i64, 400_001, 16_777_217, 1_000_000_001] {
        let minimum = (spacing as f32) * 1.05_f32;
        let actual = radii(spacing);
        assert_eq!(
            min_perimeter_infill_spacing(spacing).to_bits(),
            minimum.to_bits()
        );
        assert_eq!(
            actual.narrow_ensure.to_bits(),
            (0.5_f32 * 0.65_f32 * minimum).to_bits()
        );
        assert_eq!(
            actual.narrow_sparse.to_bits(),
            (0.5_f32 * 1.2_f32 * minimum).to_bits()
        );
        assert_eq!(actual.tiny_overlap.to_bits(), (0.2_f32 * minimum).to_bits());
        assert_eq!(
            (-actual.narrow_ensure).to_bits(),
            (-(0.5_f32 * 0.65_f32 * minimum)).to_bits()
        );
        assert_eq!(
            (actual.narrow_ensure + actual.narrow_sparse).to_bits(),
            ((0.5_f32 * 0.65_f32 * minimum) + (0.5_f32 * 1.2_f32 * minimum)).to_bits()
        );
        assert_eq!(
            (-(actual.narrow_sparse - actual.tiny_overlap)).to_bits(),
            (-((0.5_f32 * 1.2_f32 * minimum) - (0.2_f32 * minimum))).to_bits()
        );
    }
}
