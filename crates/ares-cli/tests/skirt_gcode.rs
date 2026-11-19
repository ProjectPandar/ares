mod slice_gcode_support;

#[test]
fn slice_stl_writes_skirt_artifacts_and_commands() {
    let gcode = slice_gcode_support::slice_cube(
        br#"{"layer_change_gcode":"G92 E0","skirt_loops":1,"skirt_height":1}"#,
    );
    let (_, skirt) = gcode.split_once(";TYPE:Skirt\n").unwrap();
    assert!(
        skirt
            .lines()
            .take_while(|line| !line.starts_with(";TYPE:"))
            .any(slice_gcode_support::deposits)
    );
}
