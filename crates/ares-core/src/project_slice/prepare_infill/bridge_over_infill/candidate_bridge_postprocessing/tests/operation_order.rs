use std::cell::Cell;

use super::*;

#[test]
fn task22o63_forwards_exact_arithmetic_operands_order_and_owned_state() {
    for (scale, opening_bits, closing_bits) in [
        (
            CoordinateScale::Normal,
            125_829_128.0_f32.to_bits(),
            167_772_176.0_f32.to_bits(),
        ),
        (
            CoordinateScale::LargeBed,
            12_582_913.0_f32.to_bits(),
            16_777_216.0_f32.to_bits(),
        ),
    ] {
        let boundaries = vec![polyline(&[(3, 4), (7, 8)])];
        let boundary_ptr = boundaries.as_ptr();
        let bridge = vec![rectangle(90, 0, 100, 10), rectangle(-90, 0, -80, 10)];
        let expansion = vec![rectangle(-200, -200, 200, 200)];
        let expansion_ptr = expansion.as_ptr();
        let limiting = vec![rectangle(-150, -150, 150, 150)];
        let fill = vec![rectangle(-120, -120, 120, 120)];
        let top = vec![rectangle(-10, -10, 10, 10)];
        let opened = vec![rectangle(80, 0, 90, 10)];
        let closed = vec![rectangle(70, 0, 80, 10)];
        let limited = vec![rectangle(60, 0, 70, 10)];
        let filled = vec![rectangle(50, 0, 60, 10)];
        let final_bridge = vec![rectangle(40, 0, 50, 10), rectangle(-40, 0, -30, 10)];
        let remaining = vec![rectangle(-200, -200, -100, 200)];
        let intersection_call = Cell::new(0);
        let difference_call = Cell::new(0);
        let mut exact_flow = flow();
        exact_flow.spacing = 167.772_17_f32;

        let output = postprocess_candidate_bridge_using(
            collision(boundaries, bridge.clone(), 0.37),
            expansion,
            &limiting,
            &fill,
            &top,
            exact_flow,
            scale,
            |subject, delta| {
                assert_eq!(snapshot(subject), snapshot(&bridge));
                assert_eq!(delta.to_bits(), opening_bits);
                Ok(opened.clone())
            },
            |subject, delta| {
                assert_eq!(snapshot(subject), snapshot(&opened));
                assert_eq!(delta.to_bits(), closing_bits);
                Ok(closed.clone())
            },
            |subject, clip| {
                let call = intersection_call.get();
                intersection_call.set(call + 1);
                match call {
                    0 => {
                        assert_eq!(snapshot(subject), snapshot(&closed));
                        assert_eq!(clip.as_ptr(), limiting.as_ptr());
                        Ok(limited.clone())
                    }
                    1 => {
                        assert_eq!(snapshot(subject), snapshot(&limited));
                        assert_eq!(clip.as_ptr(), fill.as_ptr());
                        Ok(filled.clone())
                    }
                    _ => panic!("only two source intersections are allowed"),
                }
            },
            |subject, clip| {
                let call = difference_call.get();
                difference_call.set(call + 1);
                match call {
                    0 => {
                        assert_eq!(snapshot(subject), snapshot(&filled));
                        assert_eq!(clip.as_ptr(), top.as_ptr());
                        Ok(final_bridge.clone())
                    }
                    1 => {
                        assert_eq!(subject.as_ptr(), expansion_ptr);
                        assert_eq!(snapshot(clip), snapshot(&final_bridge));
                        Ok(remaining.clone())
                    }
                    _ => panic!("only two source differences are allowed"),
                }
            },
        )
        .unwrap();

        assert_eq!(intersection_call.get(), 2);
        assert_eq!(difference_call.get(), 2);
        assert_eq!(output.boundary_polylines.as_ptr(), boundary_ptr);
        assert_eq!(output.bridging_area, final_bridge);
        assert_eq!(output.expansion_area, remaining);
        assert_eq!(output.bridging_angle.to_bits(), 0.37_f64.to_bits());
    }
}

#[test]
fn task22o63_truncates_scaled_spacing_before_f64_opening_multiply() {
    let mut exact_flow = flow();
    exact_flow.spacing = 0.45;
    let result = postprocess_candidate_bridge_using(
        collision(Vec::new(), Vec::new(), 0.0),
        Vec::new(),
        &[],
        &[],
        &[],
        exact_flow,
        CoordinateScale::LargeBed,
        |_, delta| {
            assert_eq!(delta.to_bits(), 33_749.25_f32.to_bits());
            Ok(Vec::new())
        },
        |_, delta| {
            assert_eq!(delta.to_bits(), 44_999.0_f32.to_bits());
            Ok(Vec::new())
        },
        |_, _| Ok(Vec::new()),
        |_, _| Ok(Vec::new()),
    );
    assert!(result.is_ok());
}

#[test]
fn task22o63_injected_errors_stop_at_each_source_operation() {
    for fail_at in 0..6 {
        let step = Cell::new(0);
        let visit = || {
            let current = step.get();
            step.set(current + 1);
            if current == fail_at {
                Err(ClipperError::CoordinateOutOfRange)
            } else {
                Ok(Vec::new())
            }
        };
        let result = postprocess_candidate_bridge_using(
            collision(Vec::new(), Vec::new(), 0.0),
            Vec::new(),
            &[],
            &[],
            &[],
            flow(),
            CoordinateScale::Normal,
            |_, _| visit(),
            |_, _| visit(),
            |_, _| visit(),
            |_, _| visit(),
        );
        assert_eq!(result.unwrap_err(), ClipperError::CoordinateOutOfRange);
        assert_eq!(step.get(), fail_at + 1);
    }
}
