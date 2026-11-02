//! Beta occupied support-fill inputs before skirt offset and seam selection.
use crate::runner::native_stream_case::check;

#[test]
fn skirt_support_occupied_input_matches_native_complete_output() {
    check(
        (
            "skirt-support-occupied-input",
            &[
                "Print.cpp::Print::_make_skirt",
                "ExtrusionEntity.cpp::ExtrusionPath::collect_points",
                "GCode.cpp::Skirt::find_start_point",
            ],
        ),
        "support-overhang",
        include_bytes!(
            "../../../../../tests/parity/skirt-support-occupied-input/support-overhang.3mf"
        ),
        "58bf8c9a7a28cc19d7ee61c45458ae319dbce6882036cc7a7f6dde06d60c288d",
        (false, |reference, replay| {
            for native in [reference, replay] {
                let prefix = native.native.split_once(";TYPE:Skirt\n").unwrap().0;
                let start = prefix
                    .lines()
                    .rev()
                    .find(|line| line.starts_with("G1 X") && line.contains(" Y"))
                    .unwrap();
                assert_eq!(start, "G1 X95.517 Y99.935");
            }
        }),
    );
}
