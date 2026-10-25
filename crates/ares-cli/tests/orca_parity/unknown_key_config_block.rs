//! Config.cpp:608-629 drops project-settings keys unknown to the definition
//! table (no def, no alias), and GCode.cpp:5591-5646 serializes only defined
//! keys — an unknown key must never reach the `; CONFIG_BLOCK`.

use crate::runner::native_stream_case;

const DIR: &str = "unknown-key-config-block";
const UPSTREAM: &[&str] = &[
    "Config.cpp::set_deserialize_nothrow:576-590;handle_legacy clears+records",
    "Config.cpp::set_deserialize_raw:608-629;UnknownOptionException without def/alias",
    "GCode.cpp::append_full_config:5591-5646;defined non-banned non-nil keys only",
];

const CONTROL_LABEL: &str = "p1s-bylayer-control";
const CONTROL_SHA: &str = "2ff89639d4f6c6898393690e3236b6a1fff7edc116c3ba77c3eb8f98de5ff6ca";
const CONTROL_PROJECT: &[u8] =
    include_bytes!("../../../../tests/parity/unknown-key-config-block/p1s-bylayer-control.3mf");

const UNKNOWN_KEY_LABEL: &str = "p1s-bylayer-unknown-key";
const UNKNOWN_KEY_SHA: &str = "026a736275d93c7a88ce979b67afeeab088625de2f1c5e05fc08a7c840546565";
const UNKNOWN_KEY_PROJECT: &[u8] =
    include_bytes!("../../../../tests/parity/unknown-key-config-block/p1s-bylayer-unknown-key.3mf");

fn assert_native_witness(native: &str) {
    for marker in [
        "; printer_model = Bambu Lab P1S\n",
        "; print_sequence = by layer\n",
        "; total layer number: 50\n",
        "; model label id: 45,56\n",
    ] {
        assert!(native.contains(marker), "Native witness missing {marker:?}");
    }
    // The unknown key exists only in the target project settings; upstream
    // drops it during load (Config.cpp:576-590/608-629), so it must not be
    // serialized into the CONFIG_BLOCK (GCode.cpp:5591-5646).
    assert!(
        !native.contains("; first_layer_size_compensation"),
        "Native serialized the unknown key"
    );
}

#[test]
fn orca_parity_unknown_key_config_block_control_p1s() {
    native_stream_case::check(
        (DIR, UPSTREAM),
        CONTROL_LABEL,
        CONTROL_PROJECT,
        CONTROL_SHA,
        (false, |reference, replay| {
            assert_native_witness(&reference.native);
            assert_native_witness(&replay.native);
        }),
    );
}

#[test]
fn orca_parity_unknown_key_config_block_target_p1s() {
    native_stream_case::check(
        (DIR, UPSTREAM),
        UNKNOWN_KEY_LABEL,
        UNKNOWN_KEY_PROJECT,
        UNKNOWN_KEY_SHA,
        (false, |reference, replay| {
            assert_native_witness(&reference.native);
            assert_native_witness(&replay.native);
        }),
    );
}
