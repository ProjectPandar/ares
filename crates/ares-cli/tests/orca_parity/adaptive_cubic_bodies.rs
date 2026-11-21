use crate::runner::native_stream_case;

const UPSTREAM: (&str, &[&str]) = (
    "adaptive-cubic-bodies",
    &[
        "ConnectedBodies.cpp:27-89",
        "PrintObject.cpp:1115-1257",
        "Fill/FillAdaptive.hpp:43-60",
    ],
);

pub(super) fn sparse_moves(gcode: &str) -> Vec<(&str, &str)> {
    let mut sparse = false;
    let mut moves = Vec::new();
    for line in gcode.lines() {
        if let Some(role) = line
            .strip_prefix("; FEATURE: ")
            .or_else(|| line.strip_prefix(";TYPE:"))
        {
            sparse = role == "Sparse infill";
        }
        if !sparse || !line.starts_with("G1 ") {
            continue;
        }
        let mut x = None;
        let mut y = None;
        let mut extrusion = false;
        for word in line.split_ascii_whitespace().skip(1) {
            if word.starts_with(';') {
                break;
            }
            if let Some(value) = word.strip_prefix('X') {
                x = Some(value);
            } else if let Some(value) = word.strip_prefix('Y') {
                y = Some(value);
            } else if let Some(value) = word.strip_prefix('E') {
                extrusion = value.parse::<f64>().unwrap() > 0.0;
            }
        }
        if let (Some(x), Some(y), true) = (x, y, extrusion) {
            moves.push((x, y));
        }
    }
    moves
}

fn separated_bodies((stock, stock_sha): (&[u8], &str), (adaptive, adaptive_sha): (&[u8], &str)) {
    if crate::OrcaRunner::from_env().unwrap().is_none() {
        return;
    }
    let (control, _) = native_stream_case::qualify(UPSTREAM, "stock", stock, stock_sha, false);
    let (enabled, _) =
        native_stream_case::qualify(UPSTREAM, "adaptive", adaptive, adaptive_sha, false);
    let stock_moves = sparse_moves(&control.native);
    let adaptive_moves = sparse_moves(&enabled.native);
    assert!(!stock_moves.is_empty() && !adaptive_moves.is_empty());
    assert!(
        stock_moves != adaptive_moves,
        "Adaptive Cubic must change Native sparse geometry: {:?}",
        enabled.root
    );
    let metadata = ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0);
    control.compare(metadata);
    enabled.compare(metadata);
}

macro_rules! case {
    ($test:ident, $printer:literal, $stock_sha:literal, $adaptive_sha:literal) => {
        #[test]
        fn $test() {
            separated_bodies(
                (
                    include_bytes!(concat!(
                        "../../../../tests/parity/adaptive-cubic-bodies/",
                        $printer,
                        "-stock.3mf"
                    )),
                    $stock_sha,
                ),
                (
                    include_bytes!(concat!(
                        "../../../../tests/parity/adaptive-cubic-bodies/",
                        $printer,
                        "-adaptive.3mf"
                    )),
                    $adaptive_sha,
                ),
            );
        }
    };
}

case!(
    adaptive_cubic_preserves_separated_body_lattices_x1c,
    "x1c",
    "8c7a15c59f621035d65d6b513df0248c7ea669f5bb5aa524a7ddda7a4df15fe2",
    "86bf70d06dfda7f7c637e21d33dba89d0869d7c9a4d341d9b0b6f864c3d85bb0"
);
case!(
    adaptive_cubic_preserves_separated_body_lattices_k1,
    "k1",
    "b02701b53d9ce1cd8203113c96006841531b55958640ccf17ec13bffd27d2cbf",
    "86869a42792f30ca15c49f833f6574934054e85495d4d150cf2915bd48b28f35"
);
case!(
    adaptive_cubic_preserves_separated_body_lattices_mk4,
    "mk4",
    "6f46e288bcfca5a18be1e44fab089ac2cbac70542c6ee173ad5492bc19084814",
    "aaf4acb4fb2556d2c7c3b7b898d960e930e96bd487817b801c745dbf54a308ff"
);
