#![cfg(target_os = "linux")]

pub(crate) fn result_comparison_bytes(bytes: &[u8]) -> Vec<u8> {
    // OrcaSlicer.cpp:5540/436 and :7104/437 serialize UTC elapsed seconds, and so does every
    // success-report plate entry (sliced_time / sliced_time_with_cache, :7259-7260 at the beta).
    let mut json = std::str::from_utf8(bytes).unwrap().to_owned();
    for field in ["prepare_time", "export_time"] {
        let prefix = format!("\"{field}\": ");
        let start = json.find(&prefix).unwrap() + prefix.len();
        zero_digits(&mut json, start);
    }
    for field in ["sliced_time", "sliced_time_with_cache"] {
        let prefix = format!("\"{field}\": ");
        let mut from = 0;
        while let Some(found) = json[from..].find(&prefix) {
            let start = from + found + prefix.len();
            zero_digits(&mut json, start);
            from = start;
        }
    }
    json.into_bytes()
}

fn zero_digits(json: &mut String, start: usize) {
    let digits = json[start..].bytes().take_while(u8::is_ascii_digit).count();
    json.replace_range(start..start + digits, "0");
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

/// The success report's per-plate `sliced_time` / `sliced_time_with_cache` are wall-clock seconds
/// too (`OrcaSlicer.cpp:7259-7260`).
#[test]
fn success_json_comparison_ignores_every_sliced_time() {
    let actual = br#"{"export_time": 0, "prepare_time": 0, "sliced_plates": [{"id": 1, "sliced_time": 0, "sliced_time_with_cache": 0, "triangle_count": 12}, {"id": 2, "sliced_time": 0, "sliced_time_with_cache": 0, "triangle_count": 3}]}"#;
    let native = br#"{"export_time": 4, "prepare_time": 1, "sliced_plates": [{"id": 1, "sliced_time": 7, "sliced_time_with_cache": 2, "triangle_count": 12}, {"id": 2, "sliced_time": 15, "sliced_time_with_cache": 9, "triangle_count": 3}]}"#;
    assert_eq!(result_comparison_bytes(native).as_slice(), actual);
    let other_count = br#"{"export_time": 0, "prepare_time": 0, "sliced_plates": [{"id": 1, "sliced_time": 0, "sliced_time_with_cache": 0, "triangle_count": 13}, {"id": 2, "sliced_time": 0, "sliced_time_with_cache": 0, "triangle_count": 3}]}"#;
    assert_ne!(
        result_comparison_bytes(actual),
        result_comparison_bytes(other_count)
    );
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
