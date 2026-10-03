//! `GCodeWriter::set_temperature` command formatting (`117–165`).

use crate::GCodeFlavor;

pub(super) fn append(
    output: &mut Vec<u8>,
    flavor: GCodeFlavor,
    temperature: u32,
    wait: bool,
    tool: Option<usize>,
) {
    output.extend_from_slice(command(flavor, temperature, wait, tool, "").as_bytes());
}

pub(super) fn command(
    flavor: GCodeFlavor,
    temperature: u32,
    wait: bool,
    tool: Option<usize>,
    comment: &str,
) -> String {
    if wait && flavor.skips_waiting_nozzle_temperature() {
        return String::new();
    }
    let command = flavor.nozzle_temperature_code(wait);
    let parameter = if matches!(flavor, GCodeFlavor::Mach3 | GCodeFlavor::Machinekit) {
        'P'
    } else {
        'S'
    };
    let tool = match (flavor, tool) {
        (_, None) => String::new(),
        (GCodeFlavor::RepRapFirmware, Some(tool)) => format!(" P{tool}"),
        (_, Some(tool)) => format!(" T{tool}"),
    };
    let comment = if !comment.is_empty() {
        comment
    } else if command == "M109" {
        "set nozzle temperature and wait for it to be reached"
    } else {
        "set nozzle temperature"
    };
    let mut output = format!("{command} {parameter}{temperature}{tool} ; {comment}\n");
    if flavor.waits_after_nozzle_temperature(wait) {
        output.push_str("M116 ; wait for temperature to be reached\n");
    }
    output
}
