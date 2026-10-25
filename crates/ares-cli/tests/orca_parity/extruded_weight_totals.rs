//! GCode.cpp:1675-1690 computes the template totals from the typed
//! per-extruder filament diameter/density; hard-coded defaults diverge the
//! moment a profile differs from 1.75/1.24.

use crate::runner::native_stream_case;

const DIR: &str = "extruded-weight-totals";
const UPSTREAM: &[&str] = &[
    "GCode.cpp:1675-1690;per-extruder w = v * filament_density * 0.001",
    "GCode.cpp:1648-1652;extruded_weight_total placeholder registration",
    "GCode.cpp:4638-4641;before-layer template render timing",
    "PrintConfig.cpp:11188;extruded_weight_total coFloat",
];

const STOCK_LABEL: &str = "mk4-stock";
const STOCK_SHA: &str = "ef89144a9eeaaf742bfdc521ee81442f44dbcb2dbbf5bc6c4bdee74763f8665b";
const STOCK_PROJECT: &[u8] =
    include_bytes!("../../../../tests/parity/extruded-weight-totals/mk4-stock.3mf");

const DENSITY_LABEL: &str = "mk4-density2";
const DENSITY_SHA: &str = "9125d70dd47f11b5e2c4e738e6f181889b5b519bfa599b1323d12ec83058781d";
const DENSITY_PROJECT: &[u8] =
    include_bytes!("../../../../tests/parity/extruded-weight-totals/mk4-density2.3mf");

const DIAMETER_LABEL: &str = "mk4-diameter185";
const DIAMETER_SHA: &str = "358c56127ba2a5dce4a9cf6211e95afe58e9a5faf1a7a7ffc84c097b083aa64a";
const DIAMETER_PROJECT: &[u8] =
    include_bytes!("../../../../tests/parity/extruded-weight-totals/mk4-diameter185.3mf");

fn assert_native_template_witness(native: &str, clamp: &str, weight: &str) {
    for marker in [
        "; total layer number: 50\n",
        "; printing object cube_with_hole.obj id:0 copy 0\n",
        "; stop printing object cube_with_hole.obj id:0 copy 0\n",
        ";BEFORE_LAYER_CHANGE\n",
        "M201 X4000 Y4000 Z200 E2500\n",
        "M201 X4000 Y4000\n",
        "M74 W0\n",
    ] {
        assert!(native.contains(marker), "Native witness missing {marker:?}");
    }
    assert!(
        native.contains(clamp),
        "Native witness missing clamp {clamp:?}"
    );
    assert!(
        native.contains(weight),
        "Native witness missing weight {weight:?}"
    );
}

#[test]
fn orca_parity_extruded_weight_totals_stock_mk4() {
    native_stream_case::check(
        (DIR, UPSTREAM),
        STOCK_LABEL,
        STOCK_PROJECT,
        STOCK_SHA,
        (false, |reference, replay| {
            for native in [&reference.native, &replay.native] {
                assert_native_template_witness(
                    native,
                    "M201 X3999.92 Y3999.92\n",
                    "M74 W0.07497\n",
                );
            }
        }),
    );
}

#[test]
fn orca_parity_extruded_weight_totals_density2_mk4() {
    native_stream_case::check(
        (DIR, UPSTREAM),
        DENSITY_LABEL,
        DENSITY_PROJECT,
        DENSITY_SHA,
        (false, |reference, replay| {
            for native in [&reference.native, &replay.native] {
                // Typed density 2.0: weight = volume * 2.0 * 0.001.
                assert_native_template_witness(
                    native,
                    "M201 X3999.87 Y3999.87\n",
                    "M74 W0.120919\n",
                );
            }
        }),
    );
}

#[test]
fn orca_parity_extruded_weight_totals_diameter185_mk4() {
    native_stream_case::check(
        (DIR, UPSTREAM),
        DIAMETER_LABEL,
        DIAMETER_PROJECT,
        DIAMETER_SHA,
        (false, |reference, replay| {
            for native in [&reference.native, &replay.native] {
                // Typed diameter 1.85: smaller cross-section volume, stock
                // density 1.24.
                assert_native_template_witness(
                    native,
                    "M201 X3999.92 Y3999.92\n",
                    "M74 W0.07497\n",
                );
            }
        }),
    );
}
