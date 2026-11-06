//! PrintConfig.cpp:10207-10209,10214-10215: the cooling fan and nozzle temperature range options
//! are per-variant filament options, laid out to the variant slots (PrintConfig.cpp:11431-11457)
//! and selected per filament by the variant gather (PrintApply.cpp:1453-1455).
use super::variant_footer::check;

const CASE: (&str, &[&str]) = (
    "cooling-fan-nozzle-range-variants",
    &[
        "PrintConfig.cpp:10207-10209",
        "PrintConfig.cpp:10214-10215",
        "PrintApply.cpp:1453-1455",
    ],
);

const ENDER3_FOUR_SLOTS: &[(&str, &str)] = &[
    ("fan_min_speed", "100,100,100,100"),
    ("fan_max_speed", "100,100,100,100"),
    ("additional_cooling_fan_speed", "70,70,70,70"),
    ("nozzle_temperature_range_low", "190,190,190,190"),
    ("nozzle_temperature_range_high", "230,230,230,230"),
];

#[test]
fn ender3_four_slot_cooling_fan_and_nozzle_range_labelled_section() {
    check(
        CASE,
        "auto-brim-pla",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/auto-brim-object-adhesion/auto_brim-pla.3mf"
        )),
        "ec2304a186a04bc1dcd0a7ad2497003f0c209fafd78eb5a66c90f044fc9ddb31",
        (&[("filament_self_index", "1,1,1,1")], ENDER3_FOUR_SLOTS),
    );
}

#[test]
fn ender3_four_slot_two_tools_cooling_fan_and_nozzle_range_labelled_section() {
    check(
        CASE,
        "tools2-minimum10",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/skirt-minlength-multitool/tools2-minimum10.3mf"
        )),
        "cd9460b1c11e7dff39c5f1f4ffb60e9558489efd1edf7e3abb90ea3ff2a7caba",
        (&[("filament_self_index", "1,1,1,1")], ENDER3_FOUR_SLOTS),
    );
}

#[test]
fn bbl_two_filaments_selected_cooling_fan_and_nozzle_range_labelled_section() {
    check(
        CASE,
        "bbl-two-categories",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/bbl-type1-tower/two-categories.3mf"
        )),
        "09d8037066427023093b02b068e7c29dd36acb27d9680d173f82925e6eb42f3a",
        (
            &[("filament_self_index", "1,2")],
            &[
                ("fan_min_speed", "100,20"),
                ("fan_max_speed", "100,30"),
                ("additional_cooling_fan_speed", "70,70"),
                ("nozzle_temperature_range_low", "190,190"),
                ("nozzle_temperature_range_high", "240,240"),
            ],
        ),
    );
}
