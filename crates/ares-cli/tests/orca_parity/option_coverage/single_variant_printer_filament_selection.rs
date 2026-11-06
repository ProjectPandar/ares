//! PrintApply.cpp:1453-1455 and Print.cpp:4679-4684 select the filament variants on a
//! single-variant printer once a filament defines several (`has_multi_variant_filament`,
//! PrintConfig.cpp:10829-10834).
use super::variant_footer::check;

const CASE: (&str, &[&str]) = (
    "single-variant-printer-filament-selection",
    &[
        "PrintApply.cpp:1421-1455",
        "Print.cpp:4679-4684",
        "PrintConfig.cpp:10829-10834",
    ],
);

const VORON_DIRECT_DRIVE_STANDARD: &[(&str, &str)] = &[
    ("filament_self_index", "1"),
    ("filament_extruder_variant", "\"Direct Drive Standard\""),
    ("filament_max_volumetric_speed", "12"),
    ("activate_air_filtration", "0"),
    ("nozzle_temperature", "220"),
];

#[test]
fn voron_six_variant_system_filament_labelled_section() {
    check(
        CASE,
        "voron-outer1234-5",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/acceleration-raw-f64/voron-outer1234_5.3mf"
        )),
        "cac9d740501e61bc2e32d149ad554e9289f98c28d04c65ee0e73aea726a55c58",
        (&[], VORON_DIRECT_DRIVE_STANDARD),
    );
}

#[test]
fn voron_six_variant_system_filament_scarf_labelled_section() {
    check(
        CASE,
        "voron-all-inner0",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/scarf-clip-end/voron-all-inner0.3mf"
        )),
        "63c8325570c7c2ab40beff07ef2894d696ace59457e94a53af1d18fb42a792ae",
        (&[], VORON_DIRECT_DRIVE_STANDARD),
    );
}
