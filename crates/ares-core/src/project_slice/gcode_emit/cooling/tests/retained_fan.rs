//! Failure modes: END overwrites retained role speed; a changed baseline fails
//! to reset it; an active request loses precedence over FORCE_RESUME.
//! `CoolingBuffer.cpp:808-812,984-1005` owns these distinct state writes.
use super::super::*;
use crate::options::PartCoolingFanRampConfig;
use super::role_end::cooling_state;

#[test]
fn role_end_keeps_retained_speed_for_forced_resume() {
    let mut cooling = cooling_state();
    let mut output = Vec::new();
    append_deferred_role_fan(
        &mut output,
        DeferredRoleFan::Fixed {
            speed: 80,
            force: false,
        },
    );
    append_deferred_role_fan(&mut output, DeferredRoleFan::Baseline);
    output.extend_from_slice(b";_FORCE_RESUME_FAN_SPEED\nT1\n");
    cooling.resolve_role_fans(&mut output, 0, 51, 3);
    assert_eq!(output, b"M106 S204\nM106 S130\nM106 S204\nT1\n");
    assert_eq!(cooling.physical_part_speed, 80);
}

#[test]
fn changed_layer_baseline_resets_retained_role_speed() {
    let mut cooling = cooling_state();
    cooling.physical_part_speed = 80;
    let mut output = Vec::new();
    cooling.part_speed = Some(51);
    cooling.part_fan_ramp = PartCoolingFanRamp::new(PartCoolingFanRampConfig {
        min_speed: 100.0,
        max_speed: 100.0,
        full_speed_layer: 0,
        close_fan_first_layers: 0,
        layer_times_s: [0.0, 0.0],
        fan_kickstart_s: 0.0,
        reduce_fan_stop_start_freq: true,
    });
    cooling.cool_layer(
        &mut output,
        b";__ARES_PART_FAN_STATE__\n;_FORCE_RESUME_FAN_SPEED\n".to_vec(),
        (3, true),
        false,
    );
    assert_eq!(cooling.physical_part_speed, 100);
    assert_eq!(output, b"M106 S255\nM106 S255\n");
}

#[test]
fn active_role_request_takes_precedence_over_forced_resume() {
    let mut cooling = cooling_state();
    let mut output = Vec::new();
    append_deferred_role_fan(
        &mut output,
        DeferredRoleFan::Fixed {
            speed: 80,
            force: false,
        },
    );
    output.extend_from_slice(b";_FORCE_RESUME_FAN_SPEED\nT1\n");
    cooling.resolve_role_fans(&mut output, 0, 51, 3);
    assert_eq!(output, b"M106 S204\nT1\n");
}
