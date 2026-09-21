//! Firmware-retraction G10/G11 emulation (`GCodeProcessor.cpp:4873-4893`):
//! expand to a fake G1 with the retraction E delta and the
//! retraction/deretraction feedrate.

use super::super::motion_util::MMMIN_TO_MMSEC;
use super::MotionKind;
use super::MotionState;

pub(super) fn emulate_firmware_retraction(
    state: &mut MotionState,
    command: &str,
) -> Option<super::MotionBlock> {
    let (e_delta, feedrate) = if command == "G10" {
        (-state.retraction_length, state.retraction_feedrate)
    } else {
        (
            state.retraction_length + state.retract_restart_extra,
            state.deretraction_feedrate,
        )
    };
    let mut fake = String::from("G1");
    if !state.e_relative {
        let absolute = state.e_position + e_delta;
        fake.push_str(&format!(" E{absolute}"));
    } else {
        fake.push_str(&format!(" E{e_delta}"));
    }
    fake.push_str(&format!(" F{feedrate}"));
    state.motion(&fake).map(|mut block| {
        block.kind = if command == "G10" {
            MotionKind::Regular
        } else {
            MotionKind::Unretract
        };
        block
    })
}
