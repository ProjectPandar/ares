use super::account_used_filament;

#[test]
fn m83_local_relative_e_survives_g90() {
    let gcode = b"M83\nG1 X1 E10\nG90\nG1 X2 E10\n";

    assert_eq!(
        format!("{:.2}", account_used_filament(gcode, 1.75)),
        "20.00"
    );
}

#[test]
fn g91_makes_e_relative_even_after_m82() {
    let gcode = b"M82\nG92 E0\nG1 X1 E10\nG91\nG1 X1 E10\nG90\nG1 X3 E25\n";

    assert_eq!(
        format!("{:.2}", account_used_filament(gcode, 1.75)),
        "25.00"
    );
}
