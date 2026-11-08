//! Organic default support seam anchor parity from
//! `tree_supports_generate_paths` (`SupportCommon.cpp:642-843`).
//!
//! The short-edge threshold of the shrink helper is not covered by this
//! fixture: at this upstream revision no edge falls under it, so the output
//! is the same with the filter disabled.
use crate::runner::native_stream_case;

const UPSTREAM: (&str, &[&str]) = (
    "mk4-tree-support-default-seam-anchor",
    &[
        "SupportCommon.cpp:1955-1958::organic default base path dispatch",
        "SupportCommon.cpp:642-843::tree_supports_generate_paths",
        "SupportCommon.cpp:651-707::shrink_expolygon_with_contour_idx",
        "SupportCommon.cpp:758-827::seam anchor search and cut",
    ],
);

/// A travel with X and Y that rises above the layer's print height.
fn is_lifted_travel(line: &str, print_z: f64) -> bool {
    let Some(words) = line.strip_prefix("G1 ") else {
        return false;
    };
    let has = |axis| words.split(' ').any(|word| word.starts_with(axis));
    has('X')
        && has('Y')
        && !has('E')
        && words.split(' ').any(|word| {
            word.strip_prefix('Z')
                .and_then(|z| z.parse::<f64>().ok())
                .is_some_and(|z| z > print_z)
        })
}

/// Some layer of the stream extrudes support and lifts for a travel.
fn has_support_layer_with_lifted_travel(native: &str) -> bool {
    native.split(";LAYER_CHANGE").skip(1).any(|layer| {
        let print_z = layer
            .lines()
            .find_map(|line| line.strip_prefix(";Z:"))
            .and_then(|z| z.parse::<f64>().ok());
        print_z.is_some_and(|print_z| {
            layer.contains(";TYPE:Support\n")
                && layer.lines().any(|line| is_lifted_travel(line, print_z))
        })
    })
}

#[test]
fn mk4_tree_support_default_seam_anchor_matches_native() {
    native_stream_case::check(
        UPSTREAM,
        "seam-anchor",
        include_bytes!("../../../../tests/parity/mk4-tree-support-default-short-edge/default.3mf"),
        "13b069f79fb3aebdddf9fd077298e80fed821247d1ee9bd855e35178bb058964",
        (false, |reference, _replay| {
            assert!(
                has_support_layer_with_lifted_travel(&reference.native),
                "a support layer must lift for a travel: {:?}",
                reference.root
            );
        }),
    );
}

#[test]
fn mk4_tree_support_rectilinear_base_control_matches_native() {
    native_stream_case::check(
        UPSTREAM,
        "rectilinear-control",
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
