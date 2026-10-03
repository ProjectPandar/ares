//! `GCode.cpp:3752` runs PressureEqualizer before the CoolingBuffer, and
//! `PressureEqualizer.cpp:174-200` holds each layer back until the next
//! arrives; the trailing NOP layer result drains the last one.
use super::super::*;
use super::role_end::cooling_state;
use crate::project_slice::pressure_equalizer::PressureEqualizerPass;

#[test]
fn finish_drains_the_layer_the_equalizer_holds_back() {
    let mut cooling = cooling_state();
    // The fixture ramp resolves to 0% here; start there so no fan line interleaves.
    cooling.part_speed = Some(0);
    cooling.equalizer = Some(PressureEqualizerPass::new(0.0, 3.0, false, &[1.75], true));
    let mut output = Vec::new();
    for (index, layer) in ["G1 X1 Y1 F600\n", "G1 X2 Y2 F600\n"]
        .into_iter()
        .enumerate()
    {
        cooling.pending_layer_index = Some(index);
        let start = output.len();
        output.extend_from_slice(PART_FAN_MARKER);
        output.extend_from_slice(layer.as_bytes());
        cooling.finish_layer(&mut output, start, true);
    }
    assert_eq!(output, b"G1 X1 Y1 F600\n");

    cooling.finish(&mut output);
    assert_eq!(output, b"G1 X1 Y1 F600\nG1 X2 Y2 F600\n");
    assert!(cooling.equalized_layer_result.is_none());
}

#[test]
fn empty_tool_result_advances_equalizer_without_flushing_cooling() {
    let mut cooling = cooling_state();
    cooling.part_speed = Some(0);
    cooling.equalizer = Some(PressureEqualizerPass::new(0.0, 3.0, false, &[1.75], true));
    let mut output = Vec::new();
    cooling.pending_layer_index = Some(0);
    output.extend_from_slice(PART_FAN_MARKER);
    output.extend_from_slice(b"G1 X1 Y1 F600\n");
    cooling.finish_layer(&mut output, 0, true);

    cooling.empty_layer(&mut output, 42, false);
    assert_eq!(output, b"G1 X1 Y1 F600\n");
    assert_eq!(cooling.equalized_layer_result, Some((42, false)));

    cooling.pending_layer_index = Some(1);
    let start = output.len();
    output.extend_from_slice(PART_FAN_MARKER);
    output.extend_from_slice(b"G1 X2 Y2 F600\n");
    cooling.finish_layer(&mut output, start, true);
    assert_eq!(output, b"G1 X1 Y1 F600\n");
    cooling.finish(&mut output);
    assert_eq!(output, b"G1 X1 Y1 F600\nG1 X2 Y2 F600\n");
}

#[test]
fn empty_tool_result_keeps_support_chunk_buffered() {
    let mut cooling = cooling_state();
    cooling.part_speed = Some(0);
    let mut output = Vec::new();
    cooling.pending_layer_index = Some(0);
    output.extend_from_slice(PART_FAN_MARKER);
    output.extend_from_slice(b"G1 X1 Y1 F600\n");
    cooling.finish_layer(&mut output, 0, false);
    cooling.empty_layer(&mut output, 42, false);
    assert!(output.is_empty());

    cooling.pending_layer_index = Some(1);
    output.extend_from_slice(PART_FAN_MARKER);
    output.extend_from_slice(b"G1 X2 Y2 F600\n");
    cooling.finish_layer(&mut output, 0, true);
    assert_eq!(output, b"G1 X1 Y1 F600\nG1 X2 Y2\n");
}

#[test]
fn final_empty_tool_result_flushes_preceding_support_chunk() {
    for equalized in [false, true] {
        let mut cooling = cooling_state();
        cooling.part_speed = Some(0);
        if equalized {
            cooling.equalizer = Some(PressureEqualizerPass::new(0.0, 3.0, false, &[1.75], true));
        }
        let mut output = Vec::new();
        cooling.pending_layer_index = Some(0);
        output.extend_from_slice(PART_FAN_MARKER);
        output.extend_from_slice(b"G1 X1 Y1 F600\n");
        cooling.finish_layer(&mut output, 0, false);
        cooling.empty_layer(&mut output, 42, true);
        cooling.finish(&mut output);
        assert_eq!(output, b"G1 X1 Y1 F600\n", "equalized={equalized}");
    }
}

#[test]
fn final_empty_tool_result_applies_fan_without_a_marker() {
    let mut cooling = cooling_state();
    cooling.part_fan_ramp = PartCoolingFanRamp::new(PartCoolingFanRampConfig {
        min_speed: 100.0,
        max_speed: 100.0,
        full_speed_layer: 0,
        close_fan_first_layers: 0,
        layer_times_s: [5.0, 10.0],
        fan_kickstart_s: 0.0,
        reduce_fan_stop_start_freq: false,
    });
    cooling.part_speed = Some(0);
    cooling.physical_part_speed = 0;
    let mut output = Vec::new();
    cooling.empty_layer(&mut output, 42, true);
    assert_eq!(output, b"M106 S255\n");
    assert_eq!(cooling.part_speed, Some(100));
    assert_eq!(cooling.physical_part_speed, 100);
}
