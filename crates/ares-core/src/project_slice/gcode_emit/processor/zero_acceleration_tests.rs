//! `GCodeProcessor.cpp:130–158,255–274`: zero acceleration is cruise-only.
use super::estimate::Estimate;
use super::motion::{MotionState, planned_times};
use super::{ProcessorLimits, process};
use crate::{GCodeFlavor, MachineEnvelopeOptions};

fn zero_z_limits() -> ProcessorLimits {
    let machine: MachineEnvelopeOptions = serde_json::from_value(serde_json::json!({
        "machine_max_acceleration_z": [0.0],
        "machine_max_speed_z": [10.0]
    }))
    .unwrap();
    ProcessorLimits::from_config(&machine, GCodeFlavor::Klipper, false)
}

#[test]
fn zero_acceleration_linear_move_has_one_second_total_and_complete_output() {
    let input =
        "M73 P0 R0\nG1 Z10 F600\nM73 P100 R0\n; estimated printing time (normal mode) = 0s\n";
    let lines = input.lines().map(str::to_owned).collect::<Vec<_>>();
    let estimate = Estimate::from_lines(&lines, 0.0, zero_z_limits());
    // Upstream: both acceleration distances/times = 0; cruise = 10 / 10.
    assert!(estimate.total.is_finite());
    assert_eq!(estimate.total, 1.0);
    assert_eq!(
        process(input.as_bytes().to_vec(), true, 0.0, 0.0, zero_z_limits()),
        b"M73 P0 R0\nG1 Z10 F600\nM73 P100 R0\n; estimated printing time (normal mode) = 1.000000s\n"
    );
}

#[test]
fn zero_acceleration_linear_moves_emit_finite_remaining_time_and_halfway_progress() {
    let input = "M73 P0 R0\nG1 Z600 F600\nG1 Z1200\nM73 P100 R0\n; estimated printing time (normal mode) = 0s\n";
    let lines = input.lines().map(str::to_owned).collect::<Vec<_>>();
    let estimate = Estimate::from_lines(&lines, 0.0, zero_z_limits());
    // Two 600 / 10 = 60s blocks. The existing cache exports the first at G1 #2.
    assert!(estimate.total.is_finite());
    assert_eq!(estimate.total, 120.0);
    assert_eq!(estimate.elapsed_at(3), Some(60.0));
    assert_eq!(
        process(input.as_bytes().to_vec(), true, 0.0, 0.0, zero_z_limits()),
        b"M73 P0 R2\nG1 Z600 F600\nG1 Z1200\nM73 P50 R1\nM73 P100 R0\n; estimated printing time (normal mode) = 2m 0s\n"
    );
}

#[test]
fn zero_print_acceleration_clamps_junction_feedrate_to_zero() {
    // `GCodeProcessor.cpp:4025-4031`: the centripetal limit is
    // `sqrt(get_acceleration(mode) * r)` with NO floor — `M204 P0`
    // collapses every junction feedrate to 0 (the oracle's 7m30s total
    // in the machine_max_acceleration_extruding=min sweep case). Ares
    // previously floored the value at 1.0 and diverged.
    let mut state = MotionState::with_limits(zero_z_limits());
    assert!(state.motion("M204 P0").is_none());
    // Two XY moves meeting at a 90° junction: the junction feedrate
    // clamps to sqrt(0 * r) = 0 and the zero-acceleration trapezoid
    // collapses to cruise-only — distance / feedrate with no ramp.
    let first = state.motion("G1 X10 Y0 F600").unwrap();
    let second = state.motion("G1 X0 Y10 F600").unwrap();
    assert_eq!(second.centripetal_acceleration, 0.0);
    let times = planned_times(&[first, second]);
    assert_eq!(times.len(), 2);
    // 10mm at 10mm/s cruise-only, then 14.142136mm at 10mm/s
    // (f32 trapezoid arithmetic — compare loosely).
    assert!((times[0] - 1.0).abs() < 1e-6);
    assert!((times[1] - std::f64::consts::SQRT_2).abs() < 1e-6);
}

#[test]
fn fractional_print_acceleration_still_limits_the_junction() {
    let mut state = MotionState::with_limits(zero_z_limits());
    assert!(state.motion("M204 P0.5").is_none());
    let first = state.motion("G1 X10 Y0 F600").unwrap();
    let second = state.motion("G1 X0 Y10 F600").unwrap();
    assert_eq!(second.centripetal_acceleration, 0.5);
    let times = planned_times(&[first, second]);
    // Junction feedrate min(speed, sqrt(0.5 * r)) — still finite and
    // strictly capped by the centripetal limit.
    assert!(times.iter().all(|time| time.is_finite() && *time >= 0.0));
}

#[test]
fn zero_acceleration_zero_cruise_feedrate_has_zero_time() {
    // GCodeProcessor.hpp:439: cruise_time is zero when cruise_feedrate is zero.
    let mut block = MotionState::with_limits(zero_z_limits())
        .motion("G1 Z10 F600")
        .unwrap();
    block.speed = 0.0;
    assert_eq!(planned_times(&[block]), [0.0]);
}

#[test]
fn zero_acceleration_helical_segments_have_cruise_only_times_and_complete_output() {
    let command = "G3 X10 Y10 Z100 I0 J10 F600";
    let blocks = MotionState::with_limits(zero_z_limits()).motions(command);
    assert!(blocks.len() > 1);
    let times = planned_times(&blocks);
    for (block, time) in blocks.iter().zip(&times) {
        assert_eq!(block.max_acceleration[2], 0.0);
        assert!(time.is_finite());
        // Upstream f32 cruise-distance / feedrate; no accel/decel contribution.
        assert_eq!(*time, f64::from(block.distance as f32 / block.speed as f32));
    }
    let input = format!(
        "M73 P0 R0\n{command}\nM73 P100 R0\n; estimated printing time (normal mode) = 0s\n"
    );
    let lines = input.lines().map(str::to_owned).collect::<Vec<_>>();
    let estimate = Estimate::from_lines(&lines, 0.0, zero_z_limits());
    assert!(estimate.total.is_finite());
    assert_eq!(estimate.total, times.iter().sum::<f64>());
    assert_eq!(estimate.total as u64, 10);
    // The arc line's own marker reads segment 15 of 16's cache entry
    // (`GCodeProcessor.cpp:1466`), so it carries ~94% progress.
    assert_eq!(
        process(input.into_bytes(), true, 0.0, 0.0, zero_z_limits()),
        b"M73 P0 R0\nG3 X10 Y10 Z100 I0 J10 F600\nM73 P93 R0\nM73 P100 R0\n; estimated printing time (normal mode) = 10s\n"
    );
}
