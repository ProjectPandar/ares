use crate::runner::native_stream_case;

const SOURCE: (&str, &[&str]) = (
    "tree-raft-without-support",
    &[
        "PrintObject.cpp:998-1002,5088-5093",
        "Support/SupportParameters.hpp:215-221",
        "Support/TreeSupport.cpp:1901-1908",
        "Support/TreeSupport3D.cpp:1064-1082,3662-3673",
        "Support/SupportCommon.cpp:447-513",
    ],
);

fn support_moves_at_first_layer(gcode: &str) -> Option<&str> {
    let mut role = "";
    let mut first_layer = false;
    for line in gcode.lines() {
        if matches!(line, "; CHANGE_LAYER" | ";LAYER_CHANGE") {
            if first_layer {
                break;
            }
            first_layer = true;
        }
        if let Some(value) = line
            .strip_prefix("; FEATURE: ")
            .or_else(|| line.strip_prefix(";TYPE:"))
        {
            role = value;
        }
        if first_layer
            && matches!(role, "Support" | "Support interface")
            && line.starts_with("G1 ")
            && (line.contains(" X") || line.contains(" Y"))
            && line.contains(" E")
        {
            return Some(role);
        }
    }
    None
}

macro_rules! raft_case {
    ($test:ident, $file:literal, $sha:literal, $role:expr) => {
        #[test]
        fn $test() {
            native_stream_case::check(
                SOURCE,
                $file,
                include_bytes!(concat!(
                    "../../../../tests/parity/tree-raft-without-support/",
                    $file,
                    ".3mf"
                )),
                $sha,
                (false, |reference, _| {
                    assert!(
                        reference.native.contains("; Z_HEIGHT: 0.2\n")
                            || reference.native.contains(";Z:0.2\n"),
                        "expected a Native layer at 0.2 mm: {:?}",
                        reference.root
                    );
                    assert_eq!(
                        support_moves_at_first_layer(&reference.native),
                        $role,
                        "Native first-layer support activation: {:?}",
                        reference.root
                    );
                }),
            );
        }
    };
}

raft_case!(
    x1c_without_raft_emits_only_object,
    "x1c-0",
    "017f59ccd121f93e824580350f693c25197531dff033a0bc310901be2f38ce18",
    None
);
raft_case!(
    x1c_tree_raft_prints_contact_without_object_support,
    "x1c-1",
    "50a25494a720faa35b65ce1d677e717d60b2010d1e7eabfd0cf62d53034227f7",
    Some("Support interface")
);
raft_case!(
    x1c_tree_raft_prints_base_without_object_support,
    "x1c-3",
    "a3178f84170f5c9dc6950050d9a80c1d56ec6ba622ca332d5d1b5ba070e66e00",
    Some("Support")
);
raft_case!(
    mk4_without_raft_emits_only_object,
    "mk4-0",
    "784578a9acee02352b26e4e69d6d8d5726f3031c892a491044cfecd4a61c4303",
    None
);
raft_case!(
    mk4_tree_raft_prints_contact_without_object_support,
    "mk4-1",
    "cf838a0768195abc142071d372def8643eeb54e8ba68821adbce1c31c1a6cb8c",
    Some("Support interface")
);
raft_case!(
    mk4_tree_raft_prints_base_without_object_support,
    "mk4-3",
    "d76aa80bdeca3f959eaae67057911d4f3c0d904c87c583c4aecb36dd8193a719",
    Some("Support")
);
