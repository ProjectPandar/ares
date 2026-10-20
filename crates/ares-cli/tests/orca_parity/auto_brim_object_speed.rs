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
    assert_eq!(case.native.contains(";TYPE:Brim\n"), has_brim);
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
        assert_eq!(text.contains(";TYPE:Brim\n"), has_brim);
        assert_eq!(brim_moves(text), native_moves);
    }
}

pub(super) fn brim_moves(gcode: &str) -> Vec<&str> {
    let mut active = false;
    let mut moves = Vec::new();
    for line in gcode.lines() {
        if line == ";TYPE:Brim" {
            active = true;
        } else if line.starts_with(";TYPE:") || line == ";LAYER_CHANGE" {
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
