use super::{MotionOptions, PathProperties};

fn properties(feature: &'static str, is_perimeter: bool) -> PathProperties<'static> {
    PathProperties {
        contour: None,
        mm3_per_mm: 0.08,
        width: 0.45,
        height: 0.2,
        feature,
        is_perimeter,
        end_clip: 0.0,
        fitting: &[],
        slope: None,
        inside_loop: false,
    }
}

#[test]
fn first_layer_uses_wall_and_infill_speed_classes() {
    let options = MotionOptions {
        initial_layer_speed: 30.0,
        initial_layer_infill_speed: 60.0,
        support_speed: 80.0,
        ..MotionOptions::default()
    };

    assert_eq!(
        properties("Outer wall", true).speed(&options, 0, false, None),
        30.0
    );
    assert_eq!(
        properties("Brim", false).speed(&options, 0, false, None),
        60.0
    );
    assert_eq!(
        properties("Sparse infill", false).speed(&options, 0, false, None),
        60.0
    );
}

#[test]
fn resonance_avoidance_clamps_outer_wall_speed_band() {
    let options = MotionOptions {
        resonance_avoidance: Some((40.0, 90.0)),
        ..MotionOptions::default()
    };
    let outer = properties("Outer wall", true);

    // Lower half of [min, max) drops to min, upper half boosts to max.
    assert_eq!(outer.resonance_avoided(&options, 50.0), 40.0);
    assert_eq!(outer.resonance_avoided(&options, 70.0), 90.0);
    // Below min and at/above max are untouched.
    assert_eq!(outer.resonance_avoided(&options, 30.0), 30.0);
    assert_eq!(outer.resonance_avoided(&options, 90.0), 90.0);
    // Only `erExternalPerimeter` participates.
    assert_eq!(
        properties("Inner wall", true).resonance_avoided(&options, 50.0),
        50.0
    );
    assert_eq!(
        outer.resonance_avoided(&MotionOptions::default(), 50.0),
        50.0
    );
}

#[test]
fn gap_fill_jerk_uses_default_not_infill() {
    // `is_infill` (ExtrusionEntity.hpp:73-82) excludes `erGapFill`, so gap
    // fill falls to `default_jerk` even when `infill_jerk` is set
    // (`GCode.cpp:6451`).
    let options = MotionOptions {
        default_jerk: 9.0,
        infill_jerk: 20.0,
        ..MotionOptions::default()
    };
    assert_eq!(
        properties("Gap infill", false).jerk(&options, 3),
        options.default_jerk
    );
    // Contrast: real infill roles take `infill_jerk`.
    assert_eq!(
        properties("Internal solid infill", false).jerk(&options, 3),
        20.0
    );
}

#[test]
fn ironing_jerk_uses_infill() {
    // `is_infill` (ExtrusionEntity.hpp:73-82) includes `erIroning`, so
    // ironing takes `infill_jerk` when set (`GCode.cpp:6451`) and falls to
    // `default_jerk` otherwise.
    let options = MotionOptions {
        default_jerk: 9.0,
        infill_jerk: 20.0,
        ..MotionOptions::default()
    };
    assert_eq!(properties("Ironing", false).jerk(&options, 3), 20.0);
    let unset = MotionOptions {
        default_jerk: 9.0,
        ..MotionOptions::default()
    };
    assert_eq!(properties("Ironing", false).jerk(&unset, 3), 9.0);
}

#[test]
fn overhang_wall_jerk_falls_to_default() {
    // `is_external_perimeter` covers only `erExternalPerimeter`
    // (ExtrusionEntity.hpp:64-66), so the overhang wall hits no role arm in
    // the jerk selection (`GCode.cpp:6440-6458`) and takes `default_jerk`
    // even when `outer_wall_jerk` is set.
    let options = MotionOptions {
        default_jerk: 9.0,
        outer_wall_jerk: 7.0,
        inner_wall_jerk: 7.0,
        ..MotionOptions::default()
    };
    assert_eq!(properties("Overhang wall", true).jerk(&options, 3), 9.0);
    // Contrast: the true external perimeter takes the outer value.
    assert_eq!(properties("Outer wall", true).jerk(&options, 3), 7.0);
}
