//! PrintConfig.cpp:11431-11457: per-variant filament options are laid out to the
//! `filament_self_index` slots at 3MF load (PrintConfig.cpp:10094-10096) and after the CLI
//! defaults completion (OrcaSlicer.cpp:4250-4259).
use super::variant_footer::check;

const CASE: (&str, &[&str]) = (
    "load-filament-variant-normalize",
    &[
        "PrintConfig.cpp:11431-11457",
        "PrintConfig.cpp:10094-10096",
        "OrcaSlicer.cpp:4250-4259",
    ],
);

const ENDER3_FOUR_SLOTS: &[(&str, &str)] = &[
    ("activate_air_filtration", "0,0,0,0"),
    ("during_print_exhaust_fan_speed", "60,60,60,60"),
    ("nozzle_temperature", "220,220,220,220"),
    ("complete_print_exhaust_fan_speed", "80,80,80,80"),
];

#[test]
fn ender3_four_slot_project_singletons_labelled_section() {
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
fn ender3_four_slot_two_tools_labelled_section() {
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
fn mk4_two_slot_project_singletons_labelled_section() {
    check(
        CASE,
        "mk4base",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/mk4-layer-change/mk4base.3mf"
        )),
        "d0a4dfdb7f89120719b380a2995687a81a7e699025b8f3734349480bdebdc046",
        (
            &[("filament_self_index", "1,1")],
            &[
                ("activate_air_filtration", "0,0"),
                ("during_print_exhaust_fan_speed", "60,60"),
                ("nozzle_temperature", "220,220"),
                ("complete_print_exhaust_fan_speed", "80,80"),
            ],
        ),
    );
}

#[test]
fn ender3_four_slot_absent_keys_take_laid_out_defaults_labelled_section() {
    check(
        CASE,
        "absent-variant-keys",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/load-filament-variant-normalize/absent-variant-keys.3mf"
        )),
        "a8e7658679fafb80e11a73ca7b5025519e2621ad97362e34a85282f5e6f203c5",
        (&[("filament_self_index", "1,1,1,1")], ENDER3_FOUR_SLOTS),
    );
}
