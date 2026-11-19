mod slice_gcode_support;

#[test]
fn slice_stl_writes_output_with_stl_format() {
    let gcode = slice_gcode_support::slice_cube(br#"{"layer_change_gcode":"G92 E0","layer_height":0.25,"nozzle_diameter":["0.4"],"filament_diameter":["1.75"]}"#);
    assert!(gcode.starts_with("; HEADER_BLOCK_START\n"));
    assert!(gcode.lines().any(|line| line == "; layer_height = 0.25"));
    assert!(gcode.lines().any(|line| line == "; nozzle_diameter = 0.4"));
    assert!(
        gcode
            .lines()
            .any(|line| line == "; filament_diameter = 1.75")
    );
    assert_eq!(
        gcode.lines().find_map(|line| line.strip_prefix(";Z:")),
        Some("0.2")
    );
    assert!(gcode.lines().any(slice_gcode_support::deposits));
}
