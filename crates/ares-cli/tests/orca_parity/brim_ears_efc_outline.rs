//! Brim.cpp:56-62,504-551,311-350: BrimEars post-EFC outline consumer.

use crate::efc_outline_case::run;

const RAW_EFC: &[u8] =
    include_bytes!("../../../../tests/parity/brim-ears-efc-outline/ears-efc0p3-outline0.3mf");

#[test]
fn brim_ears_efc_outline_active() {
    run(
        "brim-ears-efc-active-",
        include_bytes!("../../../../tests/parity/brim-ears-efc-outline/ears-efc0p3-outline1.3mf"),
        "e0f7ed008f4ce90ee5c3f4bc1dcf2299cca14fdbf81178506090f80959b8af9f",
        Some((RAW_EFC, "brim-ears-efc-native-raw-")),
        assert_ears,
    );
}

#[test]
fn brim_ears_efc_outline_zero_disabled_control() {
    run(
        "brim-ears-efc-zero-off-",
        include_bytes!("../../../../tests/parity/brim-ears-efc-outline/ears-efc0-outline0.3mf"),
        "f9b7119c2497bd4a583ead2052d45715177813294c93f4732596ef013c2ba818",
        None,
        assert_ears,
    );
}

#[test]
fn brim_ears_efc_outline_zero_enabled_control() {
    run(
        "brim-ears-efc-zero-on-",
        include_bytes!("../../../../tests/parity/brim-ears-efc-outline/ears-efc0-outline1.3mf"),
        "324b0ef69892957449bb71d8192f18486409863e9b9078ef6a4768f9bc3d355a",
        None,
        assert_ears,
    );
}

#[test]
fn brim_ears_efc_outline_flag_disabled_control() {
    run(
        "brim-ears-efc-flag-off-",
        RAW_EFC,
        "7fdba5182c5de418e2a7f00de94bf77ac39490296783fce257e0b4b2c8ccf617",
        None,
        assert_ears,
    );
}

fn assert_ears(text: &str) {
    assert_eq!(text.matches(";TYPE:Brim\n").count(), 2);
    assert!(has_skirt_extrusion(text));
}

fn has_skirt_extrusion(gcode: &str) -> bool {
    let mut active = false;
    gcode.lines().any(|line| {
        if let Some(role) = line.strip_prefix(";TYPE:") {
            active = role == "Skirt";
        } else if line == ";LAYER_CHANGE" {
            active = false;
        }
        let words = line.split(';').next().unwrap().split_ascii_whitespace();
        active
            && line.starts_with("G1 ")
            && words.clone().any(|word| word.starts_with(['X', 'Y']))
            && words.clone().any(|word| {
                word.strip_prefix('E')
                    .is_some_and(|value| value.parse::<f64>().unwrap() > 0.0)
            })
    })
}
