use super::super::{ProcessorLimits, estimate::Estimate};

fn used_filament(gcode: &[u8]) -> f64 {
    let settings = crate::ProjectSettings::default();
    let config = ProcessorLimits::default().with_settings(&settings);
    let lines = std::str::from_utf8(gcode)
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    Estimate::record(&lines, 0.0, &config, None)
        .print_statistics
        .total_used_filament
}

#[test]
fn m83_local_relative_e_survives_g90() {
    let gcode = b"M83\nG1 X1 E10\nG90\nG1 X2 E10\n";

    assert_eq!(format!("{:.2}", used_filament(gcode)), "20.00");
}

#[test]
fn g91_makes_e_relative_even_after_m82() {
    let gcode = b"M82\nG92 E0\nG1 X1 E10\nG91\nG1 X1 E10\nG90\nG1 X3 E25\n";

    assert_eq!(format!("{:.2}", used_filament(gcode)), "25.00");
}
