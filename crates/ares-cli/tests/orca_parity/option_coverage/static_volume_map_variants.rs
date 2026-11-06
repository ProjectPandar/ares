//! Print::update_filament_maps_to_config (Print.cpp:4656-4745) writes the static filament volume map
//! back on every printer the static grouping branch reaches (ToolOrdering.cpp:3054-3081), and
//! selects the filament variants again from the original full config (Print.cpp:4722-4745).
use super::variant_footer::check;

const CASE: (&str, &[&str]) = (
    "static-volume-map-variants",
    &[
        "ToolOrdering.cpp:3054-3081",
        "Print.cpp:4656-4711",
        "Print.cpp:4722-4745",
    ],
);

#[test]
fn ender3_high_flow_column_keeps_unused_standard_volume() {
    check(
        CASE,
        "ender3-variant-column",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/nozzle-footer/variant-column.3mf"
        )),
        "908612c123f73537d6cd07b3baaf93e5c48e97f84503030944c330ac3327f23d",
        (
            &[("filament_map", "1,1"), ("nozzle_volume_type", "High Flow")],
            &[("filament_volume_map", "0,1")],
        ),
    );
}

#[test]
fn bbl_two_filaments_write_one_volume_per_filament() {
    check(
        CASE,
        "x1c-positive-z-control",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/bbl-jerk-tail-precision/positive-z-control.3mf"
        )),
        "70db0f1afd38380c28f93b2d435b01120d49cb74a46cf3925c10ed6ddcf03afe",
        (
            &[("filament_self_index", "1,2")],
            &[("filament_volume_map", "0,0")],
        ),
    );
}

#[test]
fn bbl_plate_map_longer_than_consumables_keeps_one_variant_slot() {
    check(
        CASE,
        "x1c-cube-too-long",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/filament-map-length/x1c-cube-too-long.3mf"
        )),
        "3de8de84dee7bbbdb45530a0c247e79896f43ed253f69bb6a5020cc46375ca13",
        (
            &[("filament_map", "1")],
            &[
                ("filament_self_index", "1"),
                ("filament_volume_map", "0"),
                ("nozzle_temperature", "220"),
            ],
        ),
    );
}
