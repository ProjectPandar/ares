//! Public GCodeProcessor apply_config + process_buffer/finalize boundary.
//! Failure inventory before exposure: missing public entry, different processor
//! accounting, dropped moves/settings, or an unnecessary G-code rewrite.

use ares_core::{ProjectSettings, gcode_processor};

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
