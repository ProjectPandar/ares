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
        properties("Outer wall", true).speed(&options, 0, 10.0),
        30.0
    );
    assert_eq!(properties("Brim", false).speed(&options, 0, 10.0), 60.0);
    assert_eq!(
        properties("Sparse infill", false).speed(&options, 0, 10.0),
        60.0
    );
}
