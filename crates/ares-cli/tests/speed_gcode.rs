mod slice_gcode_support;

#[test]
fn slice_stl_writes_speed_feedrates() {
    let gcode = slice_gcode_support::slice_cube(br#"{"layer_change_gcode":"G92 E0","outer_wall_speed":30,"initial_layer_speed":30,"filament_max_volumetric_speed":[0],"slow_down_for_layer_cooling":[false]}"#);
    assert!(gcode.split(";TYPE:Outer wall\n").skip(1).any(|wall| {
        let wall = wall.split(";TYPE:").next().unwrap();
        wall.lines().any(|line| line == "G1 F1800")
            && wall.lines().any(slice_gcode_support::deposits)
    }));
}
