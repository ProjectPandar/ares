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
        "38f2aeba0b18b9f28b0e84cb01290a687d4310c0bddeed282d341943343dfe54",
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
        "1269ab69f9e3c3a76202094278d0ec5cd18fe978ed0de96d01398036e3150f87",
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
        "6cb35f55f11e7e77d7a521a1763629e880962e996d22d42a6577304ec5a6ce65",
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
        "106156bada89ed98a8d9fa774c7f1dcdf14ca71507ab1fe8a6eb8689aa8b1103",
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
        "29333c079d80736fcc8e608707e11057e39ef9450c47c26ef096e2cd71368893",
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
        "31732d4f8e9a8bd33688c46719a3d19aa1334605f32af6653e053d88fe38e784",
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
