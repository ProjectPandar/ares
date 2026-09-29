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
    cooling.part_speed = 0;
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
        cooling.finish_layer(&mut output, start);
    }
    assert_eq!(output, b"G1 X1 Y1 F600\n");

    cooling.finish(&mut output);
    assert_eq!(output, b"G1 X1 Y1 F600\nG1 X2 Y2 F600\n");
    assert!(cooling.equalized_layer_index.is_none());
}
