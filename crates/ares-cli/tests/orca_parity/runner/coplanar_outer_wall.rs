//! TriangleMeshSlicer.cpp:169-240,623-635,1180-1287.
use crate::runner::native_stream_case::qualify;

#[test]
fn lightning_coplanar_outer_wall_records_match_native() {
    check(
        "lightning",
        include_bytes!("../../../../../tests/parity/lightning-initial-overhang-join/positive.3mf"),
        "3622362104e0fb943cf8b4fb7277d168e8f6285633a5e21d10636e2f7c1472d1",
    );
}

#[test]
fn raft_control_coplanar_outer_wall_records_match_native() {
    check(
        "raft-control",
        include_bytes!("../../../../../tests/parity/raft-contact-first-flow/control.3mf"),
        "4a868581914a8b5dde68862978fe2678401eb4c784a7ca0e6ccc32632343704e",
    );
}

fn check(label: &str, bytes: &[u8], sha: &str) {
    let (reference, replay) = qualify(
        (
            "coplanar-outer-wall",
            &["TriangleMeshSlicer.cpp:169-240,623-635,1180-1287"],
        ),
        label,
        bytes,
        sha,
        false,
    );
    let api = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_project_plate_validated(
            bytes,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    let expected = outer_wall(&reference.native);
    assert!(!expected.is_empty());
    assert_eq!(outer_wall(&replay.native), expected);
    let actual = outer_wall(std::str::from_utf8(&api).unwrap());
    std::fs::write(reference.root.join("api-outer-wall.raw"), actual.join("\n")).unwrap();
    std::fs::write(
        reference.root.join("native-outer-wall.raw"),
        expected.join("\n"),
    )
    .unwrap();
    assert_eq!(actual, expected, "{label}: {:?}", reference.root);
}

fn outer_wall(gcode: &str) -> Vec<&str> {
    let mut outer = false;
    gcode
        .lines()
        .filter(|line| {
            if let Some(role) = line.strip_prefix(";TYPE:") {
                outer = role == "Outer wall";
            }
            if *line == ";LAYER_CHANGE" || *line == ";WIPE_START" {
                outer = false;
            }
            outer
                && ["G0 ", "G1 ", "G2 ", "G3 "]
                    .iter()
                    .any(|prefix| line.starts_with(prefix))
        })
        .collect()
}
