//! FillBase.cpp:1915: valid previous extension wins even when next rises farther.

use crate::runner::native_stream_case;
use serde_json::json;
use std::fs;

const DIR: &str = "support-base-extension-priority";
const UPSTREAM: &[&str] = &[
    "FillBase.cpp:1834-1948;1914-1915;2326",
    "FillRectilinear.cpp:3610-3634;SupportCommon.cpp::generate_support_toolpaths",
];
const CASES: [(&str, &str, &[u8]); 3] = [
    (
        "k1-tree-auto-overhang",
        "33023baaf33cdab1f5753cbc5beea7e104daa0d888f7a8daad1e1dddf8145d63",
        include_bytes!("../../../../tests/parity/support-base-extension-priority/tree.3mf"),
    ),
    (
        "k1-stock-overhang",
        "b3f72543f8794281e1d99ef784a993dbb0523bfbe96468403cec4b21da34a5a2",
        include_bytes!("../../../../tests/parity/support-base-extension-priority/stock.3mf"),
    ),
    (
        "k1-normal-auto-overhang",
        "05545ab741fbd56710dfc41aa5185654ad9b586901aa37d1e6fff34813cfb5fb",
        include_bytes!("../../../../tests/parity/support-base-extension-priority/normal.3mf"),
    ),
];

#[test]
fn support_base_tree_auto_overhang() {
    let (label, sha, project) = CASES[0];
    native_stream_case::check((DIR, UPSTREAM), label, project, sha, (false, |_, _| {}));
}

#[test]
fn support_base_stock_control() {
    let (label, sha, project) = CASES[1];
    native_stream_case::check((DIR, UPSTREAM), label, project, sha, (false, |_, _| {}));
}

#[test]
fn support_base_normal_auto_control() {
    let (label, sha, project) = CASES[2];
    native_stream_case::check((DIR, UPSTREAM), label, project, sha, (false, |_, _| {}));
}

#[test]
fn support_base_native_preflight() {
    let namespace = format!("{DIR}-preflight");
    let mut qualified = Vec::new();
    for (label, sha, project) in CASES {
        let (reference, replay) =
            native_stream_case::qualify((&namespace, UPSTREAM), label, project, sha, false);
        for native in [&reference, &replay] {
            let has_support = native.native.contains(";TYPE:Support\n")
                || native.native.contains(";TYPE:Support interface\n");
            assert_eq!(has_support, label != "k1-stock-overhang", "{label}");
        }
        qualified.push(json!({"case":label,"input_sha256":sha,
            "reference":reference.root,"replay":replay.root}));
    }
    fs::write(
        crate::artifacts::root_from_env()
            .unwrap()
            .join(format!("{namespace}.json")),
        serde_json::to_vec_pretty(&qualified).unwrap(),
    )
    .unwrap();
}
