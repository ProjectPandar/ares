use crate::{OrcaRunner, golden, runner::native_api_cli::NativeCase, runner::native_stream_case};

const PLAIN: &[u8] = include_bytes!("../../../../tests/parity/seam-paint-spiral/k1-plain.3mf");
const PAINTED: &[u8] = include_bytes!("../../../../tests/parity/seam-paint-spiral/k1-front.3mf");
const UPSTREAM: (&str, &[&str]) = ("seam-paint-spiral", &["GCode.cpp:6657-6671,8526-8549"]);

fn wall_z_moves(layer: &str) -> Vec<bool> {
    let mut outer_wall = false;
    let mut extrusion = Vec::new();
    for line in layer.lines() {
        if let Some(role) = line.strip_prefix(";TYPE:") {
            outer_wall = role == "Outer wall";
        }
        if outer_wall && let Some(command) = line.strip_prefix("G1 ") {
            let command = command.split(';').next().unwrap();
            let has = |axis| {
                command
                    .split_whitespace()
                    .any(|word| word.starts_with(axis))
            };
            if (has('X') || has('Y')) && has('E') {
                extrusion.push(has('Z'));
            }
        }
    }
    extrusion
}

#[test]
fn unpainted_vase_prints_flat_bottom_shells_and_spiral_walls() {
    native_stream_case::check(
        UPSTREAM,
        "k1-plain",
        PLAIN,
        "6bfc1ab33cc8390a551d946d11e8e7759be5fbffc7273b4d8765b690fddd9dbb",
        (false, |_, _| {}),
    );
}

#[test]
fn seam_paint_changes_neither_flat_bottom_shells_nor_spiral_walls() {
    native_stream_case::check(
        UPSTREAM,
        "k1-front",
        PAINTED,
        "40272fa603e71328f09bacdb79db05314e485127b25e6953d029a1affc5ab778",
        (false, |painted, _| {
            let runner = OrcaRunner::from_env().unwrap().unwrap();
            let plain = NativeCase::slice(&runner, PLAIN, "seam-paint-spiral-unpainted-");
            let normalize = |case: &NativeCase| {
                golden::normalize_one_generator_line(
                    case.native.as_bytes(),
                    golden::GeneratorKind::Orca,
                )
                .unwrap()
            };
            if let Some(difference) =
                golden::first_difference(&normalize(&plain), &normalize(painted))
            {
                panic!("seam paint changed Native spiral output: {difference}");
            }
            let layers: Vec<_> = painted.native.split("\n;LAYER_CHANGE\n").skip(1).collect();
            assert!(
                layers.len() > 3,
                "Native must print walls above the bottom shells"
            );
            for layer in &layers[..3] {
                let moves = wall_z_moves(layer);
                assert!(
                    !moves.is_empty() && moves.iter().all(|z| !z),
                    "bottom shell must have a flat outer wall"
                );
            }
            assert!(
                wall_z_moves(layers[3]).iter().any(|z| *z),
                "first vase layer must extrude while changing Z"
            );
        }),
    );
}
