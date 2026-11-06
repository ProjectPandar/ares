use crate::runner::native_stream_case;

fn assert_native(native: &str, label: &str, (arc, eager, interior): (bool, bool, bool)) {
    if let Some(physical) = label.strip_prefix("physical-") {
        let map = if physical == "1" { "0,0" } else { "1,1" };
        assert!(native.contains(&format!("; filament_map_2 = {map}\n")));
        assert!(native.contains("133x0,256x0,256x256,133x256"));
    } else if interior {
        assert!(native.lines().any(|line| {
            if arc {
                line.starts_with("G3 Z")
            } else {
                line.starts_with("G1 X")
                    && line.contains(" Y")
                    && line.contains(" Z")
                    && !line.contains(" E")
            }
        }));
    }
    if eager {
        let lifts: Vec<_> = native
            .rsplit("; PRINTABLE_AREA_EAGER\n")
            .skip(1)
            .filter_map(|prefix| {
                prefix
                    .lines()
                    .rev()
                    .find(|line| line.starts_with("G1 ") || line.starts_with("G3 "))
            })
            .collect();
        assert!(!lifts.is_empty());
        assert!(lifts.iter().any(|line| {
            if !interior {
                line.starts_with("G1 Z")
            } else if arc {
                line.starts_with("G3 Z")
            } else {
                line.starts_with("G1 X") && line.contains(" Z")
            }
        }));
    }
}

fn check(label: &str, project: &[u8], sha: &str, flags: (bool, bool, bool)) {
    native_stream_case::check(
        (
            "spiral-area-coverage",
            &[
                "GCodeWriter.cpp::apply_print_config:181-225",
                "GCodeWriter.cpp::active_printable_area:229-239",
                "GCodeWriter.cpp::eager_lift:950-965",
                "GCodeWriter.cpp::travel_to_xyz:1023-1036",
            ],
        ),
        label,
        project,
        sha,
        (false, |reference, _| {
            assert_native(&reference.native, label, flags)
        }),
    );
}

#[test]
fn interior_deferred_disabled_arc_retains_spiral() {
    check(
        "interior-deferred-arc-0",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/spiral-lift-printable-area/interior-deferred-arc-0.3mf"
        )),
        "96b858ed6d5dc6c5963bec18d41228c87588e96c0db4e2ee416de0af9299ef88",
        (false, false, true),
    );
}

#[test]
fn interior_deferred_enabled_arc_retains_spiral() {
    check(
        "interior-deferred-arc-1",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/spiral-lift-printable-area/interior-deferred-arc-1.3mf"
        )),
        "83d0f1cc5da23b1904516d3492c3324905610365b1cfa5458b58a8e8bcf6c2f3",
        (true, false, true),
    );
}

#[test]
fn interior_eager_disabled_arc_retains_spiral() {
    check(
        "interior-eager-arc-0",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/spiral-lift-printable-area/interior-eager-arc-0.3mf"
        )),
        "23f66318e3a5f81d454f1880b7d563526ab6ba0d19d020bf8d5a76d9e19bdc9e",
        (false, true, true),
    );
}

#[test]
fn interior_eager_enabled_arc_retains_spiral() {
    check(
        "interior-eager-arc-1",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/spiral-lift-printable-area/interior-eager-arc-1.3mf"
        )),
        "94e164709a3a1775b47b34fde04c9f54e9adac5c20d283aaeb6d5efaceaeb681",
        (true, true, true),
    );
}

#[test]
fn front_edge_eager_disabled_arc_uses_normal_lift() {
    check(
        "front-edge-eager-arc-0",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/spiral-lift-printable-area/front-edge-eager-arc-0.3mf"
        )),
        "ed0559f266f619ecc68a0b79a0660fa57ff9d9565448f622b4d3bbedf7d7eae3",
        (false, true, false),
    );
}

#[test]
fn front_edge_eager_enabled_arc_uses_normal_lift() {
    check(
        "front-edge-eager-arc-1",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/spiral-lift-printable-area/front-edge-eager-arc-1.3mf"
        )),
        "7017ae3b5916564705170ead727af56c77ce5fdc45b0af364884b7a8e15a5ec8",
        (true, true, false),
    );
}

#[test]
fn physical_head_one_uses_its_own_polygon() {
    check(
        "physical-1",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/spiral-lift-printable-area/physical-1.3mf"
        )),
        "f6710b699ee35c63c1b6b6f4eb41a129f0d168c4ea76bb2483085940f6ced6cc",
        (true, false, true),
    );
}

#[test]
fn physical_head_two_uses_its_own_polygon() {
    check(
        "physical-2",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/spiral-lift-printable-area/physical-2.3mf"
        )),
        "7bb9e8fc0e0c2e651fd334bb6e7564a1a2a8e20d1a78f80265b4b46334af64b5",
        (true, false, false),
    );
}
