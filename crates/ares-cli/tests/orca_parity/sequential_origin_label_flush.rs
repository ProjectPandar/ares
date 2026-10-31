//! GCode.cpp:7466-7467 consumes object-change labels before sequential origin travel.

use crate::runner::native_stream_case;
use serde_json::json;
use std::fs;

const DIR: &str = "sequential-origin-label-flush";
const UPSTREAM: &[&str] = &[
    "GCode.cpp::_do_export:3321-3327;GCode::travel_to:7466-7467",
    "GCodeWriter.cpp::add_object_end_labels/add_object_change_labels:1181-1200",
];
const CASES: [(&str, &str, &[u8]); 2] = [
    (
        "k1-byobject-two-objects",
        "baa6c8d352d5e62c400cc450d50de643ce868fcc619a71352748f89aabe7aab1",
        include_bytes!("../../../../tests/parity/sequential-origin-label-flush/by-object.3mf"),
    ),
    (
        "k1-bylayer-two-objects",
        "855dbb94249d0ce5c31eb80a6641a86b63118cd560668fa00a62eeaa54460b12",
        include_bytes!("../../../../tests/parity/sequential-origin-label-flush/by-layer.3mf"),
    ),
];

fn assert_native_objects(native: &str) {
    assert_eq!(native.matches("EXCLUDE_OBJECT_DEFINE NAME=").count(), 2);
    for name in ["overhang.obj_id_0_copy_0", "cube_with_hole.obj_id_1_copy_0"] {
        assert!(native.contains(&format!("EXCLUDE_OBJECT_START NAME={name}\n")));
        assert!(native.contains(&format!("EXCLUDE_OBJECT_END NAME={name}\n")));
    }
}

#[test]
fn sequential_origin_label_target() {
    let (label, sha, project) = CASES[0];
    native_stream_case::check(
        (DIR, UPSTREAM),
        label,
        project,
        sha,
        (false, |reference, replay| {
            for native in [&reference.native, &replay.native] {
                assert_native_objects(native);
                assert!(
                    native.contains(
                        "EXCLUDE_OBJECT_END NAME=overhang.obj_id_0_copy_0\nG1 Z7 F30000\n"
                    )
                );
            }
        }),
    );
}

#[test]
fn sequential_origin_label_bylayer_control() {
    let (label, sha, project) = CASES[1];
    native_stream_case::check(
        (DIR, UPSTREAM),
        label,
        project,
        sha,
        (false, |reference, replay| {
            assert_native_objects(&reference.native);
            assert_native_objects(&replay.native);
        }),
    );
}

#[test]
fn sequential_origin_label_native_preflight() {
    let namespace = format!("{DIR}-preflight");
    let mut qualified = Vec::new();
    for (label, sha, project) in CASES {
        let (reference, replay) =
            native_stream_case::qualify((&namespace, UPSTREAM), label, project, sha, false);
        assert_native_objects(&reference.native);
        assert_native_objects(&replay.native);
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
