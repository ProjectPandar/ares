use super::fixture::{Overrides, configured};
use serde_json::json;

#[test]
fn machine_end_template_reads_decimal_filament_retraction_and_machine_defaults() {
    if !crate::runner::reference_available() {
        return;
    }
    let (reference, actual) = configured(
        "",
        "",
        false,
        Overrides {
            machine: &[
                ("retraction_speed", json!(["30", "30"])),
                ("deretraction_speed", json!(["32", "32"])),
                (
                    "machine_end_gcode",
                    json!(
                        "; retraction rates {retraction_speed[0]} {retraction_speed[1]}\n; deretraction rates {deretraction_speed[0]} {deretraction_speed[1]}"
                    ),
                ),
            ],
            filaments: &[
                ("filament_retraction_speed", ["5.25", "nil"]),
                ("filament_deretraction_speed", ["7.75", "nil"]),
            ],
            ..Default::default()
        },
    );
    let expected = "\n; retraction rates 5.25 30\n; deretraction rates 7.75 32\n";
    assert!(std::str::from_utf8(&reference).unwrap().contains(expected));
    crate::golden::compare_ordered_bytes_generator_only(&reference, &actual).unwrap();
}
