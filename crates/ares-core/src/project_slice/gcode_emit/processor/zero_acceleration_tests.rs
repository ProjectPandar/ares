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
    // Two blocks: upstream `calculate_time` drops a lone trailing block.
    let input = "M73 P0 R0\nG1 Z10 F600\nG1 Z20 F600\nM73 P100 R0\n; estimated printing time (normal mode) = 0s\n";
    let lines = input.lines().map(str::to_owned).collect::<Vec<_>>();
    let estimate = Estimate::record(
        &lines,
        0.0,
        &zero_z_limits().with_settings(&Default::default()),
        None,
    );
    // Upstream: both acceleration distances/times = 0; cruise = 10 / 10.
    assert!(estimate.total.is_finite());
    assert_eq!(estimate.total, 2.0);
    assert_eq!(
        process(input.as_bytes().to_vec(), true, 0.0, 0.0, zero_z_limits().with_settings(&Default::default())).gcode,
        b"M73 P0 R0\nG1 Z10 F600\nG1 Z20 F600\nM73 P50 R0\nM73 P100 R0\n; estimated printing time (normal mode) = 2s\n"
    );
}

#[test]
fn zero_acceleration_linear_moves_emit_finite_remaining_time_and_halfway_progress() {
    let input = "M73 P0 R0\nG1 Z600 F600\nG1 Z1200\nM73 P100 R0\n; estimated printing time (normal mode) = 0s\n";
    let lines = input.lines().map(str::to_owned).collect::<Vec<_>>();
    let estimate = Estimate::record(
        &lines,
        0.0,
        &zero_z_limits().with_settings(&Default::default()),
        None,
    );
    // Two 600 / 10 = 60s blocks. The existing cache exports the first at G1 #2.
    assert!(estimate.total.is_finite());
    assert_eq!(estimate.total, 120.0);
    assert_eq!(estimate.elapsed_at(3), Some(60.0));
    assert_eq!(
        process(input.as_bytes().to_vec(), true, 0.0, 0.0, zero_z_limits().with_settings(&Default::default())).gcode,
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
    // Shallow ~0.11° turn (unit-vector difference ~0.002 < 0.5) so
    // the centripetal branch at planner/mod.rs:256-264 actually runs:
    // the junction feedrate clamps to sqrt(0 * r) = 0 and the
    // zero-acceleration trapezoid collapses to cruise-only.
    let first = state.motion("G1 X10 Y0 F600").unwrap();
    let second = state.motion("G1 X19.99 Y0.02 F600").unwrap();
    assert_eq!(second.centripetal_acceleration, 0.0);
    let times = planned_times(&[first, second]);
    assert_eq!(times.len(), 2, "times={times:?}");
    // Upstream sets `curr.feedrate = 0` for the junction-clamped
    // move (`GCodeProcessor.cpp:4031`), so its trapezoid computes
    // cruise_feedrate=0 → block_time=0. This is exactly the
    // collapse the oracle shows for `machine_max_acceleration_*`.
    assert_eq!(times[1], 0.0, "times={times:?}");
    // The first move keeps cruise-only time ~1.005s (f32 trapezoid
    // on the post-clamp junction feedrate).
    assert!((times[0] - 1.005).abs() < 1e-3, "times={times:?}");
}

#[test]
fn fractional_print_acceleration_still_limits_the_junction() {
    let mut state = MotionState::with_limits(zero_z_limits());
    assert!(state.motion("M204 P0.5").is_none());
    let first = state.motion("G1 X10 Y0 F600").unwrap();
    // ~5.3° turn continuing forward — inside the
    // `difference < 0.5` centripetal gate; radius ~110mm ⇒ upstream
    // clamps `curr.feedrate` to sqrt(0.5*110) ≈ 7.4 mm/s
    // (`GCodeProcessor.cpp:4031` clamps the BLOCK's feedrate, not
    // just the junction speed), so the whole second move cruises at
    // ~7.4: ~10.05/7.4 ≈ 1.35s > 1.0.
    let second = state.motion("G1 X19.9 Y0.9 F600").unwrap();
    assert_eq!(second.centripetal_acceleration, 0.5);
    let times = planned_times(&[first, second]);
    assert!(times.iter().all(|time| time.is_finite() && *time >= 0.0));
    // Cruise-clamped block is strictly slower than the ~1.0s
    // unclamped time; an unclamped junction would finish in exactly
    // distance/speed.
    assert!(times[1] > 1.0, "times={times:?}");
}

#[test]
fn zero_acceleration_zero_cruise_feedrate_has_zero_time() {
    // GCodeProcessor.hpp:439: cruise_time is zero when cruise_feedrate is zero.
    let mut block = MotionState::with_limits(zero_z_limits())
        .motion("G1 Z10 F600")
        .unwrap();
    block.speed = 0.0;
    // Test hook force-drains the lone block (upstream's `<2` guard only
    // guards the `time +=` accounting, not the trapezoid math).
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
    let estimate = Estimate::record(
        &lines,
        0.0,
        &zero_z_limits().with_settings(&Default::default()),
        None,
    );
    assert!(estimate.total.is_finite());
    assert_eq!(estimate.total, times.iter().sum::<f64>());
    assert_eq!(estimate.total as u64, 10);
    // The arc line's own marker reads segment 15 of 16's cache entry
    // (`GCodeProcessor.cpp:1466`), so it carries ~94% progress.
    assert_eq!(
        process(input.into_bytes(), true, 0.0, 0.0, zero_z_limits().with_settings(&Default::default())).gcode,
        b"M73 P0 R0\nG3 X10 Y10 Z100 I0 J10 F600\nM73 P93 R0\nM73 P100 R0\n; estimated printing time (normal mode) = 10s\n"
    );
}
