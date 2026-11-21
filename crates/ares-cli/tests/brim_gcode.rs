mod slice_gcode_support;

#[test]
fn brim_gcode_slice_stl_writes_artifacts_and_commands() {
    let gcode = slice_gcode_support::slice_cube(br#"{"layer_change_gcode":"G92 E0","brim_width":1.2,"brim_object_gap":0.2,"brim_type":"outer_only"}"#);
    let (_, brim) = gcode.split_once(";TYPE:Brim\n").unwrap();
    assert!(
        brim.lines()
            .take_while(|line| !line.starts_with(";TYPE:"))
            .any(slice_gcode_support::deposits)
    );
}
