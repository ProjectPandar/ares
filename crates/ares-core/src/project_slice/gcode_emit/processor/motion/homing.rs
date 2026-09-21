//! G28 homing expansion (`GCodeProcessor::process_gcode` dispatches G28
//! as a synthetic G1 move over the homed axes).

pub(super) fn homing_g1(code: &str) -> String {
    let has_axis = ['X', 'Y', 'Z'].iter().any(|&axis| code.contains(axis));
    let mut homing = String::from("G1");
    let axes = ['X', 'Y', 'Z']
        .into_iter()
        .filter(|axis| !has_axis || code.contains(*axis));
    for axis in axes {
        homing.push(' ');
        homing.push(axis);
        homing.push('0');
    }
    homing
}
