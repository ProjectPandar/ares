use super::{MotionOptions, PathProperties};

fn properties(feature: &'static str, is_perimeter: bool) -> PathProperties<'static> {
    PathProperties {
        mm3_per_mm: 0.08,
        width: 0.45,
        height: 0.2,
        feature,
        is_perimeter,
        end_clip: 0.0,
        fitting: &[],
        slope: None,
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
        properties("Outer wall", true).speed(&options, 0, false, 10.0),
        30.0
    );
    assert_eq!(
        properties("Brim", false).speed(&options, 0, false, 10.0),
        60.0
    );
    assert_eq!(
        properties("Sparse infill", false).speed(&options, 0, false, 10.0),
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
