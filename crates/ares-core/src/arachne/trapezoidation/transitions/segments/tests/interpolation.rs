use crate::arachne::beading::base::Beading;

use super::super::interpolate_beading;

#[test]
fn task22o176_interpolation_preserves_source_f32_complement() {
    let beading = Beading {
        total_thickness: 1_000_000,
        bead_widths: vec![377_079],
        toolpath_locations: vec![565_618],
        left_over: 0,
    };

    let result = interpolate_beading(&beading, f64::from(0.349_575_f32), &beading);

    assert_eq!(result.bead_widths, [377_078]);
    assert_eq!(result.toolpath_locations, [565_617]);
}

#[test]
fn task22o206_interpolation_multiplies_right_term_as_source_float() {
    let left = Beading {
        total_thickness: 2_386_356,
        bead_widths: vec![377_079],
        toolpath_locations: vec![188_539],
        left_over: 0,
    };
    let right = Beading {
        total_thickness: 2_059_210,
        ..left.clone()
    };

    let result = interpolate_beading(&left, f64::from(0.404_002_5_f32), &right);

    assert_eq!(result.bead_widths, [377_078]);
    assert_eq!(result.toolpath_locations, [188_538]);
}
