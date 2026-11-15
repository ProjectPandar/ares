//! Public processor regression for the source-owned end-gcode M400 skip.
//! Native authority and pre-test failure inventory:
//! docs/architecture/processor-end-gcode-m400-skip.md.

use ares_core::{GCodeFlavor, ProjectSettings, gcode_processor};

#[test]
fn end_gcode_skips_m400_dwells_but_preserves_g4_and_resets_for_next_input() {
    let mut settings = ProjectSettings::default();
    settings.printer.gcode.gcode_flavor = GCodeFlavor::MarlinLegacy;
    let cases = [
        (
            "marked_m400",
            concat!(
                "M400 S2\n",
                "M400 P1000\n",
                "; MACHINE_END_GCODE_START\n",
                "M400 S3\n",
                "M400 P100\n",
                "G4 S7\n",
                "G4 P2000\n",
            ),
            12.0_f64,
        ),
        ("fresh_unmarked_m400", "M400 S3\nM400 P1000\n", 4.0_f64),
        (
            "wrong_tag",
            "; MACHINE_END_GCODE_START_extra\nM400 S3\nM400 P1000\n",
            4.0_f64,
        ),
        (
            "g4_after_end_tag",
            "; MACHINE_END_GCODE_START\nG4 S7\nG4 P2000\n",
            9.0_f64,
        ),
    ];
    let results = cases.map(|(name, input, expected)| {
        let output = gcode_processor::process_gcode(input, &settings, false, false);
        let actual = output.result.print_statistics.normal_time;
        eprintln!(
            "M400_END_GCODE_RED {}",
            serde_json::json!({
                "case": name,
                "input": input,
                "output": std::str::from_utf8(&output.gcode).unwrap(),
                "expected_seconds": expected,
                "actual_seconds": actual,
                "expected_bits": expected.to_bits(),
                "actual_bits": actual.to_bits(),
            })
        );
        assert_eq!(
            output.gcode,
            input.as_bytes(),
            "{name}: input bytes changed"
        );
        (name, actual.to_bits(), expected.to_bits())
    });
    for &(name, actual, expected) in &results[1..] {
        assert_eq!(actual, expected, "{name}: Native-source control time");
    }
    let (name, actual, expected) = results[0];
    assert_eq!(actual, expected, "{name}: Native end-gcode M400 skip");
}
