//! `GCodeWriter::set_temperature` command formatting (`117–165`).

use crate::GCodeFlavor;

pub(super) fn append(
    output: &mut Vec<u8>,
    flavor: GCodeFlavor,
    temperature: u32,
    wait: bool,
    tool: Option<usize>,
) {
    if wait && flavor.skips_waiting_nozzle_temperature() {
        return;
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
    let comment = if command == "M109" {
        "set nozzle temperature and wait for it to be reached"
    } else {
        "set nozzle temperature"
    };
    output.extend_from_slice(
        format!("{command} {parameter}{temperature}{tool} ; {comment}\n").as_bytes(),
    );
    if flavor.waits_after_nozzle_temperature(wait) {
        output.extend_from_slice(b"M116 ; wait for temperature to be reached\n");
    }
}
