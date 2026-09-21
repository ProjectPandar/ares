use super::*;

fn input(min_skirt_length_mm: f64) -> LoopCountInput {
    LoopCountInput {
        configured_loops: 1,
        min_skirt_length_mm,
        distance_mm: 2.0,
        effective_line_width: 0.4,
        bounds: Bounds {
            min_x: -0.5,
            min_y: -0.5,
            max_x: 0.5,
            max_y: 0.5,
        },
        skirt_extrusion_per_mm: 0.05,
        apply_min_length: true,
    }
}

#[test]
fn keeps_configured_loops_when_min_length_is_disabled() {
    assert_eq!(loop_count(input(0.0)).unwrap(), 1);
}

#[test]
fn zero_configured_loops_disable_min_length_extension() {
    let input = LoopCountInput {
        configured_loops: 0,
        ..input(1.0)
    };

    assert_eq!(loop_count(input).unwrap(), 0);
}

#[test]
fn extends_until_min_length_is_reached() {
    assert_eq!(loop_count(input(3.0)).unwrap(), 3);
}

#[test]
fn rejects_requests_over_the_loop_bound() {
    let err = loop_count(input(1.0e12)).unwrap_err();

    assert!(
        matches!(err, SliceError::InvalidInput(message) if message.contains("min_skirt_length would require more than 10000 skirt loops"))
    );
}
