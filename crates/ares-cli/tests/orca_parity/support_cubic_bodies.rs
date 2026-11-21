use super::adaptive_cubic_bodies::sparse_moves;
use crate::runner::native_stream_case;

const UPSTREAM: (&str, &[&str]) = (
    "support-cubic-bodies",
    &[
        "PrintObject.cpp:1169-1256",
        "Fill/FillAdaptive.cpp:1999-2064",
        "Fill/Fill.cpp:1482-1485,1670-1673",
    ],
);

fn separated_bodies((stock, stock_sha): (&[u8], &str), (support, support_sha): (&[u8], &str)) {
    if crate::OrcaRunner::from_env().unwrap().is_none() {
        return;
    }
    let (control, _) = native_stream_case::qualify(UPSTREAM, "stock", stock, stock_sha, false);
    let (enabled, _) =
        native_stream_case::qualify(UPSTREAM, "support", support, support_sha, false);
    let stock_moves = sparse_moves(&control.native);
    let support_moves = sparse_moves(&enabled.native);
    assert!(!stock_moves.is_empty() && !support_moves.is_empty());
    assert!(
        stock_moves != support_moves,
        "Support Cubic must change Native sparse geometry: {:?}",
        enabled.root
    );
    let metadata = ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0);
    control.compare(metadata);
    enabled.compare(metadata);
}

macro_rules! case {
    ($test:ident, $printer:literal, $stock_sha:literal, $support_sha:literal) => {
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
                        "../../../../tests/parity/support-cubic-bodies/",
                        $printer,
                        "-support.3mf"
                    )),
                    $support_sha,
                ),
            );
        }
    };
}

case!(
    support_cubic_preserves_separated_body_lattices_x1c,
    "x1c",
    "8c7a15c59f621035d65d6b513df0248c7ea669f5bb5aa524a7ddda7a4df15fe2",
    "7efcae8607cd3189a00418b50061ca72a62271ca8b2720420f722df029a8160e"
);
case!(
    support_cubic_preserves_separated_body_lattices_k1,
    "k1",
    "b02701b53d9ce1cd8203113c96006841531b55958640ccf17ec13bffd27d2cbf",
    "c83383aa2d7ed5e9db8be666befba47f6ddbef4941155058adcb66b9795e61fb"
);
case!(
    support_cubic_preserves_separated_body_lattices_mk4,
    "mk4",
    "6f46e288bcfca5a18be1e44fab089ac2cbac70542c6ee173ad5492bc19084814",
    "61b97bf2336e691dc20b7ae3b75913b93f00b3b7751fc9fa65229dd212b5da3e"
);
