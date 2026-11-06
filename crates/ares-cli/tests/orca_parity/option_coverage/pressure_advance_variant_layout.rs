//! PrintConfig.cpp:10199-10205: the six pressure advance options are per-variant filament
//! options, laid out to the variant slots (PrintConfig.cpp:11431-11457). The per-filament refresh
//! grows a short destination with its own front value (OrcaSlicer.cpp:3698-3699, Config.hpp:712-715).
use super::variant_footer::check;

const CASE: (&str, &[&str]) = (
    "pressure-advance-variant-layout",
    &[
        "PrintConfig.cpp:10199-10205",
        "PrintConfig.cpp:11431-11457",
        "OrcaSlicer.cpp:3658-3793",
        "Config.hpp:698-715",
    ],
);

#[test]
fn ender3_four_slot_pressure_advance_labelled_section() {
    check(
        CASE,
        "ender3-four-slots",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/auto-brim-object-adhesion/auto_brim-pla.3mf"
        )),
        "ec2304a186a04bc1dcd0a7ad2497003f0c209fafd78eb5a66c90f044fc9ddb31",
        (
            &[("filament_self_index", "1,1,1,1")],
            &[
                ("enable_pressure_advance", "0,0,0,0"),
                ("pressure_advance", "0.02,0.02,0.02,0.02"),
                ("adaptive_pressure_advance", "0,0,0,0"),
                (
                    "adaptive_pressure_advance_model",
                    r#""0,0,0\n0,0,0";"0,0,0\n0,0,0";"0,0,0\n0,0,0";"0,0,0\n0,0,0""#,
                ),
                ("adaptive_pressure_advance_overhangs", "0,0,0,0"),
                ("adaptive_pressure_advance_bridges", "0,0,0,0"),
            ],
        ),
    );
}

#[test]
fn mk4_two_slot_pressure_advance_labelled_section() {
    check(
        CASE,
        "mk4-two-slots",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/mk4-layer-change/mk4base.3mf"
        )),
        "d0a4dfdb7f89120719b380a2995687a81a7e699025b8f3734349480bdebdc046",
        (
            &[("filament_self_index", "1,1")],
            &[
                ("enable_pressure_advance", "0,0"),
                ("pressure_advance", "0.02,0.02"),
                ("adaptive_pressure_advance", "0,0"),
                (
                    "adaptive_pressure_advance_model",
                    r#""0,0,0\n0,0,0";"0,0,0\n0,0,0""#,
                ),
                ("adaptive_pressure_advance_overhangs", "0,0"),
                ("adaptive_pressure_advance_bridges", "0,0"),
            ],
        ),
    );
}

#[test]
fn x1c_two_filaments_keep_authored_pressure_advance_labelled_section() {
    check(
        CASE,
        "x1c-authored-enable",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/bbl-pressure-advance-precedence/bbl-stock-pa-control.3mf"
        )),
        "1d56a96dfa6bb2d86ef8a2d29be12ee0ea395e25d96172ee9a07a87f70456b14",
        (
            &[("filament_self_index", "1,2")],
            &[
                ("enable_pressure_advance", "1,1"),
                ("pressure_advance", "0.02,0.02"),
            ],
        ),
    );
}
