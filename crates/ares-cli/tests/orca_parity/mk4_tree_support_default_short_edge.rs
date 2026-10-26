//! Organic default support anchor parity from `SupportCommon.cpp:486-570`.
use crate::runner::native_stream_case;

const UPSTREAM: (&str, &[&str]) = (
    "mk4-tree-support-default-short-edge",
    &[
        "SupportCommon.cpp:1791-1801::organic default base path dispatch",
        "SupportCommon.cpp:486-570::shrink_expolygon_with_contour_idx",
        "SupportCommon.cpp:508-511,531-537::positive short-edge threshold and signed Execute delta",
        "deps_src/clipper/clipper.cpp:3380-3406::ClipperOffset::AddPath short-edge filtering",
    ],
);

#[test]
fn mk4_tree_support_default_short_edge_target_matches_native() {
    native_stream_case::check(
        UPSTREAM,
        "target",
        include_bytes!("../../../../tests/parity/mk4-tree-support-default-short-edge/default.3mf"),
        "13b069f79fb3aebdddf9fd077298e80fed821247d1ee9bd855e35178bb058964",
        (false, |reference, _replay| {
            let layer = reference
                .native
                .split_once(";Z:2\n")
                .expect("target must reach print_z 2")
                .1
                .split(";LAYER_CHANGE")
                .next()
                .unwrap();
            assert!(
                layer.contains(";TYPE:Support\n"),
                "target layer must extrude support: {:?}",
                reference.root
            );
            assert!(
                layer.contains("G1 X124.217 Y119.368 Z2.4 F18000\n"),
                "target must retain the native anchor and lifted travel: {:?}",
                reference.root
            );
        }),
    );
}

#[test]
fn mk4_tree_support_default_short_edge_control_matches_native() {
    native_stream_case::check(
        UPSTREAM,
        "control",
        include_bytes!(
            "../../../../tests/parity/mk4-tree-support-default-short-edge/rectilinear-control.3mf"
        ),
        "339351a56db7c4df91e5f13aa2d358442ae50d6085e1ffa2a485e91e86ce0e00",
        (false, |reference, _replay| {
            assert!(
                reference.native.contains(";TYPE:Support\n"),
                "explicit rectilinear control must extrude support: {:?}",
                reference.root
            );
        }),
    );
}
