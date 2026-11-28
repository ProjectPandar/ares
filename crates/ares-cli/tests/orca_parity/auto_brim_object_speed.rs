//! Model.cpp:2976-3010,3222-3255 / Brim.cpp:477-501: object speed lookup.

use crate::{
    OrcaRunner,
    runner::native_api_cli::{NativeCase, sha},
};

const GLOBAL_AUTO: &[u8] =
    include_bytes!("../../../../tests/parity/auto-brim-object-speed/auto_brim-global-only.3mf");

#[test]
fn auto_brim_object_speed_lookup_inner_override() {
    run(
        "auto-brim-object-inner-",
        include_bytes!(
            "../../../../tests/parity/auto-brim-object-speed/auto_brim-object-inner60.3mf"
        ),
        "5524e86cb65791afff41b03f9ea00a6866bbc43978367c19829fab120ed57fd2",
        true,
        Some((GLOBAL_AUTO, "auto-brim-speed-native-control-")),
    );
}

#[test]
fn auto_brim_object_speed_lookup_global_control() {
    run(
        "auto-brim-global-speed-",
        GLOBAL_AUTO,
        "e862e3a09ab896c964f2caad70e974f880354c79e740238186cfc6ab609c27d5",
        true,
        None,
    );
}

#[test]
fn auto_brim_object_speed_lookup_no_brim_global_control() {
    run(
        "no-brim-global-speed-",
        include_bytes!("../../../../tests/parity/auto-brim-object-speed/no_brim-global-only.3mf"),
        "59ac1c8bac3851349e5d4ddb0645b46d8896f0778fea13cd692b4c652b2a80a5",
        false,
        None,
    );
}

#[test]
fn auto_brim_object_speed_lookup_no_brim_inner_control() {
    run(
        "no-brim-object-inner-",
        include_bytes!(
            "../../../../tests/parity/auto-brim-object-speed/no_brim-object-inner60.3mf"
        ),
        "219e0806e2ca7e0c04f888c3129c9940692608317b784e8d5384dfc57be86978",
        false,
        None,
    );
}

pub(super) fn run(
    prefix: &str,
    project: &[u8],
    expected_sha: &str,
    has_brim: bool,
    changed_brim_control: Option<(&[u8], &str)>,
) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    assert_eq!(sha(project), expected_sha);
    let case = NativeCase::slice(&runner, project, prefix);
    let native_moves = brim_moves(&case.native);
    assert_eq!(!native_moves.is_empty(), has_brim);
    if let Some((control_project, control_prefix)) = changed_brim_control {
        let control = NativeCase::slice(&runner, control_project, control_prefix);
        let control_moves = brim_moves(&control.native);
        assert!(!control_moves.is_empty());
        assert_ne!(
            native_moves, control_moves,
            "Native control changes actual Brim geometry"
        );
    }
    let (api, cli) = case.compare(ares_core::GenerationMetadata::deterministic(
        2026, 10, 20, 0, 0, 0,
    ));
    for bytes in [&api, &cli] {
        let text = std::str::from_utf8(bytes).unwrap();
        assert_eq!(brim_moves(text).is_empty(), !has_brim);
        assert_eq!(brim_moves(text), native_moves);
    }
}

#[test]
fn auto_brim_rotated_instance_exact_height() {
    run_rotated_instance_case(
        "auto-brim-rotated-exact-",
        include_bytes!("../../../../tests/parity/auto-brim-rotated-instance/quarter-auto.3mf"),
        "10bf09b0effe4958b9e5b84884948e2c6b7636fbecf2aad5e923e460e21e453e",
        false,
    );
}

#[test]
fn auto_brim_rotated_instance_no_brim_control() {
    run_rotated_instance_case(
        "auto-brim-rotated-none-",
        include_bytes!("../../../../tests/parity/auto-brim-rotated-instance/quarter-none.3mf"),
        "829bcd48df727ba4d9f1d10c244db24c0cfce8a2e8ad287909e0190e7b765edb",
        false,
    );
}

#[test]
fn auto_brim_rotated_instance_unrotated_control() {
    run_rotated_instance_case(
        "auto-brim-unrotated-control-",
        include_bytes!("../../../../tests/parity/auto-brim-rotated-instance/unrotated-auto.3mf"),
        "017fc72e3670c91b4a753fa3d9e796719899af427c1448307f1fd0f800f938b8",
        false,
    );
}

#[test]
fn auto_brim_rotated_instance_low_height_control() {
    run_rotated_instance_case(
        "auto-brim-rotated-low-",
        include_bytes!("../../../../tests/parity/auto-brim-rotated-instance/quarter-low-auto.3mf"),
        "512ed09c399bd5f4c2dba5f98219ccf62ed8f959cd77b506e363b77d443fac80",
        false,
    );
}

#[test]
fn auto_brim_rotated_instance_high_height_control() {
    run_rotated_instance_case(
        "auto-brim-rotated-high-",
        include_bytes!("../../../../tests/parity/auto-brim-rotated-instance/quarter-high-auto.3mf"),
        "cf4a6f584a5a2a4b33641d72979e9416b7532815fdde838caf05fa6ea2471176",
        true,
    );
}

fn run_rotated_instance_case(prefix: &str, project: &[u8], expected_sha: &str, has_brim: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    assert_eq!(sha(project), expected_sha);
    let case = NativeCase::slice(&runner, project, prefix);
    let native_moves = brim_moves(&case.native);
    assert_eq!(case.native.contains(";TYPE:Brim\\n"), has_brim);
    assert_eq!(!native_moves.is_empty(), has_brim);
    let (api, cli) = case.compare(ares_core::GenerationMetadata::deterministic(
        2026, 10, 20, 0, 0, 0,
    ));
    for bytes in [&api, &cli] {
        let text = std::str::from_utf8(bytes).unwrap();
        assert_eq!(text.contains(";TYPE:Brim\\n"), has_brim);
        assert_eq!(brim_moves(text), native_moves);
    }
}

pub(super) fn brim_moves(gcode: &str) -> Vec<&str> {
    let mut active = false;
    let mut moves = Vec::new();
    for line in gcode.lines() {
        if line == ";TYPE:Brim" || line == "; FEATURE: Brim" {
            active = true;
        } else if line.starts_with(";TYPE:")
            || line.starts_with("; FEATURE:")
            || line == ";LAYER_CHANGE"
        {
            active = false;
        }
        let words = line.split(';').next().unwrap().split_ascii_whitespace();
        if active
            && line.starts_with("G1 ")
            && words.clone().any(|word| word.starts_with(['X', 'Y']))
            && words.clone().any(|word| {
                word.strip_prefix('E')
                    .is_some_and(|value| value.parse::<f64>().unwrap() > 0.0)
            })
        {
            moves.push(line);
        }
    }
    moves
}
