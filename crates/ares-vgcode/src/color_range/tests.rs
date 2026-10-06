use super::*;

fn range(range_type: ColorRangeType, values: &[f32]) -> ColorRange {
    let mut range = ColorRange::new(range_type);
    for &value in values {
        range.update(value);
    }
    range
}

fn approx(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= 1e-6 * expected.abs().max(1.0),
        "{actual} != {expected}"
    );
}

/// `m_range{FLT_MAX, -FLT_MAX}` before any update: `std::clamp` keeps the low
/// bound and the step is negative, so the color is the first one.
#[test]
fn unset_range_colors_with_first_palette_entry() {
    let range = ColorRange::new(ColorRangeType::Linear);
    assert_eq!(range.get_range(), [f32::MAX, -f32::MAX]);
    assert_eq!(range.get_color_at(5.0), DEFAULT_RANGES_COLORS[0]);
}

/// One value: `get_values` has one item and the zero step maps every value
/// (clamped) to the first color (`ColorRange.cpp:57-80, 91-94`).
#[test]
fn single_value_range() {
    let range = range(ColorRangeType::Linear, &[0.2, 0.2, 0.2]);
    assert_eq!(range.get_values(), [0.2]);
    assert_eq!(range.get_color_at(0.2), DEFAULT_RANGES_COLORS[0]);
    assert_eq!(range.get_color_at(100.0), DEFAULT_RANGES_COLORS[0]);
}

/// `update` counts values that differ from the current bounds, so repeated
/// bounds keep the two-value case (`ColorRange.cpp:123-130, 95-99`).
#[test]
fn repeated_bounds_keep_two_values() {
    let range = range(ColorRangeType::Linear, &[0.0, 10.0, 0.0, 10.0]);
    assert_eq!(range.get_values(), [0.0, 10.0]);
    assert_eq!(range.get_color_at(10.0), DEFAULT_RANGES_COLORS[10]);
}

/// Linear step `(max - min) / (palette - 1)`; colors interpolate between the
/// bins around the value and channels truncate (`Types.cpp:29-40`).
#[test]
fn linear_values_and_interpolation() {
    let range = range(ColorRangeType::Linear, &[0.0, 10.0, 5.0]);
    assert_eq!(
        range.get_values(),
        (0..11).map(|i| i as f32).collect::<Vec<_>>()
    );
    assert_eq!(range.get_color_at(2.5), [16, 175, 80]);
    assert_eq!(range.get_color_at(-3.0), DEFAULT_RANGES_COLORS[0]);
}

/// Logarithmic range with a zero minimum: the step is 0 (`ColorRange.cpp:28-30`),
/// so every value takes the first color and `exp(log(0))` values are 0.
#[test]
fn logarithmic_range_with_zero_minimum() {
    let range = range(ColorRangeType::Logarithmic, &[0.0, 5.0, 10.0]);
    assert_eq!(range.get_color_at(7.0), DEFAULT_RANGES_COLORS[0]);
    assert_eq!(range.get_values(), [0.0; 11]);
}

/// Logarithmic values are geometric between the bounds (`ColorRange.cpp:108`).
#[test]
fn logarithmic_values_are_geometric() {
    let range = range(ColorRangeType::Logarithmic, &[1.0, 1024.0, 32.0]);
    for (i, value) in range.get_values().into_iter().enumerate() {
        approx(value, (1 << i) as f32);
    }
    assert_eq!(range.get_color_at(0.5), DEFAULT_RANGES_COLORS[0]);
}

/// `set_palette` ignores palettes with fewer than two colors.
#[test]
fn single_color_palette_is_ignored() {
    let mut range = ColorRange::new(ColorRangeType::Linear);
    range.set_palette(vec![[1, 2, 3]]);
    assert_eq!(range.get_palette(), DEFAULT_RANGES_COLORS);
}

/// `round_to_bin` (`ViewerImpl.cpp:47-63`): two significant digits, the scale
/// picked by the 0.095 / 0.0095 / ... thresholds, at most 1e6.
#[test]
fn round_to_bin_thresholds() {
    approx(round_to_bin(0.123_456), 0.12);
    approx(round_to_bin(0.095), 0.1);
    approx(round_to_bin(0.0949), 0.095);
    approx(round_to_bin(0.000_012_34), 0.000_012);
    assert_eq!(round_to_bin(1e-7), 0.0);
    assert_eq!(round_to_bin(0.0), 0.0);
    approx(round_to_bin(215.0), 215.0);
}
