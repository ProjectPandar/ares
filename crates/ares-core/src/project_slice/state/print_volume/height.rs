//! BuildVolume.cpp:377-421: zero disables the scene ceiling only.

pub(super) fn scene_limit(printable_height: f64) -> f64 {
    if printable_height == 0.0 {
        f64::MAX
    } else {
        printable_height
    }
}
