/// `GCodeWriter.cpp:475-481`: both values use decimal-point general precision.
pub(super) fn append_klipper_accel_to_decel(command: &mut String, acceleration: u32, factor: f64) {
    command.push_str(" ACCEL_TO_DECEL=");
    command.push_str(&crate::options::format_number(
        f64::from(acceleration) * factor / 100.0,
    ));
    command.push_str(" MINIMUM_CRUISE_RATIO=");
    command.push_str(&crate::options::format_number(1.0 - factor / 100.0));
}
