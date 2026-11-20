use crate::runner::native_stream_case;

const UPSTREAM: (&str, &[&str]) = (
    "normal-raft-interface-arcs",
    &[
        "Layer.cpp:379-425::support path simplification",
        "libslic3r.h:91-92::SUPPORT_RESOLUTION",
        "ArcFitter.cpp:103-152::do_arc_fitting_and_simplify",
    ],
);

#[test]
fn x1c_normal_raft_interface_fits_arcs() {
    check(
        "x1c-enabled",
        include_bytes!("../../../../tests/parity/normal-raft-interface-arcs/x1c-enabled.3mf"),
        "91547e4890b7ff3e7859c6c07ebc64b42ef67b5834a7aa6aad6b867e3b5712d6",
        true,
    );
}

#[test]
fn x1c_normal_raft_interface_uses_lines_with_arc_fitting_disabled() {
    check(
        "x1c-disabled",
        include_bytes!("../../../../tests/parity/normal-raft-interface-arcs/x1c-disabled.3mf"),
        "1ef1725e032ac20349c09423cdefa6f797e72a477f67ff48508c00968dec1a17",
        false,
    );
}

#[test]
fn mk4_normal_raft_interface_fits_arcs() {
    check(
        "mk4-enabled",
        include_bytes!("../../../../tests/parity/normal-raft-interface-arcs/mk4-enabled.3mf"),
        "eed691bc95834bceace9856300f5c7f0612acf3334db0299d7337fe2628c282d",
        true,
    );
}

#[test]
fn mk4_normal_raft_interface_uses_lines_with_arc_fitting_disabled() {
    check(
        "mk4-disabled",
        include_bytes!("../../../../tests/parity/normal-raft-interface-arcs/mk4-disabled.3mf"),
        "ac0e520effb1cd6837c8506c8f385f485f9f7d092a0a4d7e477393eb28511cb3",
        false,
    );
}

fn check(label: &str, input: &[u8], sha: &str, arcs: bool) {
    native_stream_case::check(
        UPSTREAM,
        label,
        input,
        sha,
        (false, |reference, _| {
            let (mut layer, mut interface, mut linear, mut circular) = (false, false, 0, 0);
            for line in reference.native.lines() {
                if let Some(z) = line
                    .strip_prefix("; Z_HEIGHT: ")
                    .or_else(|| line.strip_prefix(";Z:"))
                {
                    layer = z == "0.5";
                    interface = false;
                }
                if let Some(role) = line
                    .strip_prefix("; FEATURE: ")
                    .or_else(|| line.strip_prefix(";TYPE:"))
                {
                    interface = role == "Support interface";
                }
                if layer && interface && line.contains(" E") {
                    linear += usize::from(line.starts_with("G1 ") && line.contains(" X"));
                    circular += usize::from(line.starts_with("G2 ") || line.starts_with("G3 "));
                }
            }
            assert!(
                linear > 0,
                "missing raft interface extrusions at Z0.5: {label}"
            );
            assert_eq!(
                circular > 0,
                arcs,
                "unexpected raft interface arc count ({circular}): {label}"
            );
        }),
    );
}
