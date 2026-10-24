//! GCode.cpp::_do_export:2484-2528,2890 binds the mode-dependent layer total.

use crate::runner::native_stream_case;

const DIR: &str = "sequential-template-layer-count";
const UPSTREAM: &[&str] = &[
    "GCode.cpp::_do_export:2484-2528 m_layer_count;:2890 total_layer_count",
    "GCode.cpp::process_layer:4677-4684 layer_change_gcode",
];

fn assert_native(native: &str, printer: &str, sequence: &str, total: usize, offset: &str) {
    for marker in [
        format!("; printer_model = {printer}\n"),
        format!("; print_sequence = {sequence}\n"),
        format!("; total layer number: {total}\n"),
        format!("; extruder_offset = {offset}\n"),
        "; model label id: 45,56\n".to_owned(),
        "; printing object overhang.obj id:0 copy 0\n".to_owned(),
        "; printing object cube_with_hole.obj id:0 copy 0\n".to_owned(),
    ] {
        assert!(
            native.contains(&marker),
            "Native witness missing {marker:?}"
        );
    }
    let actual = native
        .lines()
        .filter_map(|line| line.strip_prefix("; layer num/total_layer_count: "))
        .collect::<Vec<_>>();
    let expected = (1..=total)
        .map(|layer| format!("{layer}/{total}"))
        .collect::<Vec<_>>();
    assert_eq!(actual, expected, "Native layer-template sequence");
}

#[test]
fn orca_parity_sequential_template_layer_count_a1_byobject() {
    native_stream_case::check(
        (DIR, UPSTREAM),
        "a1-byobject",
        include_bytes!("../../../../tests/parity/sequential-template-layer-count/a1-byobject.3mf"),
        "b436ba663498652206663fb1f276c7d972f09b5b6fccad15e1d998aeeb53e138",
        (false, |reference, replay| {
            for case in [reference, replay] {
                assert_native(&case.native, "Bambu Lab A1", "by object", 81, "0x0");
            }
        }),
    );
}

#[test]
fn orca_parity_sequential_template_layer_count_a1_bylayer_control() {
    native_stream_case::check(
        (DIR, UPSTREAM),
        "a1-bylayer-control",
        include_bytes!("../../../../tests/parity/sequential-template-layer-count/a1-bylayer.3mf"),
        "4f0e4606b6725a207c07378093e3e169fa7610bb50cd8e24969ca50d3e1b5ace",
        (false, |reference, replay| {
            for case in [reference, replay] {
                assert_native(&case.native, "Bambu Lab A1", "by layer", 50, "0x0");
            }
        }),
    );
}

#[test]
fn orca_parity_sequential_template_layer_count_p1s_bylayer_control() {
    native_stream_case::check(
        (DIR, UPSTREAM),
        "p1s-bylayer-control",
        include_bytes!("../../../../tests/parity/auto-lift-query-frame/project.3mf"),
        "f22c2132032860447369a3b8d29291230108f20ef1030cb532447cfb90ae31dd",
        (false, |reference, replay| {
            for case in [reference, replay] {
                assert_native(&case.native, "Bambu Lab P1S", "by layer", 50, "0x2");
                assert!(
                    case.native
                        .contains("G17\nG3 Z4 I.058 J-1.216 P1  F30000\n")
                );
            }
        }),
    );
}
