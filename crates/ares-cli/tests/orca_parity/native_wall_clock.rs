#![cfg(target_os = "linux")]

pub(crate) fn result_comparison_bytes(bytes: &[u8]) -> Vec<u8> {
    // OrcaSlicer.cpp:5540/436 and :7104/437 serialize UTC elapsed seconds.
    let mut json = std::str::from_utf8(bytes).unwrap().to_owned();
    for field in ["prepare_time", "export_time"] {
        let prefix = format!("\"{field}\": ");
        let start = json.find(&prefix).unwrap() + prefix.len();
        let digits = json[start..].bytes().take_while(u8::is_ascii_digit).count();
        json.replace_range(start..start + digits, "0");
    }
    json.into_bytes()
}

#[test]
fn custom_gcode_json_comparison_ignores_only_wall_clock_values() {
    let actual = br#"{"prepare_time": 0, "export_time": 0, "return_code": -100, "error_string": "numbers 27 12", "untouched": 27}"#;
    let native = br#"{"prepare_time": 27, "export_time": 12, "return_code": -100, "error_string": "numbers 27 12", "untouched": 27}"#;
    assert_eq!(
        result_comparison_bytes(actual),
        result_comparison_bytes(native)
    );
    assert_eq!(result_comparison_bytes(native).as_slice(), actual);
}

#[test]
fn custom_gcode_json_comparison_preserves_other_bytes() {
    let actual = br#"{"prepare_time": 0, "export_time": 0, "return_code": -100}"#;
    for different in [
        br#"{"prepare_time": 0, "export_time": 0, "return_code": -101}"#.as_slice(),
        br#"{"prepare_time": 0, "export_time": 0, "return_code":  -100}"#.as_slice(),
        br#"{"export_time": 0, "prepare_time": 0, "return_code": -100}"#.as_slice(),
    ] {
        assert_ne!(
            result_comparison_bytes(actual),
            result_comparison_bytes(different)
        );
    }
}
