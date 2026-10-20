//! Brim.cpp:120-148,477,501: object adhesion, group thermal length unchanged.

use crate::auto_brim_object_speed::run;

const PLA_AUTO: &[u8] =
    include_bytes!("../../../../tests/parity/auto-brim-object-adhesion/auto_brim-pla.3mf");

#[test]
fn auto_brim_object_adhesion_petg_active() {
    run(
        "auto-brim-adhesion-petg-",
        include_bytes!("../../../../tests/parity/auto-brim-object-adhesion/auto_brim-petg.3mf"),
        "fe3d354b46bd4ab237e9b185ddb4d66ce85b1d74baba385ca71fb2e7a1b2447b",
        true,
        Some((PLA_AUTO, "auto-brim-adhesion-native-pla-control-")),
    );
}

#[test]
fn auto_brim_object_adhesion_pla_control() {
    run(
        "auto-brim-adhesion-pla-",
        PLA_AUTO,
        "ec2304a186a04bc1dcd0a7ad2497003f0c209fafd78eb5a66c90f044fc9ddb31",
        true,
        None,
    );
}

#[test]
fn auto_brim_object_adhesion_no_brim_petg_control() {
    run(
        "no-brim-adhesion-petg-",
        include_bytes!("../../../../tests/parity/auto-brim-object-adhesion/no_brim-petg.3mf"),
        "c1f7b2f5ad04f7235c601043af25babe19a1065e1e0b9d1cbc4827e3440b0eb3",
        false,
        None,
    );
}

#[test]
fn auto_brim_object_adhesion_no_brim_pla_control() {
    run(
        "no-brim-adhesion-pla-",
        include_bytes!("../../../../tests/parity/auto-brim-object-adhesion/no_brim-pla.3mf"),
        "35de53e42d3622a52b8f76f8b628116f5907617d9f8cb1bb349a461323270f38",
        false,
        None,
    );
}
