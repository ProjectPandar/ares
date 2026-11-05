//! Public GCodeProcessor apply_config + process_buffer/finalize boundary.
//! Failure inventory before exposure: missing public entry, different processor
//! accounting, dropped moves/settings, or an unnecessary G-code rewrite.

use ares_core::{GCodeFlavor, ProjectSettings, gcode_processor};

#[test]
fn public_processor_delegates_handwritten_relative_extrusion() {
    let input = "M83\nG1 X1 E10 F1200\nG90\nG1 X2 E10\n";
    let output = gcode_processor::process_gcode(input, &ProjectSettings::default(), false, false);
    assert_eq!(output.gcode, input.as_bytes());
    assert_eq!(output.result.filament_diameters, [1.75]);
    assert_eq!(output.result.filament_densities, [0.0]);
    assert_eq!(
        format!("{:.2}", output.result.print_statistics.total_used_filament),
        "20.00"
    );
    let extrusion = output
        .result
        .moves
        .iter()
        .filter(|vertex| {
            vertex.move_type == gcode_processor::EMoveType::Extrude && !vertex.internal_only
        })
        .collect::<Vec<_>>();
    assert_eq!(extrusion.len(), 2);
    assert_eq!(extrusion[0].gcode_id, 2);
    assert_eq!(extrusion[1].gcode_id, 4);
    assert_eq!(extrusion[1].position, [2.0, 0.0, 0.0]);
    assert_eq!(extrusion[1].delta_extruder, 10.0);
}

#[test]
fn public_processor_matches_native_marlin_junction_deviation_start() {
    let mut settings = ProjectSettings::default();
    settings.printer.gcode.gcode_flavor = GCodeFlavor::MarlinFirmware;
    settings.printer.machine.machine_max_junction_deviation.0[0].0 = 0.0;
    let input = "M201 X1000 Y1000 Z500 E5000\nM203 X500 Y500 Z12 E120\nM205 X9 Y9 Z0.2 E2.5 J0.01\nM204 P1000 T1000\nG1 X100 F6000\nG1 X200 F6000\n";
    let output = gcode_processor::process_gcode(input, &settings, false, false);
    assert_eq!(
        format!("{:.6}", output.result.print_statistics.normal_time),
        "2.099956"
    );
}

#[test]
fn public_processor_keeps_classic_jerk_for_nonpositive_live_junction_deviation() {
    for junction_deviation in ["0", "-0.01"] {
        let mut settings = ProjectSettings::default();
        settings.printer.gcode.gcode_flavor = GCodeFlavor::MarlinFirmware;
        let input = format!(
            "M201 X1000 Y1000 Z500 E5000\nM203 X500 Y500 Z12 E120\nM205 X9 Y9 Z0.2 E2.5 J{junction_deviation}\nM204 P1000 T1000\nG1 X100 F6000\nG1 X200 F6000\n"
        );
        let output = gcode_processor::process_gcode(&input, &settings, false, false);
        assert_eq!(
            format!("{:.6}", output.result.print_statistics.normal_time),
            "2.082810",
            "M205 J{junction_deviation}"
        );
    }
}
