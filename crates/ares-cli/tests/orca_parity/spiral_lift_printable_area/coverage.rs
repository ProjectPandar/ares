use crate::runner::native_stream_case;

fn assert_native(native: &str, label: &str, (arc, eager, interior): (bool, bool, bool)) {
    if let Some(physical) = label.strip_prefix("physical-") {
        let map = if physical == "1" { "0,0" } else { "1,1" };
        assert!(native.contains(&format!("; filament_map_2 = {map}\n")));
        assert!(native.contains("133x0,256x0,256x256,133x256"));
        let lift = if physical == "1" {
            "G3 Z1 I-.864 J.857 P1  F60000"
        } else {
            "G1 Z1 F60000"
        };
        assert!(native.lines().any(|line| line == lift));
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
        "44465492c87e179ac3da6c48599600f424799ddcc3aa73de12ede4ce3879e1de",
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
        "047bf1790e5cdd9747671c9f0bae018680751f929b612e99907cc4b1a9f7b024",
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
        "122dfa08f498a8b8cde6eb50fe0449a05486ca6153b37cf80e293f824972b9b3",
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
        "ea95c1338b13087950c6d40a33b8bb9d9b18a6f824edd85df2e095a39f5e0559",
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
        "718fc367e21e92030f0c3c37ea2c5ea74e536ebc474a9081a2ee24c569e1b35c",
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
        "71715ee49512ce481923f68ae0a645286dcfaa50969a3238085febee64ab37ef",
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
