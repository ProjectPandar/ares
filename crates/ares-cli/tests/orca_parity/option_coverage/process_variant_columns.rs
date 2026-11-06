//! PrintApply.cpp:1429 selects the process variant through update_values_to_printer_extruders,
//! which first synthesizes length-1 process columns from the printer's extruder_variant_list
//! (PrintConfig.cpp:11623-11660,11702-11703) and then selects the active column
//! (PrintConfig.cpp:11712-11730).
use super::variant_footer::check;

const CASE: (&str, &[&str]) = (
    "process-variant-columns",
    &[
        "PrintApply.cpp:1429",
        "PrintConfig.cpp:11623-11660",
        "PrintConfig.cpp:11702-11730",
    ],
);

const ENDER3_VARIANT_LIST: (&str, &str) = (
    "extruder_variant_list",
    "\"Direct Drive Standard,Direct Drive High Flow\"",
);

#[test]
fn ender3_high_flow_selects_synthesized_process_column() {
    check(
        CASE,
        "ender3-high-flow",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/nozzle-footer/variant-column.3mf"
        )),
        "908612c123f73537d6cd07b3baaf93e5c48e97f84503030944c330ac3327f23d",
        (
            &[ENDER3_VARIANT_LIST, ("nozzle_volume_type", "High Flow")],
            &[
                ("print_extruder_id", "1"),
                ("print_extruder_variant", "\"Direct Drive High Flow\""),
            ],
        ),
    );
}

#[test]
fn ender3_standard_selects_first_synthesized_process_column() {
    check(
        CASE,
        "ender3-standard",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/process-variant-columns/ender3-standard-nozzle.3mf"
        )),
        "036316b8d1fd464e9209a6efb85b71396aa197eb8cb0798ead31eec7a18133ca",
        (
            &[ENDER3_VARIANT_LIST, ("nozzle_volume_type", "Standard")],
            &[
                ("print_extruder_id", "1"),
                ("print_extruder_variant", "\"Direct Drive Standard\""),
            ],
        ),
    );
}
