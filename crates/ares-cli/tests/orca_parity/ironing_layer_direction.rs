use crate::runner::{application, native_stream_case};
use std::fs;

const UPSTREAM: (&str, &[&str]) = (
    "ironing-layer-direction",
    &["Fill.cpp:1796-1806,1899-1903", "FillBase.cpp:454-458"],
);

type Point = (f64, f64);
type Stroke = (Point, Point);

fn ironing_strokes(gcode: &str) -> Vec<Stroke> {
    let mut ironing = false;
    let mut relative_e = false;
    let mut xy = (0.0, 0.0);
    let mut strokes = Vec::new();
    for line in gcode.lines() {
        if let Some(role) = line.strip_prefix(";TYPE:") {
            ironing = role == "Ironing";
        }
        let (code, _) = line.split_once(';').unwrap_or((line, ""));
        let mut words = code.split_whitespace();
        match words.next() {
            Some("M82") => relative_e = false,
            Some("M83") => relative_e = true,
            Some("G0" | "G1" | "G2" | "G3" | "G92") => {
                let start = xy;
                let mut e = 0.0;
                for word in words {
                    let (axis, value) = word.split_at(1);
                    match axis {
                        "X" => xy.0 = value.parse().unwrap(),
                        "Y" => xy.1 = value.parse().unwrap(),
                        "E" => e = value.parse().unwrap(),
                        _ => (),
                    }
                }
                if ironing && e > 0.0 && xy != start {
                    assert!(relative_e && code.starts_with("G1 "), "{line}");
                    strokes.push((start, xy));
                }
            }
            _ => (),
        }
    }
    strokes
}

fn check(label: &str, project: &[u8], sha: &str, heading: f64, check_waypoint: bool) {
    if crate::OrcaRunner::from_env().unwrap().is_none() {
        return;
    }
    let (reference, _) = native_stream_case::qualify(UPSTREAM, label, project, sha, false);
    let expected = ironing_strokes(&reference.native);
    let dx_dy = expected.iter().map(|&((x, y), (u, v))| (u - x, v - y));
    let (dx, dy) = dx_dy.clone().find(|(dx, dy)| dx.hypot(*dy) >= 2.0).unwrap();
    assert_eq!(dy.atan2(dx).to_degrees().rem_euclid(180.0).round(), heading);
    if check_waypoint {
        assert!(expected.iter().any(|&(_, xy)| xy == (99.305, 103.026)));
    }

    let output = reference.root.join("ares.cli.gcode");
    let cli = application::cli_command()
        .args(["slice", "-o"])
        .arg(&output)
        .arg(reference.root.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(reference.root.join("ares.cli.stderr"), cli.stderr).unwrap();
    fs::write(reference.root.join("ares.cli.stdout"), cli.stdout).unwrap();
    assert!(cli.status.success(), "CLI at {:?}", reference.root);
    let actual = ironing_strokes(&fs::read_to_string(output).unwrap());
    let first_difference = expected.iter().zip(&actual).position(|(n, a)| n != a);
    if let Some(i) = first_difference {
        panic!(
            "ironing stroke {}: Native {:?}, Ares {:?}; {:?}",
            i + 1,
            expected[i],
            actual[i],
            reference.root
        );
    }
    assert_eq!(actual.len(), expected.len(), "{:?}", reference.root);
}

#[test]
fn ironing_top_direction_overrides_solid_template() {
    check(
        "top-over-template",
        include_bytes!("../../../../tests/parity/ironing-layer-direction/top-over-template.3mf"),
        "3c2cb5e38a47dcbd1634e896819e9cdadf6bf24df68dc65bdd8b549581bbd5d0",
        40.0,
        false,
    );
}

#[test]
fn ironing_fixed_top_direction_does_not_alternate() {
    check(
        "fixed-top",
        include_bytes!("../../../../tests/parity/ironing-layer-direction/fixed-top.3mf"),
        "6cd8d6970b14c35fe76eb6194c5e0a09afde29f92189bb212bc2ee4c4d997d0b",
        54.0,
        false,
    );
}

#[test]
fn ironing_model_alignment_preserves_coordinate_precision() {
    check(
        "aligned",
        include_bytes!("../../../../tests/parity/ironing-layer-direction/aligned.3mf"),
        "75e8bf886d2641f5b7f2fa8bb62458384010ea6ca4cd659f64bab13eeaf3d819",
        48.0,
        true,
    );
}

#[test]
fn ironing_unset_top_uses_alternating_solid_direction() {
    check(
        "unset",
        include_bytes!("../../../../tests/parity/ironing-layer-direction/unset.3mf"),
        "b8a3a97025dfa47d5a7b4027c40f9c63dcbe524fbb4ae5434e86e2c1e18dcfe2",
        144.0,
        false,
    );
}

#[test]
fn ironing_explicit_fixed_angle_ignores_top_and_template() {
    check(
        "explicit-fixed",
        include_bytes!("../../../../tests/parity/ironing-layer-direction/explicit-fixed.3mf"),
        "68427bb6a9d594789043c65558bac245bb64e5c11a92a501307094cc4af11afb",
        17.0,
        false,
    );
}

#[test]
fn ironing_fixed_angle_without_alignment_uses_literal_angle() {
    check(
        "literal",
        include_bytes!("../../../../tests/parity/ironing-layer-direction/literal.3mf"),
        "ef2f1b051dc578142a9b20474da46308dd249fb1d13cb325b119416a31a5021f",
        48.0,
        true,
    );
}
